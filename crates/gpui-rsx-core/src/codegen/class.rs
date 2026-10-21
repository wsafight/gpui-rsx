//! Class 字符串解析
//!
//! 将 CSS class 字符串解析为 GPUI 方法调用链，支持：
//! - Tailwind 风格的实用类（flex, gap-4, text-red-500）
//! - 任意颜色/长度值（`bg-[#ff0000]`、`w-[280px]`）
//! - 状态前缀（hover:、focus:、focus-visible:、active:、group-hover:、group-active:）
//! - `rsx.toml` 设计令牌（bg-primary、rounded-card）
//!
//! 所有 class 都经过 [`analyze_class`] 统一分类，宏展开和外部工具可共用同一结果：
//! - 支持 → 生成方法调用
//! - 未知但是合法标识符 → 透传为同名方法（GPUI 或第三方 trait），始终给出扩展方法提示和可用的拼写建议
//! - 无法表达的写法（如 `dark:`、`md:`）→ 宽松模式编译期警告，严格模式编译错误
//! - 非法值（如 `w-[abc]`）→ 编译错误
//!
//! 警告借助 `#[deprecated]`：生成一个带说明的已弃用单元结构体并立即使用，
//! stable rustc 会在 class 字面量位置报出黄色 warning。

use super::tables::*;
use crate::theme::Theme;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use std::sync::OnceLock;

/// 数值 spacing / sizing class 的单位。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SpacingScale {
    /// `gap-4` → `px(4.0)`（默认，兼容 0.x 行为）。
    #[default]
    Px,
    /// Tailwind 刻度：`gap-4` → `px(16.0)`（1 单位 = 0.25rem = 4px）。
    Tailwind,
}

impl SpacingScale {
    pub const fn factor(self) -> f32 {
        match self {
            Self::Px => 1.0,
            Self::Tailwind => 4.0,
        }
    }
}

/// class 处理模式。
#[derive(Clone, Copy, Debug)]
pub struct ClassMode {
    /// 严格模式：未知 class 编译报错，动态未知 class 默认 panic。
    pub strict: bool,
    pub scale: SpacingScale,
    pub theme: Option<&'static Theme>,
    /// 动态 class 中被忽略的 class 在进程退出时汇总打印（debug 构建），而不是逐条打印。
    pub report_dynamic_at_exit: bool,
}

impl ClassMode {
    pub const PERMISSIVE: Self = Self {
        strict: false,
        scale: SpacingScale::Px,
        theme: None,
        report_dynamic_at_exit: false,
    };
    pub const STRICT: Self = Self {
        strict: true,
        ..Self::PERMISSIVE
    };

    pub const fn with_scale(self, scale: SpacingScale) -> Self {
        Self { scale, ..self }
    }

    pub const fn with_theme(self, theme: Option<&'static Theme>) -> Self {
        Self { theme, ..self }
    }

    pub const fn with_dynamic_report(self, report_dynamic_at_exit: bool) -> Self {
        Self {
            report_dynamic_at_exit,
            ..self
        }
    }

    pub const fn is_strict(self) -> bool {
        self.strict
    }
}

/// Tailwind 状态前缀对应的 GPUI 样式细化方法。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    /// `hover`、`focus`、`focus_visible`、`active`、`group_hover`、`group_active`。
    pub method: &'static str,
    /// `group-*` 前缀的组名（`group-hover:` 为 `""`，`group-hover/menu:` 为 `"menu"`）。
    pub group: Option<String>,
}

