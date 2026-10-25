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

mod suggestions;
mod values;
mod variants;

pub use suggestions::{known_classes, suggest_class};
pub use values::{LengthKind, color_rgba, invalid_length_value_message, parse_length_number};
pub use variants::{SUPPORTED_VARIANTS, Variant};

use values::expand_known_class;
use variants::parse_variant;

#[cfg(test)]
mod tests;
