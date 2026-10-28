//! Known class expansion and parsing of lengths, colors, and numeric values.

use super::super::tables::*;
use super::{ClassMode, SpacingScale};
pub use crate::StyleLength as LengthKind;
use crate::{StyleOperation, StyleValue};
use proc_macro2::{Span, TokenStream};
use quote::quote;

/// 已知 class 的展开。`None` 表示未知；`Some(Err)` 表示形式已知但取值非法。
pub(super) fn expand_known_class(
    class: &str,
    mode: ClassMode,
) -> Option<Result<TokenStream, String>> {
    if let Some(operation) = resolve_style(class, mode) {
        return Some(operation.map(|operation| operation.to_tokens()));
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
        "group" => return Some(Ok(quote! { .group("") })),
        _ => {}
    }
    if let Some(name) = class.strip_prefix("group/") {
        return Some(Ok(quote! { .group(#name) }));
    }

    if let Some(token) = parse_direct_line_clamp_class(class) {
        return Some(Ok(token));
    }
    if let Some(token) = parse_direct_directional_border_class(class) {
        return Some(Ok(token));
    }

    None
}

pub(super) fn resolve_style(
    class: &str,
    mode: ClassMode,
) -> Option<Result<StyleOperation, String>> {
    if let Some(theme) = mode.theme()
        && let Some(operation) = theme.style_class(class)
    {
        return Some(Ok(operation));
    }
    if let Some((_, weight)) = FONT_WEIGHTS.iter().find(|(name, _)| *name == class) {
        return Some(Ok(StyleOperation::new(
            "font_weight",
            Some(StyleValue::Expression(format!("FontWeight::{weight}"))),
        )));
    }
    let alias = match class {
        "border" => Some("border_1"),
        "flex-grow" => Some("flex_grow_1"),
        "flex-shrink" => Some("flex_shrink_1"),
        "no-underline" => Some("text_decoration_none"),
        _ => None,
    };
    if let Some(method) = alias {
        return Some(Ok(StyleOperation::new(method, None)));
    }
    if let Some(result) = parse_arbitrary_length_class(class) {
        return Some(result);
    }
    if let Some(result) = parse_fraction_length_class(class) {
        return Some(result);
    }
    if let Some(operation) = parse_numeric_length_class(class, mode.scale()) {
        return Some(Ok(operation));
    }
    if let Some(result) = parse_direct_color_class(class) {
        return Some(result);
    }
    if let Some(result) = parse_direct_opacity_class(class) {
        return Some(result);
    }
    if let Some(method) = gpui_no_arg_class_method(class) {
        return Some(Ok(StyleOperation::new(method.name, None)));
    }
    None
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

        let ratio = numerator / denominator;
        if !ratio.is_finite() {
            return Err(format!(
                "Invalid fraction `{class}`: value is out of range."
            ));
        }
        Ok(Self::Relative(ratio))
    }
}

fn parse_arbitrary_length_class(class: &str) -> Option<Result<StyleOperation, String>> {
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

fn parse_fraction_length_class(class: &str) -> Option<Result<StyleOperation, String>> {
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

fn parse_numeric_length_class(class: &str, scale: SpacingScale) -> Option<StyleOperation> {
    let (method, value, _) = split_length_class(class)?;
    if value.starts_with('[') || value.contains('/') {
        return None;
    }
    // Scaling can overflow f32 (`w-1e38` × 4); `quote` panics on non-finite literals.
    parse_length_number(value)
        .map(|value| value * scale.factor())
        .filter(|px| px.is_finite())
        .map(|px| length_method_call(method, LengthKind::Px(px)))
}

fn parse_direct_color_class(class: &str) -> Option<Result<StyleOperation, String>> {
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

fn parse_direct_opacity_class(class: &str) -> Option<Result<StyleOperation, String>> {
    let rest = class.strip_prefix("opacity-")?;
    let n = match rest.parse::<u8>() {
        Ok(n) if n <= 100 => n,
        _ => return Some(Err(invalid_opacity_value_message(class))),
    };
    let val = n as f32 / 100.0;
    Some(Ok(StyleOperation::new(
        "opacity",
        Some(StyleValue::Number(val)),
    )))
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

fn length_method_call(method: &'static str, length: LengthKind) -> StyleOperation {
    StyleOperation::new(method, Some(StyleValue::Length(length)))
}

/// 统一的颜色解析函数（核心去重逻辑）
///
/// 将颜色名称或任意 hex / rgb() / rgba() 值转换为方法调用。
fn parse_color_with_method(
    color: &str,
    method: &'static str,
    class: &str,
) -> Option<Result<StyleOperation, String>> {
    let parsed = lookup_color_key(color)
        .map(ColorValue::Rgb)
        .or_else(|| parse_arbitrary_color_value(color));
    let Some(parsed) = parsed else {
        if color.starts_with('[') {
            return Some(Err(invalid_color_value_message(class)));
        }
        return None;
    };
    Some(Ok(StyleOperation::new(
        method,
        Some(match parsed {
            ColorValue::Rgb(hex) => StyleValue::Color { hex, alpha: false },
            ColorValue::Rgba(hex) => StyleValue::Color { hex, alpha: true },
        }),
    )))
}

/// `0xRRGGBBAA` for a color class value (`red-500`, `[#f00]`, `[rgba(...)]`), as the class
/// expansion would interpret it. Shared by editor tooling so colors never disagree.
pub fn color_rgba(color: &str) -> Option<u32> {
    match lookup_color_key(color)
        .map(ColorValue::Rgb)
        .or_else(|| parse_arbitrary_color_value(color))?
    {
        ColorValue::Rgb(hex) => Some((hex << 8) | 0xff),
        ColorValue::Rgba(hex) => Some(hex),
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