impl Variant {
    fn call(&self, inner: &[TokenStream]) -> TokenStream {
        let method = syn::Ident::new(self.method, Span::call_site());
        match &self.group {
            Some(group) => quote! { .#method(#group, |__style| __style #(#inner)*) },
            None => quote! { .#method(|__style| __style #(#inner)*) },
        }
    }
}

pub const SUPPORTED_VARIANTS: &[&str] = &[
    "hover",
    "focus",
    "focus-visible",
    "active",
    "group-hover",
    "group-active",
];

const RESPONSIVE_VARIANTS: &[&str] = &["sm", "md", "lg", "xl", "2xl"];

fn parse_variant(prefix: &str) -> Result<Variant, String> {
    let (name, group) = match prefix.split_once('/') {
        Some((name, group)) => (name, Some(group)),
        None => (prefix, None),
    };
    let variant = |method| Variant {
        method,
        group: None,
    };
    let grouped = |method: &'static str| Variant {
        method,
        group: Some(group.unwrap_or("").to_owned()),
    };
    match (name, group) {
        ("hover", None) => Ok(variant("hover")),
        ("focus", None) => Ok(variant("focus")),
        ("focus-visible", None) => Ok(variant("focus_visible")),
        ("active", None) => Ok(variant("active")),
        ("group-hover", _) => Ok(grouped("group_hover")),
        ("group-active", _) => Ok(grouped("group_active")),
        ("dark", _) => Err(format!(
            "`{prefix}:` variants are not supported: GPUI has no color-scheme media query.\n\
             \x20 help: define the color as a theme token in rsx.toml (e.g. `bg-surface`), or use `when={{(is_dark, |el| ...)}}`"
        )),
        (name, _) if RESPONSIVE_VARIANTS.contains(&name) => Err(format!(
            "Responsive variant `{prefix}:` is not supported: GPUI windows have no CSS breakpoints.\n\
             \x20 help: compute the condition from the window size and use `when={{(is_wide, |el| ...)}}`"
        )),
        _ => {
            let hint = suggest(name, SUPPORTED_VARIANTS.iter().copied())
                .map(|s| format!(" Did you mean `{s}:`?"))
                .unwrap_or_default();
            Err(format!(
                "Unsupported variant `{prefix}:`.{hint}\n\
                 \x20 note: supported variants are hover:, focus:, focus-visible:, active:, group-hover:, group-active:"
            ))
        }
    }
}

/// 单个 class 的分析结果。
#[derive(Clone, Debug)]
pub enum ClassOutcome {
    /// 已知 class，生成的方法调用片段（不含状态前缀包装）。
    Supported(TokenStream),
    /// 未知但形如标识符：透传为 `.method()`。始终警告，`suggestion` 存在时附带拼写建议。
    PassThrough {
        tokens: TokenStream,
        suggestion: Option<String>,
    },
    /// 宽松模式下无法表达、被忽略的 class（编译期警告）。
    Ignored { message: String },
    /// 编译错误（严格模式未知 class、非法取值等）。
    Error { message: String },
}

#[derive(Clone, Debug)]
pub struct ClassAnalysis {
    pub variant: Option<Variant>,
    pub outcome: ClassOutcome,
}

/// 工具（lint / LSP）使用的诊断级别。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassDiagnostic {
    pub class: String,
    /// class 在原字符串中的字节偏移。
    pub offset: usize,
    pub severity: Severity,
    pub message: String,
    pub suggestion: Option<String>,
}

/// 分析单个 class（含状态前缀）。
pub fn analyze_class(class: &str, mode: ClassMode) -> ClassAnalysis {
    let Some((prefix, rest)) = class.split_once(':') else {
        return ClassAnalysis {
            variant: None,
            outcome: base_outcome(class, mode),
        };
    };

    let reject = |message: String| ClassAnalysis {
        variant: None,
        outcome: if mode.strict {
            ClassOutcome::Error { message }
        } else {
            ClassOutcome::Ignored { message }
        },
    };

    if rest.contains(':') {
        return reject(format!(
            "Stacked variants are not supported in `{class}`.\n\
             \x20 help: use a single variant such as `hover:` per class"
        ));
    }
    let variant = match parse_variant(prefix) {
        Ok(variant) => variant,
        Err(message) => return reject(message),
    };
    if is_stateful_class(rest)
        || matches!(rest, "debug-outline" | "group")
        || rest.starts_with("group/")
    {
        return ClassAnalysis {
            variant: None,
            outcome: ClassOutcome::Error {
                message: format!(
                    "Class `{rest}` cannot be used inside the `{prefix}:` variant.\n\
                     \x20 note: variants compile to GPUI StyleRefinement closures, which only accept style classes"
                ),
            },
        };
    }

    ClassAnalysis {
        variant: Some(variant),
        outcome: base_outcome(rest, mode),
    }
}

/// 为 lint / LSP 输出 class 字符串中所有需要提示的 class。
pub fn class_diagnostics(class_str: &str, mode: ClassMode) -> Vec<ClassDiagnostic> {
    let mut out = Vec::new();
    let base = class_str.as_ptr() as usize;
    for class in class_str.split_ascii_whitespace() {
        let offset = class.as_ptr() as usize - base;
        let (severity, message, suggestion) = match analyze_class(class, mode).outcome {
            ClassOutcome::Supported(_) => continue,
            ClassOutcome::PassThrough { suggestion, .. } => (
                Severity::Warning,
                passthrough_message(class, suggestion.as_deref()),
                suggestion,
            ),
            ClassOutcome::Ignored { message } => {
                (Severity::Warning, message, suggest_class(class, mode))
            }
            ClassOutcome::Error { message } => {
                (Severity::Error, message, suggest_class(class, mode))
            }
        };
        out.push(ClassDiagnostic {
            class: class.to_owned(),
            offset,
            severity,
            message,
            suggestion,
        });
    }
    out
}

