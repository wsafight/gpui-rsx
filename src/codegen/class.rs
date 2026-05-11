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

/// 解析 class 字符串为方法链片段迭代器
///
/// `"flex flex-col gap-4"` → `[.flex(), .flex_col(), .gap(px(4.0))]`
///
/// 返回迭代器而非 Vec，调用方通过 `extend` 消费时避免中间 Vec 分配。
pub(crate) fn parse_class_string(class_str: &str) -> impl Iterator<Item = TokenStream> + '_ {
    class_str.split_ascii_whitespace().map(parse_single_class)
}

/// 解析单个 CSS class 为方法调用
pub(crate) fn parse_single_class(class: &str) -> TokenStream {
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
        if let Ok(num) = suffix.parse::<f32>() {
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
        } else if let Some(token) = parse_color_with_method(rest, "border_color") {
            return token;
        }
    }

    // text_ 前缀：统一处理颜色类（text-red-600）和文本大小类（text-xl）
    // 只做一次 strip_prefix("text_")，避免先在颜色分支、再在大小分支各做一次。
    if let Some(rest) = method_name.strip_prefix("text_") {
        // 先查颜色表（text-red-500 → .text_color(rgb(...))）
        if let Some(token) = parse_color_with_method(rest, "text_color") {
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
        && let Some(token) = parse_color_with_method(rest, "bg")
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

    // 默认：无参方法调用（防御性检查：跳过含非标识符字符的 class，如 "hover:bg-blue-500"）
    if !method_name.is_empty()
        && method_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        let ident = syn::Ident::new(&method_name, Span::call_site());
        quote! { .#ident() }
    } else {
        quote! {}
    }
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
    // 统一颜色表查找和任意 hex 解析，仅在匹配成功时创建 Ident
    let hex = lookup_color(color).or_else(|| parse_arbitrary_hex(color))?;
    let ident = syn::Ident::new(method, Span::call_site());
    Some(quote! { .#ident(rgb(#hex)) })
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
fn parse_arbitrary_hex(s: &str) -> Option<u32> {
    // 匹配 [#rrggbb] 或 [#rgb]
    let inner = s.strip_prefix("[#")?.strip_suffix(']')?;
    match inner.len() {
        6 => u32::from_str_radix(inner, 16).ok(),
        3 => {
            // 3 位 hex 扩展为 6 位: #abc → #aabbcc
            // 用位运算零分配实现，避免 String 堆分配
            let b = inner.as_bytes();
            let d = |c: u8| -> Option<u32> { (c as char).to_digit(16) };
            let r = d(b[0])?;
            let g = d(b[1])?;
            let bl = d(b[2])?;
            // 每个 4-bit 数字复制到高低 nibble: 0xA → 0xAA
            Some(r << 20 | r << 16 | g << 12 | g << 8 | bl << 4 | bl)
        }
        _ => None,
    }
}
