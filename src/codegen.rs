//! 代码生成器
//!
//! 将解析后的 RSX 转换为 GPUI 代码
//!
//! 生成惯用的 GPUI 方法链模式：
//! ```ignore
//! div().id("auto_0").flex().bg(rgb(0xff)).on_click(handler).child("text")
//! ```

use crate::parser::{RsxAttribute, RsxElement, RsxNode};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::sync::atomic::{AtomicUsize, Ordering};

/// 编译期自动 ID 计数器（每次宏展开递增，保证唯一）
static AUTO_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// 需要 `.id()` 的事件（`StatefulInteractiveElement` trait）
const STATEFUL_EVENTS: &[&str] = &["onClick", "on_click"];

/// 事件处理器映射表：(camelCase, snake_case, method_name)
const EVENT_HANDLERS: &[(&str, &str, &str)] = &[
    ("onClick", "on_click", "on_click"),
    ("onMouseDown", "on_mouse_down", "on_mouse_down"),
    ("onMouseUp", "on_mouse_up", "on_mouse_up"),
    ("onMouseMove", "on_mouse_move", "on_mouse_move"),
    ("onKeyDown", "on_key_down", "on_key_down"),
    ("onKeyUp", "on_key_up", "on_key_up"),
    ("onFocus", "on_focus", "on_focus"),
    ("onBlur", "on_blur", "on_blur"),
];

/// 颜色映射表
const COLOR_MAP: &[(&str, u32)] = &[
    ("red_500", 0xef4444),
    ("red_600", 0xef4444),
    ("green_600", 0x22c55e),
    ("blue_500", 0x3b82f6),
    ("blue_600", 0x2563eb),
    ("gray_600", 0x6b7280),
];

/// 间距/尺寸 class 前缀映射表
const SPACING_PATTERNS: &[(&str, &str)] = &[
    ("gap_", "gap"),
    ("p_", "p"),
    ("px_", "px"),
    ("py_", "py"),
    ("pt_", "pt"),
    ("pb_", "pb"),
    ("pl_", "pl"),
    ("pr_", "pr"),
    ("m_", "m"),
    ("mx_", "mx"),
    ("my_", "my"),
    ("mt_", "mt"),
    ("mb_", "mb"),
    ("ml_", "ml"),
    ("mr_", "mr"),
    ("w_", "w"),
    ("h_", "h"),
];

/// 生成 GPUI 代码（入口）
pub fn generate_code(element: &RsxElement) -> TokenStream {
    generate_element(element)
}

/// 检查属性列表中是否存在需要 `Stateful<Div>` 的事件
fn needs_stateful_id(attributes: &[RsxAttribute]) -> bool {
    attributes.iter().any(|attr| {
        if let RsxAttribute::Value { name, .. } = attr {
            let n = name.to_string();
            STATEFUL_EVENTS.iter().any(|&s| n == s)
        } else {
            false
        }
    })
}

/// 生成唯一自动 ID 字符串
fn next_auto_id() -> String {
    let n = AUTO_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("__rsx_{n}")
}

// ---------------------------------------------------------------------------
// 元素生成（方法链模式）
// ---------------------------------------------------------------------------