/// 解析 class 字符串为方法链片段。
///
/// `"flex hover:bg-blue-600"` → `[.flex(), .hover(|__style| __style.bg(rgb(...)))]`
///
/// `span` 指向 class 字面量，用于把警告和错误定位到用户代码。
pub fn parse_class_string_with_mode(
    class_str: &str,
    mode: ClassMode,
    span: Span,
) -> Vec<TokenStream> {
    let mut methods = Vec::new();
    let mut variants: Vec<(Variant, Vec<TokenStream>)> = Vec::new();

    for class in class_str.split_ascii_whitespace() {
        let analysis = analyze_class(class, mode);
        let tokens = match analysis.outcome {
            ClassOutcome::Supported(tokens) => tokens,
            ClassOutcome::PassThrough { tokens, suggestion } => {
                methods.push(warning(
                    &passthrough_message(class, suggestion.as_deref()),
                    span,
                ));
                tokens
            }
            ClassOutcome::Ignored { message } => {
                methods.push(warning(&message, span));
                continue;
            }
            ClassOutcome::Error { message } => {
                methods.push(compile_error_at(&message, span));
                continue;
            }
        };
        match analysis.variant {
            None => methods.push(tokens),
            Some(variant) => match variants.iter_mut().find(|(v, _)| *v == variant) {
                Some((_, group)) => group.push(tokens),
                None => variants.push((variant, vec![tokens])),
            },
        }
    }

    methods.extend(variants.iter().map(|(variant, inner)| variant.call(inner)));
    methods
}

