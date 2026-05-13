//! Class 字符串解析
//!
//! 将 CSS class 字符串解析为 GPUI 方法调用链，支持：
//! - Tailwind 风格的实用类（flex, gap-4, text-red-500）
//! - 任意 hex 颜色值（bg-[#ff0000]）
//! - 间距/尺寸类
//!
//! 核心优化：
//! - 统一的颜色解析函数，避免代码重复
//! - 间距前缀使用 rfind + match（O(1)）替代线性扫描（O(17)）
//! - text_ 前缀只做一次 strip_prefix，颜色与文本大小分支合并处理
//! - 文本大小使用 match 替代 contains 线性查找
//! - 先检查 `contains('-')` 跳过无连字符类的堆分配，含连字符时用 `replace` 做完整替换
//! - split_ascii_whitespace 替代 split_whitespace（class 名只含 ASCII）

use super::tables::*;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClassMode {
    Permissive,
    Strict,
}

impl ClassMode {
    fn is_strict(self) -> bool {
        matches!(self, Self::Strict)
    }
}

/// 解析 class 字符串为方法链片段迭代器
///
/// `"flex flex-col gap-4"` → `[.flex(), .flex_col(), .gap(px(4.0))]`
///
/// 返回迭代器而非 Vec，调用方通过 `extend` 消费时避免中间 Vec 分配。
pub(crate) fn parse_class_string_with_mode(
    class_str: &str,
    mode: ClassMode,
) -> impl Iterator<Item = TokenStream> + '_ {
    class_str
        .split_ascii_whitespace()
        .map(move |class| parse_single_class_with_mode(class, mode))
}