/// 生成单个元素的代码
///
/// 生成形如 `div().id("x").flex().child(...)` 的方法链，
/// 而非 `let mut element = div(); element = element.flex();` 的赋值模式。
///
/// 方法链模式的优势：
/// - 与 GPUI 惯用写法一致
/// - 正确处理 `Div` → `Stateful<Div>` 的类型变换（`.id()` 后类型改变）
fn generate_element(element: &RsxElement) -> TokenStream {
    let base = generate_base(element);

    let mut methods: Vec<TokenStream> = Vec::new();

    // 属性 → 方法调用
    for attr in &element.attributes {
        methods.extend(generate_attr_methods(attr));
    }

    // 子节点 → .child() 调用
    for child in &element.children {
        let child_expr = match child {
            RsxNode::Element(elem) => generate_element(elem),
            RsxNode::Expr(expr) => expr.to_token_stream(),
        };
        methods.push(quote! { .child(#child_expr) });
    }

    quote! { #base #(#methods)* }
}

/// 生成元素基础构造表达式（含自动 `.id()` 插入）
///
/// GPUI 中 `on_click` 属于 `StatefulInteractiveElement` trait，
/// 需要先调用 `.id()` 将 `Div` 转为 `Stateful<Div>` 才能使用。
/// 此函数在检测到 `onClick` 时自动插入 `.id()`。
fn generate_base(element: &RsxElement) -> TokenStream {
    let tag = generate_tag(&element.name);

    // 优先使用用户显式提供的 id
    let user_id = element.attributes.iter().find_map(|a| match a {
        RsxAttribute::Value { name, value } if name == "id" => Some(value),
        _ => None,
    });

    if let Some(id_value) = user_id {
        quote! { #tag.id(#id_value) }
    } else if needs_stateful_id(&element.attributes) {
        let auto_id = next_auto_id();
        quote! { #tag.id(#auto_id) }
    } else {
        tag
    }
}

/// HTML 标签 → `div()`，自定义组件 → 同名函数调用
fn generate_tag(name: &syn::Ident) -> TokenStream {
    match name.to_string().as_str() {
        "div" | "span" | "section" | "article" | "header" | "footer" | "main" | "nav"
        | "aside" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "label" | "a"
        | "button" | "input" | "textarea" | "select" | "form" | "ul" | "ol" | "li" => {
            quote! { div() }
        }
        _ => quote! { #name() },
    }
}

// ---------------------------------------------------------------------------
// 属性生成
// ---------------------------------------------------------------------------

/// 生成属性的方法链片段（返回 `.method(args)` 列表）
fn generate_attr_methods(attr: &RsxAttribute) -> Vec<TokenStream> {
    match attr {
        // id 已在 generate_base 中处理，跳过避免重复
        RsxAttribute::Value { name, .. } if name == "id" => vec![],

        RsxAttribute::Flag(name) => {
            vec![quote! { .#name() }]
        }

        RsxAttribute::Value { name, value } => {
            let method_name = name.to_string();

            // class 属性 → 展开为多个样式方法
            if method_name == "class" {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = value
                {
                    return parse_class_string(&lit_str.value());
                }
            }

            // 事件处理器查表
            for &(camel, snake, method) in EVENT_HANDLERS {
                if method_name == camel || method_name == snake {
                    let method_ident =
                        syn::Ident::new(method, proc_macro2::Span::call_site());
                    return vec![quote! { .#method_ident(#value) }];
                }
            }

            // 默认：直接作为方法调用
            vec![quote! { .#name(#value) }]
        }
    }
}

// ---------------------------------------------------------------------------
// class 字符串解析
// ---------------------------------------------------------------------------

/// 解析 class 字符串为方法链片段列表
///
/// `"flex flex-col gap-4"` → `[.flex(), .flex_col(), .gap(px(4.0))]`
fn parse_class_string(class_str: &str) -> Vec<TokenStream> {
    class_str
        .split_whitespace()
        .filter_map(parse_single_class)
        .collect()
}

/// 解析单个 CSS class 为方法调用
fn parse_single_class(class: &str) -> Option<TokenStream> {
    let method_name = class.replace('-', "_");

    // 间距/尺寸类：gap-4 → .gap(px(4.0))
    for &(prefix, method) in SPACING_PATTERNS {
        if let Some(value) = method_name.strip_prefix(prefix) {
            if let Ok(num) = value.parse::<f32>() {
                let method_ident = syn::Ident::new(method, proc_macro2::Span::call_site());
                return Some(quote! { .#method_ident(px(#num)) });
            }
        }
    }

    // 颜色类：text-red-600 → .text_color(rgb(0xef4444))
    if let Some(color_code) = parse_color_class(&method_name) {
        return Some(color_code);
    }

    // 文本大小类：text-xl → .text_xl()
    if let Some(size) = method_name.strip_prefix("text_") {
        let size_ident =
            syn::Ident::new(&format!("text_{size}"), proc_macro2::Span::call_site());
        return Some(quote! { .#size_ident() });
    }

    // 默认：无参方法调用
    let ident = syn::Ident::new(&method_name, proc_macro2::Span::call_site());
    Some(quote! { .#ident() })
}

/// 解析颜色 class（先分离前缀，再查表，避免循环内重复 strip）
fn parse_color_class(class: &str) -> Option<TokenStream> {
    if let Some(color) = class.strip_prefix("text_") {
        for &(color_name, color_value) in COLOR_MAP {
            if color == color_name {
                return Some(quote! { .text_color(rgb(#color_value)) });
            }
        }
    } else if let Some(color) = class.strip_prefix("bg_") {
        for &(color_name, color_value) in COLOR_MAP {
            if color == color_name {
                return Some(quote! { .bg(rgb(#color_value)) });
            }
        }
    }
    None
}