/// 单个已知 class 的方法调用（供动态 class 快速路径使用）。
pub fn parse_single_class_with_mode(class: &str, mode: ClassMode) -> TokenStream {
    let methods = parse_class_string_with_mode(class, mode, Span::call_site());
    quote! { #(#methods)* }
}

fn base_outcome(class: &str, mode: ClassMode) -> ClassOutcome {
    match expand_known_class(class, mode) {
        Some(Ok(tokens)) => ClassOutcome::Supported(tokens),
        Some(Err(message)) => ClassOutcome::Error { message },
        None => unknown_outcome(class, mode),
    }
}

fn unknown_outcome(class: &str, mode: ClassMode) -> ClassOutcome {
    let suggestion = suggest_class(class, mode);
    if mode.strict {
        let message = if class.starts_with("font-") {
            format!(
                "Unsupported font weight class `{class}`. Supported font weight classes: \
                 font-thin, font-extralight, font-light, font-normal, font-medium, \
                 font-semibold, font-bold, font-extrabold, font-black."
            )
        } else {
            unsupported_class_message(class, suggestion.as_deref())
        };
        return ClassOutcome::Error { message };
    }

    let method_name = class.replace(['-', '/'], "_");
    if is_method_ident(&method_name) {
        let ident = syn::Ident::new(&method_name, Span::call_site());
        // Pass-through classes may be methods from other traits (gpui-component's `h_flex`
        // is two edits from `flex`), so only warn when the start of the name agrees too.
        let suggestion = suggestion.filter(|s| {
            class.len() >= 5 && s.len() >= 2 && class.as_bytes()[..2] == s.as_bytes()[..2]
        });
        ClassOutcome::PassThrough {
            tokens: quote! { .#ident() },
            suggestion,
        }
    } else {
        let hint = suggestion
            .map(|s| format!(" Did you mean `{s}`?"))
            .unwrap_or_default();
        ClassOutcome::Ignored {
            message: format!(
                "gpui-rsx ignored unsupported class `{class}`.{hint}\n\
                 \x20 help: use a supported class, an explicit GPUI method attribute, or `rsx_strict!` to make this an error"
            ),
        }
    }
}

fn is_method_ident(name: &str) -> bool {
    name.as_bytes()
        .first()
        .is_some_and(|b| b.is_ascii_lowercase() || *b == b'_')
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && syn::parse_str::<syn::Ident>(name).is_ok()
}

/// 已知 class 的展开。`None` 表示未知；`Some(Err)` 表示形式已知但取值非法。
fn expand_known_class(class: &str, mode: ClassMode) -> Option<Result<TokenStream, String>> {
    if let Some(theme) = mode.theme
        && let Some(tokens) = theme.expand_class(class)
    {
        return Some(Ok(tokens));
    }

    if let Some(token) = parse_font_weight_class(class) {
        return Some(Ok(token));
    }

    match class {
        "debug-outline" => {
            return Some(Ok(quote! {
                .map(|__el| {
                    #[cfg(debug_assertions)]
                    {
                        __el.debug()
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        __el
                    }
                })
            }));
        }
        "flex-grow" => return Some(Ok(quote! { .flex_grow_1() })),
        "flex-shrink" => return Some(Ok(quote! { .flex_shrink_1() })),
        "border" => return Some(Ok(quote! { .border_1() })),
        "no-underline" => return Some(Ok(quote! { .text_decoration_none() })),
        "group" => return Some(Ok(quote! { .group("") })),
        _ => {}
    }
    if let Some(name) = class.strip_prefix("group/") {
        return Some(Ok(quote! { .group(#name) }));
    }

    if let Some(result) = parse_arbitrary_length_class(class) {
        return Some(result);
    }
    if let Some(result) = parse_fraction_length_class(class) {
        return Some(result);
    }
    if let Some(token) = parse_numeric_length_class(class, mode.scale) {
        return Some(Ok(token));
    }
    if let Some(result) = parse_direct_color_class(class) {
        return Some(result);
    }
    if let Some(result) = parse_direct_opacity_class(class) {
        return Some(result);
    }
    if let Some(token) = parse_direct_line_clamp_class(class) {
        return Some(Ok(token));
    }
    if let Some(token) = parse_direct_directional_border_class(class) {
        return Some(Ok(token));
    }

    // text-xl → .text_xl()（GPUI 生成表中存在对应方法时）
    if let Some(size) = class.strip_prefix("text-")
        && is_valid_text_size(size)
    {
        let ident = syn::Ident::new(&format!("text_{size}"), Span::call_site());
        return Some(Ok(quote! { .#ident() }));
    }

    // 任意 GPUI 无参样式方法：flex-col → .flex_col()、rounded-t-md → .rounded_t_md()
    if let Some(method) = gpui_no_arg_class_method(class) {
        let ident = syn::Ident::new(method.name, Span::call_site());
        return Some(Ok(quote! { .#ident() }));
    }

    None
}

fn parse_font_weight_class(class: &str) -> Option<TokenStream> {
    let (_, weight) = FONT_WEIGHTS.iter().find(|(name, _)| *name == class)?;
    let weight_ident = syn::Ident::new(weight, Span::call_site());
    Some(quote! { .font_weight(FontWeight::#weight_ident) })
}

#[derive(Clone, Copy)]
pub enum LengthKind {
    Px(f32),
    Rem(f32),
    Relative(f32),
}

impl LengthKind {
    pub fn parse_arbitrary(class: &str, value: &str) -> Result<Self, String> {
        if let Some(raw) = value.strip_suffix("px") {
            return parse_length_number(raw)
                .map(Self::Px)
                .ok_or_else(|| invalid_length_value_message(class));
        }
        if let Some(raw) = value.strip_suffix("rem") {
            return parse_length_number(raw)
                .map(Self::Rem)
                .ok_or_else(|| invalid_length_value_message(class));
        }
        if let Some(raw) = value.strip_suffix('%') {
            return parse_length_number(raw)
                .map(|n| Self::Relative(n / 100.0))
                .ok_or_else(|| invalid_length_value_message(class));
        }

        Err(invalid_length_value_message(class))
    }

    pub fn parse_fraction(class: &str, value: &str) -> Result<Self, String> {
        let (numerator, denominator) = value.split_once('/').ok_or_else(|| {
            format!("Invalid fraction `{class}`: expected numerator/denominator.")
        })?;

        let numerator = parse_length_number(numerator)
            .ok_or_else(|| format!("Invalid fraction `{class}`: numerator must be a number."))?;
        let denominator = parse_length_number(denominator)
            .ok_or_else(|| format!("Invalid fraction `{class}`: denominator must be a number."))?;

        if denominator <= 0.0 {
            return Err(format!(
                "Invalid fraction `{class}`: denominator must be greater than 0."
            ));
        }

        Ok(Self::Relative(numerator / denominator))
    }
}

fn parse_arbitrary_length_class(class: &str) -> Option<Result<TokenStream, String>> {
    let (method, value, family) = split_length_class(class)?;
    if !value.starts_with('[') {
        return None;
    }

    let Some(inner) = value.strip_prefix('[').and_then(|s| s.strip_suffix(']')) else {
        return Some(Err(invalid_length_value_message(class)));
    };

    let length = match LengthKind::parse_arbitrary(class, inner) {
        Ok(length) => length,
        Err(msg) => return Some(Err(msg)),
    };

    if matches!(length, LengthKind::Relative(_)) && !family.allows_percent() {
        return Some(Err(format!(
            "Invalid spacing class `{class}`. Percentage values are only supported for sizing \
             classes such as `w-*` and `h-*`."
        )));
    }

    Some(Ok(length_method_call(method, length)))
}

fn parse_fraction_length_class(class: &str) -> Option<Result<TokenStream, String>> {
    let (method, value, family) = split_length_class(class)?;
    if !value.contains('/') {
        return None;
    }

    if !family.allows_fraction() {
        return Some(Err(format!(
            "Invalid fraction `{class}`: fractions are only supported for sizing classes."
        )));
    }

    Some(LengthKind::parse_fraction(class, value).map(|length| length_method_call(method, length)))
}

fn parse_numeric_length_class(class: &str, scale: SpacingScale) -> Option<TokenStream> {
    let (method, value, _) = split_length_class(class)?;
    if value.starts_with('[') || value.contains('/') {
        return None;
    }
    parse_length_number(value)
        .map(|value| length_method_call(method, LengthKind::Px(value * scale.factor())))
}

fn parse_direct_color_class(class: &str) -> Option<Result<TokenStream, String>> {
    if let Some(rest) = class.strip_prefix("text-") {
        return parse_color_with_method(rest, "text_color", class);
    }
    if let Some(rest) = class.strip_prefix("bg-") {
        return parse_color_with_method(rest, "bg", class);
    }
    if let Some(rest) = class.strip_prefix("border-")
        && !is_directional_border(rest)
    {
        return parse_color_with_method(rest, "border_color", class);
    }
    None
}

fn parse_direct_opacity_class(class: &str) -> Option<Result<TokenStream, String>> {
    let rest = class.strip_prefix("opacity-")?;
    let n = match rest.parse::<u8>() {
        Ok(n) if n <= 100 => n,
        _ => return Some(Err(invalid_opacity_value_message(class))),
    };
    let val = n as f32 / 100.0;
    Some(Ok(quote! { .opacity(#val) }))
}

fn parse_direct_line_clamp_class(class: &str) -> Option<TokenStream> {
    let rest = class.strip_prefix("line-clamp-")?;
    let lines = rest.parse::<usize>().ok()?;
    Some(quote! { .line_clamp(#lines) })
}

fn parse_direct_directional_border_class(class: &str) -> Option<TokenStream> {
    let method = match class {
        "border-t" => "border_t_1",
        "border-b" => "border_b_1",
        "border-l" => "border_l_1",
        "border-r" => "border_r_1",
        "border-x" => "border_x_1",
        "border-y" => "border_y_1",
        _ => return None,
    };
    let ident = syn::Ident::new(method, Span::call_site());
    Some(quote! { .#ident() })
}

pub fn parse_length_number(raw: &str) -> Option<f32> {
    if raw.is_empty() {
        return None;
    }
    let value = raw.parse::<f32>().ok()?;
    value.is_finite().then_some(value)
}

pub fn invalid_length_value_message(class: &str) -> String {
    format!(
        "Invalid length class `{class}`. Expected a numeric value with px, rem, or %, \
         for example `w-[280px]`."
    )
}

fn length_method_call(method: &str, length: LengthKind) -> TokenStream {
    let method_ident = syn::Ident::new(method, Span::call_site());
    match length {
        LengthKind::Px(value) => quote! { .#method_ident(px(#value)) },
        LengthKind::Rem(value) => quote! { .#method_ident(rems(#value)) },
        LengthKind::Relative(value) => quote! { .#method_ident(relative(#value)) },
    }
}

/// 编译错误，定位到 class 字面量。
pub(crate) fn compile_error_at(message: &str, span: Span) -> TokenStream {
    let error = quote_spanned! {span=> compile_error!(#message); };
    quote! { .map(|__el| { #error __el }) }
}

/// 编译期警告：stable 上借用 `#[deprecated]` lint，警告定位到 class 字面量。
pub(crate) fn warning(message: &str, span: Span) -> TokenStream {
    let usage = quote_spanned! {span=> unsupported_rsx_class };
    quote! {
        .map(|__el| {
            #[deprecated(note = #message)]
            #[allow(non_camel_case_types)]
            struct unsupported_rsx_class;
            let _ = #usage;
            __el
        })
    }
}

/// 统一的颜色解析函数（核心去重逻辑）
///
/// 将颜色名称或任意 hex / rgb() / rgba() 值转换为方法调用。
fn parse_color_with_method(
    color: &str,
    method: &str,
    class: &str,
) -> Option<Result<TokenStream, String>> {
    let parsed = lookup_color_key(color)
        .map(ColorValue::Rgb)
        .or_else(|| parse_arbitrary_color_value(color));
    let Some(parsed) = parsed else {
        if color.starts_with('[') {
            return Some(Err(invalid_color_value_message(class)));
        }
        return None;
    };
    let ident = syn::Ident::new(method, Span::call_site());
    Some(Ok(match parsed {
        ColorValue::Rgb(hex) => quote! { .#ident(rgb(#hex)) },
        ColorValue::Rgba(hex) => quote! { .#ident(rgba(#hex)) },
    }))
}

fn lookup_color_key(color: &str) -> Option<u32> {
    if let Some(hex) = lookup_color(color) {
        return Some(hex);
    }
    if color.contains('-') {
        let normalized = color.replace('-', "_");
        lookup_color(&normalized)
    } else {
        None
    }
}

/// 判断 `border-` 之后的部分是否属于方向性边框类（而非颜色类）
///
/// - 方向性：`border-t`（rest = `"t"`）、`border-t-2`（rest = `"t-2"`）
/// - 颜色：`border-red-500`（首字节 `r` 在方向集合中，但第二字节不是 `-`）
fn is_directional_border(rest: &str) -> bool {
    let bytes = rest.as_bytes();
    matches!(bytes.first(), Some(b't' | b'b' | b'l' | b'r' | b'x' | b'y'))
        && (rest.len() == 1 || matches!(bytes.get(1), Some(b'-' | b'_')))
}

#[derive(Clone, Copy)]
enum ColorValue {
    Rgb(u32),
    Rgba(u32),
}

fn parse_arbitrary_color_value(s: &str) -> Option<ColorValue> {
    parse_arbitrary_hex(s).or_else(|| parse_arbitrary_color_function(s))
}

fn parse_arbitrary_hex(s: &str) -> Option<ColorValue> {
    // 匹配 [#rrggbb]、[#rrggbbaa]、[#rgb] 或 [#rgba]
    let inner = s.strip_prefix("[#")?.strip_suffix(']')?;
    if !inner.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let d = |c: u8| -> u32 { (c as char).to_digit(16).unwrap_or(0) };
    let expand = |n: u32| (n << 4) | n;
    let b = inner.as_bytes();
    match inner.len() {
        8 => u32::from_str_radix(inner, 16).ok().map(ColorValue::Rgba),
        6 => u32::from_str_radix(inner, 16).ok().map(ColorValue::Rgb),
        4 => Some(ColorValue::Rgba(
            expand(d(b[0])) << 24 | expand(d(b[1])) << 16 | expand(d(b[2])) << 8 | expand(d(b[3])),
        )),
        3 => Some(ColorValue::Rgb(
            expand(d(b[0])) << 16 | expand(d(b[1])) << 8 | expand(d(b[2])),
        )),
        _ => None,
    }
}

fn parse_arbitrary_color_function(s: &str) -> Option<ColorValue> {
    let inner = s.strip_prefix('[')?.strip_suffix(']')?;

    if let Some(args) = inner.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        let [r, g, b] = parse_rgb_args(args)?;
        return Some(ColorValue::Rgb(
            ((r as u32) << 16) | ((g as u32) << 8) | b as u32,
        ));
    }

    if let Some(args) = inner
        .strip_prefix("rgba(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let mut parts = args.split(',');
        let r = parse_u8_component(parts.next()?)?;
        let g = parse_u8_component(parts.next()?)?;
        let b = parse_u8_component(parts.next()?)?;
        let a = parse_alpha_component(parts.next()?)?;
        if parts.next().is_some() {
            return None;
        }
        return Some(ColorValue::Rgba(
            ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a as u32,
        ));
    }

    None
}

fn parse_rgb_args(args: &str) -> Option<[u8; 3]> {
    let mut parts = args.split(',');
    let r = parse_u8_component(parts.next()?)?;
    let g = parse_u8_component(parts.next()?)?;
    let b = parse_u8_component(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some([r, g, b])
}

fn parse_u8_component(raw: &str) -> Option<u8> {
    raw.trim().parse::<u8>().ok()
}

fn parse_alpha_component(raw: &str) -> Option<u8> {
    let value = raw.trim().parse::<f32>().ok()?;
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return None;
    }
    Some((value * 255.0).round() as u8)
}

fn invalid_color_value_message(class: &str) -> String {
    format!(
        "Invalid color class `{class}`. Expected #rgb, #rrggbb, #rrggbbaa, rgb(r,g,b), or rgba(r,g,b,a)."
    )
}

fn invalid_opacity_value_message(class: &str) -> String {
    format!("Invalid opacity class `{class}`. Expected `opacity-0` through `opacity-100`.")
}

fn unsupported_class_message(class: &str, suggestion: Option<&str>) -> String {
    let hint = suggestion
        .map(|s| format!(" Did you mean `{s}`?"))
        .unwrap_or_default();
    format!(
        "Unsupported class `{class}` in strict mode.{hint}\n\
         \x20 help: use `rsx!` or `rsx_permissive!` to pass unknown classes through, or replace \
         this with a supported GPUI class or attribute"
    )
}

fn passthrough_message(class: &str, suggestion: Option<&str>) -> String {
    let method = class.replace(['-', '/'], "_");
    let hint = suggestion
        .map(|s| format!(" Did you mean `{s}`?"))
        .unwrap_or_default();
    format!(
        "`{class}` is not a known GPUI class and is passed through as `.{method}()`.{hint}\n \
         help: for an extension method, use the explicit flag attribute `{method}` or `base={{...}}`; otherwise use a supported class"
    )
}

// ---------------------------------------------------------------------------
// Vocabulary and suggestions
// ---------------------------------------------------------------------------

/// 所有已知的“具体” class 名（不含数值/任意值模式），用于拼写建议、补全与规范包。
pub fn known_classes() -> &'static [String] {
    static CLASSES: OnceLock<Vec<String>> = OnceLock::new();
    CLASSES.get_or_init(|| {
        let mut out: Vec<String> = SEMANTIC_NO_ARG_CLASSES
            .iter()
            .map(|c| (*c).to_owned())
            .collect();
        for (name, _) in FONT_WEIGHTS {
            out.push((*name).to_owned());
        }
        for method in GPUI_METHODS {
            if !method.family_prefix.is_empty()
                && (method.negated
                    || !(method.family_prefix.starts_with("rounded")
                        || method.family_prefix.starts_with("border")
                        || matches!(method.family_suffix, "full" | "auto" | "px")))
            {
                continue;
            }
            let class = method.name.replace('_', "-");
            if gpui_no_arg_class_method(&class).is_some() && !out.contains(&class) {
                out.push(class);
            }
        }
        for prefix in ["text", "bg", "border"] {
            for family in COLOR_FAMILIES {
                for shade in COLOR_SHADES {
                    out.push(format!("{prefix}-{family}-{shade}"));
                }
            }
            out.push(format!("{prefix}-black"));
            out.push(format!("{prefix}-white"));
        }
        out.sort();
        out.dedup();
        out
    })
}

/// 拼写建议：编辑距离最近的已知 class（含长度前缀、状态前缀与主题令牌）。
pub fn suggest_class(class: &str, mode: ClassMode) -> Option<String> {
    if let Some((prefix, rest)) = class.split_once(':') {
        if parse_variant(prefix).is_err() {
            let variant = suggest(prefix, SUPPORTED_VARIANTS.iter().copied())?;
            return Some(format!("{variant}:{rest}"));
        }
        return suggest_class(rest, mode).map(|s| format!("{prefix}:{s}"));
    }

    // gpa-4 → gap-4：数值类只比较前缀
    if let Some((head, tail)) = class.rsplit_once('-')
        && (parse_length_number(tail).is_some() || tail.starts_with('['))
        && split_length_class(class).is_none()
    {
        let head = format!("{head}-");
        let prefixes = LENGTH_CLASS_SPECS.iter().map(|spec| spec.prefix);
        if let Some(prefix) = suggest(&head, prefixes) {
            return Some(format!("{prefix}{tail}"));
        }
    }

    let theme_classes = mode.theme.map(Theme::class_names).unwrap_or_default();
    let candidates = known_classes()
        .iter()
        .map(String::as_str)
        .chain(theme_classes.iter().map(String::as_str));
    suggest(class, candidates)
}

fn suggest<'a>(input: &str, candidates: impl Iterator<Item = &'a str>) -> Option<String> {
    let max = match input.len() {
        0..=4 => 1,
        5..=10 => 2,
        _ => 3,
    };
    let mut best: Option<(usize, &str)> = None;
    for candidate in candidates {
        if candidate == input || candidate.len().abs_diff(input.len()) > max {
            continue;
        }
        let limit = best.map_or(max, |(d, _)| d.saturating_sub(1));
        let distance = bounded_distance(input, candidate, limit);
        if distance <= limit && best.is_none_or(|(d, _)| distance < d) {
            best = Some((distance, candidate));
        }
    }
    best.map(|(_, c)| c.to_owned())
}

/// Optimal-string-alignment distance (Levenshtein plus adjacent transpositions, so
/// `gpa` → `gap` is 1). Returns `limit + 1` as soon as the distance must exceed `limit`.
fn bounded_distance(a: &str, b: &str, limit: usize) -> usize {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let width = b.len() + 1;
    let mut d = vec![0usize; (a.len() + 1) * width];
    for i in 0..=a.len() {
        d[i * width] = i;
    }
    for (j, cell) in d.iter_mut().enumerate().take(width) {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut best = (d[(i - 1) * width + j] + 1)
                .min(d[i * width + j - 1] + 1)
                .min(d[(i - 1) * width + j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(d[(i - 2) * width + j - 2] + 1);
            }
            d[i * width + j] = best;
        }
        // Every later cell is at least the minimum of the two latest rows.
        let row_min = d[i * width..(i + 1) * width]
            .iter()
            .min()
            .copied()
            .unwrap_or(0);
        let prev_min = d[(i - 1) * width..i * width]
            .iter()
            .min()
            .copied()
            .unwrap_or(0);
        if row_min.min(prev_min) > limit {
            return limit.saturating_add(1);
        }
    }
    d[a.len() * width + b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expand(class: &str, mode: ClassMode) -> String {
        parse_class_string_with_mode(class, mode, Span::call_site())
            .into_iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn malformed_arbitrary_length_reports_compile_error() {
        let tokens = expand("w-[280px", ClassMode::PERMISSIVE);

        assert!(tokens.contains("Invalid length class `w-[280px`"));
        assert!(tokens.contains("compile_error"));
    }

    #[test]
    fn fraction_parser_reports_each_malformed_component() {
        let cases = [
            (
                "12",
                "Invalid fraction `w-12`: expected numerator/denominator.",
            ),
            (
                "many/2",
                "Invalid fraction `w-many/2`: numerator must be a number.",
            ),
            (
                "1/many",
                "Invalid fraction `w-1/many`: denominator must be a number.",
            ),
        ];

        for (value, expected) in cases {
            let class = format!("w-{value}");
            assert_eq!(
                LengthKind::parse_fraction(&class, value).err().as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn fraction_spacing_reports_unsupported_family() {
        let tokens = expand("p-1/2", ClassMode::PERMISSIVE);

        assert!(tokens.contains("fractions are only supported for sizing classes"));
        assert!(tokens.contains("compile_error"));
    }

    #[test]
    fn length_number_rejects_empty_and_non_finite_values() {
        assert_eq!(parse_length_number(""), None);
        assert_eq!(parse_length_number("NaN"), None);
        assert_eq!(parse_length_number("inf"), None);
        assert_eq!(parse_length_number("-inf"), None);
    }

    #[test]
    fn variants_group_into_style_refinement_closures() {
        let tokens = expand(
            "bg-blue-500 hover:bg-blue-600 hover:text-white active:opacity-75 group-hover/menu:underline",
            ClassMode::PERMISSIVE,
        );
        assert!(tokens.contains(
            ". hover (| __style | __style . bg (rgb (2450411u32)) . text_color (rgb (16777215u32)))"
        ));
        assert!(tokens.contains(". active (| __style | __style . opacity (0.75f32))"));
        assert!(tokens.contains(". group_hover (\"menu\" , | __style | __style . underline ())"));
    }

    #[test]
    fn unsupported_variants_warn_in_permissive_and_fail_in_strict() {
        let permissive = expand("dark:bg-black md:flex", ClassMode::PERMISSIVE);
        assert!(permissive.contains("deprecated"));
        assert!(permissive.contains("color-scheme"));
        assert!(permissive.contains("breakpoints"));
        assert!(!permissive.contains("compile_error"));

        let strict = expand("dark:bg-black", ClassMode::STRICT);
        assert!(strict.contains("compile_error"));
    }

    #[test]
    fn variants_reject_element_level_classes() {
        let tokens = expand("hover:overflow-scroll", ClassMode::PERMISSIVE);
        assert!(tokens.contains("cannot be used inside the `hover:` variant"));
    }

    #[test]
    fn tailwind_scale_multiplies_numeric_lengths_only() {
        let mode = ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind);
        assert_eq!(expand("gap-4", mode), ". gap (px (16f32))");
        assert_eq!(expand("p-2.5", mode), ". p (px (10f32))");
        assert_eq!(expand("w-64", mode), ". w (px (256f32))");
        assert_eq!(expand("gap-[14px]", mode), ". gap (px (14f32))");
        assert_eq!(expand("gap-4", ClassMode::PERMISSIVE), ". gap (px (4f32))");
    }

    #[test]
    fn suggestions_cover_typos_prefixes_and_variants() {
        let mode = ClassMode::PERMISSIVE;
        assert_eq!(
            suggest_class("flex-colum", mode).as_deref(),
            Some("flex-col")
        );
        assert_eq!(
            suggest_class("bg-blu-500", mode).as_deref(),
            Some("bg-blue-500")
        );
        assert_eq!(suggest_class("gpa-4", mode).as_deref(), Some("gap-4"));
        assert_eq!(
            suggest_class("hovr:flex", mode).as_deref(),
            Some("hover:flex")
        );
        assert_eq!(suggest_class("zzzzzzzz", mode), None);
    }

    #[test]
    fn unknown_identifier_classes_pass_through_with_typo_warning() {
        let tokens = expand("flex-colum", ClassMode::PERMISSIVE);
        assert!(tokens.contains(". flex_colum ()"));
        assert!(tokens.contains("Did you mean `flex-col`?"));

        // Extensions still work, but unknown vocabulary is never silently accepted.
        let tokens = expand("h-flex", ClassMode::PERMISSIVE);
        assert!(tokens.contains(". h_flex ()"));
        assert!(tokens.contains("explicit flag attribute `h_flex`"));
    }

    #[test]
    fn digit_leading_classes_never_become_identifiers() {
        let tokens = expand("2xl 3d:flex", ClassMode::PERMISSIVE);
        assert!(tokens.contains("ignored unsupported class `2xl`"));
    }

    #[test]
    fn diagnostics_report_offsets() {
        let diagnostics =
            class_diagnostics("flex  dark:bg-black flex-colum", ClassMode::PERMISSIVE);
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].class, "dark:bg-black");
        assert_eq!(diagnostics[0].offset, 6);
        assert_eq!(diagnostics[1].suggestion.as_deref(), Some("flex-col"));
    }
}
