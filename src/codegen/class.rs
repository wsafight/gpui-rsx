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
//! - 文本大小使用 match 替代 contains 线性查找
//! - 单遍 `-` → `_` 替换（直接 replace，无需预检 contains）

use super::tables::*;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::borrow::Cow;

/// 解析 class 字符串为方法链片段列表
///
/// `"flex flex-col gap-4"` → `[.flex(), .flex_col(), .gap(px(4.0))]`
pub(crate) fn parse_class_string(class_str: &str) -> Vec<TokenStream> {
    class_str
        .split_whitespace()
        .map(parse_single_class)
        .collect()
}

/// 解析单个 CSS class 为方法调用
pub(crate) fn parse_single_class(class: &str) -> TokenStream {
    // 含 '-' 时分配新 String，不含则零拷贝借用原字符串
    let method_name: Cow<str> = if class.contains('-') {
        Cow::Owned(class.replace('-', "_"))
    } else {
        Cow::Borrowed(class)
    };

    // 间距/尺寸类：使用 rfind('_') + match 实现 O(1) 前缀查找
    if let Some(underscore_pos) = method_name.rfind('_') {
        let suffix = &method_name[underscore_pos + 1..];
        if let Ok(num) = suffix.parse::<f32>() {
            let prefix = &method_name[..=underscore_pos];
            if let Some(method) = lookup_spacing_method(prefix) {
                let method_ident = syn::Ident::new(method, Span::call_site());
                return quote! { .#method_ident(px(#num)) };
            }
        }
    }

    // border 特殊处理：
    // "border" (纯) → .border_1()（GPUI 没有无参 .border()）
    // "border-2" → .border_2()
    if method_name == "border" {
        return quote! { .border_1() };
    }

    // border-color 类：border-red-500 → .border_color(rgb(0xef4444))
    if let Some(color) = method_name.strip_prefix("border_") {
        // 排除方向性边框和数值边框宽度，用首字节快速判断
        let skip = match color.as_bytes().first() {
            Some(b't' | b'b' | b'l' | b'r' | b'x' | b'y')
                if color.len() == 1 || color.as_bytes().get(1) == Some(&b'_') =>
            {
                true
            }
            Some(b'0'..=b'9') => {
                // 数值边框宽度 border-2, border-4 等
                let ident = syn::Ident::new(&method_name, Span::call_site());
                return quote! { .#ident() };
            }
            _ => false,
        };
        if !skip {
            if let Some(token) = parse_color_with_method(color, "border_color") {
                return token;
            }
        }
    }

    // 颜色类：text-red-600, bg-blue-500 → .text_color(rgb(...)), .bg(rgb(...))
    if let Some(color_code) = parse_color_class(&method_name) {
        return color_code;
    }

    // 文本大小类：text-xl → .text_xl()（使用 match 替代线性查找）
    #[allow(clippy::collapsible_if)]
    if let Some(size) = method_name.strip_prefix("text_") {
        if is_valid_text_size(size) {
            let size_ident = syn::Ident::new(&method_name, Span::call_site());
            return quote! { .#size_ident() };
        }
        // 不在白名单中的 text_ 前缀，fall through 到默认处理
    }

    // 默认：无参方法调用
    let ident = syn::Ident::new(&method_name, Span::call_site());
    quote! { .#ident() }
}

/// 解析颜色 class（先分离前缀，再查表）
///
/// 使用统一的 `parse_color_with_method` 函数，避免重复代码。
fn parse_color_class(class: &str) -> Option<TokenStream> {
    if let Some(color) = class.strip_prefix("text_") {
        return parse_color_with_method(color, "text_color");
    }

    if let Some(color) = class.strip_prefix("bg_") {
        return parse_color_with_method(color, "bg");
    }

    None
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
fn parse_color_with_method(color: &str, method: &str) -> Option<TokenStream> {
    // 1. 尝试颜色表查找
    if let Some(hex) = lookup_color(color) {
        let ident = syn::Ident::new(method, Span::call_site());
        return Some(quote! { .#ident(rgb(#hex)) });
    }

    // 2. 尝试任意 hex 值
    if let Some(hex) = parse_arbitrary_hex(color) {
        let ident = syn::Ident::new(method, Span::call_site());
        return Some(quote! { .#ident(rgb(#hex)) });
    }

    None
}

/// 解析任意 hex 颜色值：`[#rrggbb]` 或 `[#rgb]`
///
/// 输入已经过 `-` → `_` 替换，但 `[#...]` 中不含 `-`，所以保持原样。
/// 返回解析后的 u32 颜色值。
fn parse_arbitrary_hex(s: &str) -> Option<u32> {
    // 匹配 [#rrggbb] 或 [#rgb]
    let inner = s.strip_prefix("[#")?.strip_suffix(']')?;
    match inner.len() {
        6 => u32::from_str_radix(inner, 16).ok(),
        3 => {
            // 3 位 hex 扩展为 6 位: #abc → #aabbcc
            let mut expanded = String::with_capacity(6);
            for ch in inner.chars() {
                expanded.push(ch);
                expanded.push(ch);
            }
            u32::from_str_radix(&expanded, 16).ok()
        }
        _ => None,
    }
}