/// 解析单个 CSS class 为方法调用
pub(crate) fn parse_single_class_with_mode(class: &str, mode: ClassMode) -> TokenStream {
    if let Some(token) = parse_font_weight_class(class, mode) {
        return token;
    }

    if class == "debug-outline" {
        return quote! {
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
        };
    }

    if let Some(token) = parse_gpui_0_2_compat_class(class) {
        return token;
    }

    if let Some(token) = parse_arbitrary_length_class(class) {
        return token;
    }

    if let Some(token) = parse_fraction_length_class(class) {
        return token;
    }

    // 含 '-' 或 '/' 时分配新 String，不含则零拷贝借用原字符串。
    // Tailwind fraction classes like `w-1/2` map to GPUI helpers like `w_1_2`.
    let method_name: Cow<str> = if class.contains(['-', '/']) {
        Cow::Owned(class.replace(['-', '/'], "_"))
    } else {
        Cow::Borrowed(class)
    };

    // 间距/尺寸类：使用 rfind('_') + match 实现 O(1) 前缀查找
    if let Some(underscore_pos) = method_name.rfind('_') {
        let suffix = &method_name[underscore_pos + 1..];
        if let Some(num) = parse_length_number(suffix) {
            let prefix = &method_name[..=underscore_pos];
            if let Some(method) = lookup_spacing_method(prefix) {
                let method_ident = syn::Ident::new(method, Span::call_site());
                return quote! { .#method_ident(px(#num)) };
            }
        }
    }

    // GPUI 0.2 的方向性 border 无参宽度方法带 `_1` 后缀。
    if let Some(method) = lookup_directional_border_method(&method_name) {
        let ident = syn::Ident::new(method, Span::call_site());
        return quote! { .#ident() };
    }

    // border 特殊处理：
    // "border" (纯) → .border_1()（GPUI 没有无参 .border()）
    // "border-2" → .border_2()
    if method_name == "border" {
        return quote! { .border_1() };
    }

    if method_name == "no_underline" {
        return quote! { .text_decoration_none() };
    }

    // border-color 类：border-red-500 → .border_color(rgb(0xef4444))
    if let Some(rest) = method_name.strip_prefix("border_")
        && !is_directional_border(rest)
    {
        if rest.as_bytes().first().is_some_and(|b| b.is_ascii_digit()) {
            // 数值边框宽度 border-2, border-4 等
            let ident = syn::Ident::new(&method_name, Span::call_site());
            return quote! { .#ident() };
        } else if let Some(token) = parse_color_with_method(rest, "border_color", class) {
            return token;
        }
    }

    // text_ 前缀：统一处理颜色类（text-red-600）和文本大小类（text-xl）
    // 只做一次 strip_prefix("text_")，避免先在颜色分支、再在大小分支各做一次。
    if let Some(rest) = method_name.strip_prefix("text_") {
        // 先查颜色表（text-red-500 → .text_color(rgb(...))）
        if let Some(token) = parse_color_with_method(rest, "text_color", class) {
            return token;
        }
        // 再查文本大小（text-xl → .text_xl()）
        if is_valid_text_size(rest) {
            let size_ident = syn::Ident::new(&method_name, Span::call_site());
            return quote! { .#size_ident() };
        }
        // 不在白名单中的 text_ 前缀，fall through 到默认处理
    }

    // bg_ 颜色类：bg-blue-500 → .bg(rgb(...))
    if let Some(rest) = method_name.strip_prefix("bg_")
        && let Some(token) = parse_color_with_method(rest, "bg", class)
    {
        return token;
    }

    // opacity_ 类：opacity-50 → .opacity(0.5)
    // 注意：GPUI 的 opacity 范围为 0.0–1.0，Tailwind 用 0–100 整数表示
    if let Some(rest) = method_name.strip_prefix("opacity_")
        && let Ok(n) = rest.parse::<u8>()
    {
        let val = n as f32 / 100.0;
        return quote! { .opacity(#val) };
    }

    // line-clamp-3 → .line_clamp(3usize)
    if let Some(rest) = method_name.strip_prefix("line_clamp_")
        && let Ok(lines) = rest.parse::<usize>()
    {
        return quote! { .line_clamp(#lines) };
    }

    if mode.is_strict() && !is_supported_no_arg_class(class) {
        return compile_error(unsupported_class_message(class));
    }

    // 默认：无参方法调用（防御性检查：跳过含非标识符字符的 class，如 "hover:bg-blue-500"）
    if !method_name.is_empty()
        && method_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        let ident = syn::Ident::new(&method_name, Span::call_site());
        quote! { .#ident() }
    } else {
        if mode.is_strict() {
            compile_error(unsupported_class_message(class))
        } else {
            quote! {}
        }
    }
}

fn parse_gpui_0_2_compat_class(class: &str) -> Option<TokenStream> {
    match class {
        "content-stretch" => Some(quote! {
            .map(|mut __el| {
                __el.style().align_content = Some(AlignContent::Stretch);
                __el
            })
        }),
        "flex-grow-0" => Some(quote! {
            .map(|mut __el| {
                __el.style().flex_grow = Some(0.0);
                __el
            })
        }),
        "items-stretch" => Some(quote! {
            .map(|mut __el| {
                __el.style().align_items = Some(AlignItems::Stretch);
                __el
            })
        }),
        "justify-evenly" => Some(quote! {
            .map(|mut __el| {
                __el.style().justify_content = Some(JustifyContent::SpaceEvenly);
                __el
            })
        }),
        "self-start" | "self-flex-start" => Some(quote! {
            .map(|mut __el| {
                __el.style().align_self = Some(AlignItems::FlexStart);
                __el
            })
        }),
        "self-end" | "self-flex-end" => Some(quote! {
            .map(|mut __el| {
                __el.style().align_self = Some(AlignItems::FlexEnd);
                __el
            })
        }),
        "self-center" => Some(quote! {
            .map(|mut __el| {
                __el.style().align_self = Some(AlignItems::Center);
                __el
            })
        }),
        "self-baseline" => Some(quote! {
            .map(|mut __el| {
                __el.style().align_self = Some(AlignItems::Baseline);
                __el
            })
        }),
        "self-stretch" => Some(quote! {
            .map(|mut __el| {
                __el.style().align_self = Some(AlignItems::Stretch);
                __el
            })
        }),
        _ => None,
    }
}

fn parse_font_weight_class(class: &str, mode: ClassMode) -> Option<TokenStream> {
    let weight = match class {
        "font-thin" => "THIN",
        "font-extralight" => "EXTRA_LIGHT",
        "font-light" => "LIGHT",
        "font-normal" => "NORMAL",
        "font-medium" => "MEDIUM",
        "font-semibold" => "SEMIBOLD",
        "font-bold" => "BOLD",
        "font-extrabold" => "EXTRA_BOLD",
        "font-black" => "BLACK",
        _ => {
            if mode.is_strict() && class.starts_with("font-") {
                let msg = format!(
                    "Unsupported font weight class `{class}`. Supported font weight classes: \
                     font-thin, font-extralight, font-light, font-normal, font-medium, \
                     font-semibold, font-bold, font-extrabold, font-black."
                );
                return Some(compile_error(msg));
            }
            return None;
        }
    };
    let weight_ident = syn::Ident::new(weight, Span::call_site());
    Some(quote! { .font_weight(FontWeight::#weight_ident) })
}

#[derive(Clone, Copy)]
enum LengthKind {
    Px(f32),
    Rem(f32),
    Relative(f32),
}

fn parse_arbitrary_length_class(class: &str) -> Option<TokenStream> {
    let (method, value, family) = split_length_class(class)?;
    if !value.starts_with('[') {
        return None;
    }

    let Some(inner) = value.strip_prefix('[').and_then(|s| s.strip_suffix(']')) else {
        return Some(compile_error(format!(
            "Invalid length class `{class}`. Expected a numeric value with px, rem, or %, \
             for example `w-[280px]`."
        )));
    };

    let length = match parse_arbitrary_length_value(class, inner) {
        Ok(length) => length,
        Err(msg) => return Some(compile_error(msg)),
    };

    if matches!(length, LengthKind::Relative(_)) && !family.allows_percent {
        return Some(compile_error(format!(
            "Invalid spacing class `{class}`. Percentage values are only supported for sizing \
             classes such as `w-*` and `h-*`."
        )));
    }

    Some(length_method_call(method, length))
}

fn parse_fraction_length_class(class: &str) -> Option<TokenStream> {
    let (method, value, family) = split_length_class(class)?;
    if !value.contains('/') {
        return None;
    }

    if !family.allows_fraction {
        return Some(compile_error(format!(
            "Invalid fraction `{class}`: fractions are only supported for sizing classes."
        )));
    }

    let (numerator, denominator) = value.split_once('/')?;

    let numerator = match parse_length_number(numerator) {
        Some(n) => n,
        None => {
            return Some(compile_error(format!(
                "Invalid fraction `{class}`: numerator must be a number."
            )));
        }
    };
    let denominator = match parse_length_number(denominator) {
        Some(n) => n,
        None => {
            return Some(compile_error(format!(
                "Invalid fraction `{class}`: denominator must be a number."
            )));
        }
    };

    if denominator <= 0.0 {
        return Some(compile_error(format!(
            "Invalid fraction `{class}`: denominator must be greater than 0."
        )));
    }

    Some(length_method_call(
        method,
        LengthKind::Relative(numerator / denominator),
    ))
}

#[derive(Clone, Copy)]
struct LengthFamily {
    allows_percent: bool,
    allows_fraction: bool,
}

fn split_length_class(class: &str) -> Option<(&'static str, &str, LengthFamily)> {
    const SIZING: LengthFamily = LengthFamily {
        allows_percent: true,
        allows_fraction: true,
    };
    const SPACING: LengthFamily = LengthFamily {
        allows_percent: false,
        allows_fraction: false,
    };

    for (prefix, method, family) in [
        ("min-w-", "min_w", SIZING),
        ("max-w-", "max_w", SIZING),
        ("min-h-", "min_h", SIZING),
        ("max-h-", "max_h", SIZING),
        ("gap-x-", "gap_x", SPACING),
        ("gap-y-", "gap_y", SPACING),
        ("size-", "size", SIZING),
        ("gap-", "gap", SPACING),
        ("px-", "px", SPACING),
        ("py-", "py", SPACING),
        ("pt-", "pt", SPACING),
        ("pb-", "pb", SPACING),
        ("pl-", "pl", SPACING),
        ("pr-", "pr", SPACING),
        ("mx-", "mx", SPACING),
        ("my-", "my", SPACING),
        ("mt-", "mt", SPACING),
        ("mb-", "mb", SPACING),
        ("ml-", "ml", SPACING),
        ("mr-", "mr", SPACING),
        ("w-", "w", SIZING),
        ("h-", "h", SIZING),
        ("p-", "p", SPACING),
        ("m-", "m", SPACING),
    ] {
        if let Some(value) = class.strip_prefix(prefix) {
            return Some((method, value, family));
        }
    }

    None
}

fn parse_arbitrary_length_value(class: &str, value: &str) -> Result<LengthKind, String> {
    if let Some(raw) = value.strip_suffix("px") {
        return parse_length_number(raw)
            .map(LengthKind::Px)
            .ok_or_else(|| invalid_length_value_message(class));
    }
    if let Some(raw) = value.strip_suffix("rem") {
        return parse_length_number(raw)
            .map(LengthKind::Rem)
            .ok_or_else(|| invalid_length_value_message(class));
    }
    if let Some(raw) = value.strip_suffix('%') {
        return parse_length_number(raw)
            .map(|n| LengthKind::Relative(n / 100.0))
            .ok_or_else(|| invalid_length_value_message(class));
    }

    Err(invalid_length_value_message(class))
}

fn parse_length_number(raw: &str) -> Option<f32> {
    if raw.is_empty() {
        return None;
    }
    let value = raw.parse::<f32>().ok()?;
    value.is_finite().then_some(value)
}

fn invalid_length_value_message(class: &str) -> String {
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

fn compile_error(message: String) -> TokenStream {
    quote! { .map(|__el| { compile_error!(#message); __el }) }
}

/// 统一的颜色解析函数（核心去重逻辑）
///
/// 将颜色名称或任意 hex 值转换为方法调用。
///
/// # 参数
/// - `color`: 颜色字符串（如 "red_500", "[#ff0000]"）
/// - `method`: 方法名（"text_color", "bg", "border_color"）
///
/// # 返回值
/// - `Some(TokenStream)`: 成功解析，返回 `.method(rgb(value))`
/// - `None`: 无法解析颜色
fn parse_color_with_method(color: &str, method: &str, class: &str) -> Option<TokenStream> {
    // 统一颜色表查找和任意 hex 解析，仅在匹配成功时创建 Ident
    let parsed = lookup_color_key(color)
        .map(ColorValue::Rgb)
        .or_else(|| parse_arbitrary_color_value(color));
    let Some(parsed) = parsed else {
        if color.starts_with('[') {
            return Some(compile_error(invalid_color_value_message(class)));
        }
        return None;
    };
    let ident = syn::Ident::new(method, Span::call_site());
    match parsed {
        ColorValue::Rgb(hex) => Some(quote! { .#ident(rgb(#hex)) }),
        ColorValue::Rgba(hex) => Some(quote! { .#ident(rgba(#hex)) }),
    }
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

/// 判断 `border_` 之后的部分是否属于方向性边框类（而非颜色类）
///
/// 方向性边框（fall through 到默认方法调用）：
/// - 纯方向：`border-t` → rest = `"t"`（len == 1）
/// - 方向+数值：`border-t-2` → rest = `"t_2"`（首字节是方向，第二字节是 `_`）
///
/// 颜色类（应生成 `.border_color(rgb(...))`）：
/// - `border-red-500` → rest = `"red_500"`（首字节 `r` 虽在方向集合中，
///   但第二字节 `e` ≠ `_`，故判定为颜色类）
fn is_directional_border(rest: &str) -> bool {
    let bytes = rest.as_bytes();
    matches!(bytes.first(), Some(b't' | b'b' | b'l' | b'r' | b'x' | b'y'))
        && (rest.len() == 1 || bytes.get(1) == Some(&b'_'))
}

fn lookup_directional_border_method(class: &str) -> Option<&'static str> {
    match class {
        "border_t" => Some("border_t_1"),
        "border_b" => Some("border_b_1"),
        "border_l" => Some("border_l_1"),
        "border_r" => Some("border_r_1"),
        "border_x" => Some("border_x_1"),
        "border_y" => Some("border_y_1"),
        _ => None,
    }
}

/// 解析任意 hex 颜色值：`[#rrggbb]` 或 `[#rgb]`
///
/// 输入已经过 `-` → `_` 替换，但 `[#...]` 中不含 `-`，所以保持原样。
/// 返回解析后的 u32 颜色值。
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
    match inner.len() {
        8 => u32::from_str_radix(inner, 16).ok().map(ColorValue::Rgba),
        6 => u32::from_str_radix(inner, 16).ok().map(ColorValue::Rgb),
        4 => {
            let b = inner.as_bytes();
            let d = |c: u8| -> Option<u32> { (c as char).to_digit(16) };
            let r = d(b[0])?;
            let g = d(b[1])?;
            let bl = d(b[2])?;
            let a = d(b[3])?;
            let expand = |n: u32| (n << 4) | n;
            Some(ColorValue::Rgba(
                expand(r) << 24 | expand(g) << 16 | expand(bl) << 8 | expand(a),
            ))
        }
        3 => {
            // 3 位 hex 扩展为 6 位: #abc → #aabbcc
            // 用位运算零分配实现，避免 String 堆分配
            let b = inner.as_bytes();
            let d = |c: u8| -> Option<u32> { (c as char).to_digit(16) };
            let r = d(b[0])?;
            let g = d(b[1])?;
            let bl = d(b[2])?;
            // 每个 4-bit 数字复制到高低 nibble: 0xA → 0xAA
            Some(ColorValue::Rgb(
                r << 20 | r << 16 | g << 12 | g << 8 | bl << 4 | bl,
            ))
        }
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

fn unsupported_class_message(class: &str) -> String {
    format!(
        "Unsupported class `{class}` in strict mode. Use `rsx!` or `rsx_permissive!` to keep \
         unsupported classes ignored, or replace this with a supported GPUI class or attribute."
    )
}

fn is_supported_no_arg_class(class: &str) -> bool {
    matches!(
        class,
        "absolute"
            | "aspect-square"
            | "block"
            | "border"
            | "border-2"
            | "border-b"
            | "border-b-2"
            | "border-dashed"
            | "border-l"
            | "border-l-2"
            | "border-r"
            | "border-r-2"
            | "border-t"
            | "border-t-2"
            | "border-x"
            | "border-x-2"
            | "border-y"
            | "border-y-2"
            | "col-end-auto"
            | "col-span-full"
            | "col-start-auto"
            | "content-around"
            | "content-between"
            | "content-center"
            | "content-end"
            | "content-evenly"
            | "content-normal"
            | "content-start"
            | "content-stretch"
            | "cursor-alias"
            | "cursor-col-resize"
            | "cursor-context-menu"
            | "cursor-copy"
            | "cursor-crosshair"
            | "cursor-default"
            | "cursor-e-resize"
            | "cursor-ew-resize"
            | "cursor-grab"
            | "cursor-grabbing"
            | "cursor-move"
            | "cursor-n-resize"
            | "cursor-nesw-resize"
            | "cursor-no-drop"
            | "cursor-not-allowed"
            | "cursor-ns-resize"
            | "cursor-nwse-resize"
            | "cursor-pointer"
            | "cursor-row-resize"
            | "cursor-s-resize"
            | "cursor-text"
            | "cursor-vertical-text"
            | "cursor-w-resize"
            | "debug-outline"
            | "flex"
            | "flex-1"
            | "flex-auto"
            | "flex-col"
            | "flex-col-reverse"
            | "flex-grow"
            | "flex-grow-0"
            | "flex-initial"
            | "flex-none"
            | "flex-nowrap"
            | "flex-row"
            | "flex-row-reverse"
            | "flex-shrink"
            | "flex-shrink-0"
            | "flex-wrap"
            | "flex-wrap-reverse"
            | "grid"
            | "h-auto"
            | "h-full"
            | "h-px"
            | "hidden"
            | "italic"
            | "items-baseline"
            | "items-center"
            | "items-end"
            | "items-start"
            | "items-stretch"
            | "justify-around"
            | "justify-between"
            | "justify-center"
            | "justify-end"
            | "justify-evenly"
            | "justify-start"
            | "line-through"
            | "no-underline"
            | "not-italic"
            | "overflow-hidden"
            | "overflow-scroll"
            | "overflow-x-hidden"
            | "overflow-x-scroll"
            | "overflow-y-hidden"
            | "overflow-y-scroll"
            | "relative"
            | "rounded-full"
            | "rounded-lg"
            | "rounded-md"
            | "rounded-none"
            | "rounded-sm"
            | "rounded-xl"
            | "row-end-auto"
            | "row-span-full"
            | "row-start-auto"
            | "self-baseline"
            | "self-center"
            | "self-end"
            | "self-flex-end"
            | "self-flex-start"
            | "self-start"
            | "self-stretch"
            | "shadow-2xl"
            | "shadow-2xs"
            | "shadow-lg"
            | "shadow-md"
            | "shadow-none"
            | "shadow-sm"
            | "shadow-xl"
            | "shadow-xs"
            | "size-full"
            | "size-px"
            | "text-2xl"
            | "text-3xl"
            | "text-base"
            | "text-center"
            | "text-decoration-0"
            | "text-decoration-1"
            | "text-decoration-2"
            | "text-decoration-4"
            | "text-decoration-8"
            | "text-decoration-solid"
            | "text-decoration-wavy"
            | "text-ellipsis"
            | "text-ellipsis-start"
            | "text-left"
            | "text-lg"
            | "text-right"
            | "text-sm"
            | "text-xl"
            | "text-xs"
            | "truncate"
            | "underline"
            | "w-auto"
            | "w-full"
            | "w-px"
            | "whitespace-normal"
            | "whitespace-nowrap"
    )
}
