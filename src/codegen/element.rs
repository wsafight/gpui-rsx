//! 元素代码生成
//!
//! 将 RSX 元素转换为 GPUI 方法链代码：
//! - 基础标签构造
//! - 自动 ID 管理
//! - 子节点聚合优化
//! - Fragment 和 For 循环支持

use super::attribute::generate_attr_methods;
use super::class::parse_class_string;
use super::tables::TAG_DEFAULT_STYLES;
use crate::parser::{RsxAttribute, RsxBody, RsxElement, RsxNode};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};

/// 编译期自动 ID 计数器（每次宏展开递增，保证唯一）
static AUTO_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// 需要 `.id()` 的属性（`StatefulInteractiveElement` trait）
const NEEDS_ID_ATTRS: &[&str] = &[
    "onClick",
    "on_click",
    "onHover",
    "on_hover",
    "onDrag",
    "on_drag",
    "onDrop",
    "on_drop",
    "hover",
    "active",
    "focus",
    "tooltip",
    "group",
    "track_focus",
];

/// 生成 GPUI 代码（入口）
///
/// 将解析后的 RSX AST 转换为 GPUI 的类型安全代码。
///
/// # 返回值
/// - 单个元素：返回实现 `IntoElement` 的表达式
/// - Fragment：返回 `Vec<impl IntoElement>`
pub fn generate_body(body: &RsxBody) -> TokenStream {
    match body {
        RsxBody::Single(element) => generate_element(element),
        RsxBody::Fragment(children) => {
            let child_exprs: Vec<TokenStream> = children.iter().map(generate_node).collect();
            // Fragment 保持 vec![] —— 返回类型是用户可见 API
            quote! { vec![#(#child_exprs),*] }
        }
    }
}

/// 生成单个子节点的代码
///
/// 确保生成的代码具有正确的类型推断，支持 IntoElement trait
fn generate_node(node: &RsxNode) -> TokenStream {
    match node {
        RsxNode::Element(elem) => generate_element(elem),
        // 表达式会被自动推断类型，GPUI 的 .child() 接受 impl IntoElement
        RsxNode::Expr(expr) => expr.to_token_stream(),
        RsxNode::Spread(expr) => expr.to_token_stream(),
        RsxNode::For {
            binding,
            iter,
            body,
        } => generate_for_loop(binding, iter, body),
    }
}

/// 生成 for 循环的迭代器代码
///
/// 单个子节点 → `.map()`，多个子节点 → `.flat_map()` + `vec![]`
fn generate_for_loop(binding: &syn::Pat, iter: &syn::Expr, body: &[RsxNode]) -> TokenStream {
    let body_exprs: Vec<TokenStream> = body.iter().map(generate_node).collect();
    if body_exprs.len() == 1 {
        let single = &body_exprs[0];
        quote! { (#iter).into_iter().map(|#binding| #single) }
    } else {
        quote! { (#iter).into_iter().flat_map(|#binding| [#(#body_exprs),*]) }
    }
}

/// 生成单个元素的代码
///
/// 生成形如 `div().id("x").flex().child(...)` 的方法链，
/// 而非 `let mut element = div(); element = element.flex();` 的赋值模式。
///
/// 方法链模式的优势：
/// - 与 GPUI 惯用写法一致
/// - 正确处理 `Div` → `Stateful<Div>` 的类型变换（`.id()` 后类型改变）
pub(crate) fn generate_element(element: &RsxElement) -> TokenStream {
    let base = generate_base(element);

    let mut methods: Vec<TokenStream> = Vec::new();

    // styled 标志 → 注入标签默认样式（在用户属性之前）
    let has_styled = element
        .attributes
        .iter()
        .any(|a| matches!(a, RsxAttribute::Flag(name) if name == "styled"));
    if has_styled {
        let tag_name = element.name.to_string();
        if let Some(&(_, class_str)) = TAG_DEFAULT_STYLES.iter().find(|&&(tag, _)| tag == tag_name)
        {
            methods.extend(parse_class_string(class_str));
        }
    }

    // 属性 → 方法调用（用户属性在默认样式之后，可覆盖）
    for attr in &element.attributes {
        methods.extend(generate_attr_methods(attr));
    }

    // 子节点 → .child() / .children() 调用（含聚合优化）
    generate_children_methods(&element.children, &mut methods);

    quote! { #base #(#methods)* }
}

/// 生成子节点的方法链片段
///
/// 当连续 3+ 个 Expr 子节点时，合并为单个 `.children(vec![...])` 调用。
fn generate_children_methods(children: &[RsxNode], methods: &mut Vec<TokenStream>) {
    let mut i = 0;
    while i < children.len() {
        // 收集连续的 Expr 子节点
        let mut consecutive_exprs: Vec<TokenStream> = Vec::new();
        while i < children.len() {
            if let RsxNode::Expr(expr) = &children[i] {
                consecutive_exprs.push(expr.to_token_stream());
                i += 1;
            } else {
                break;
            }
        }

        // 3 个及以上连续 Expr → .children([...])，用数组避免堆分配
        if consecutive_exprs.len() >= 3 {
            methods.push(quote! { .children([#(#consecutive_exprs),*]) });
        } else {
            for expr in consecutive_exprs {
                methods.push(quote! { .child(#expr) });
            }
        }

        // 处理非 Expr 节点
        if i < children.len() {
            match &children[i] {
                RsxNode::Element(elem) => {
                    let child_expr = generate_element(elem);
                    methods.push(quote! { .child(#child_expr) });
                }
                RsxNode::Spread(expr) => {
                    methods.push(quote! { .children(#expr) });
                }
                RsxNode::For {
                    binding,
                    iter,
                    body,
                } => {
                    let for_expr = generate_for_loop(binding, iter, body);
                    methods.push(quote! { .children(#for_expr) });
                }
                RsxNode::Expr(_) => {
                    // Expr 节点应该已在上面的 consecutive_exprs 循环（第 134-140 行）中处理
                    // 如果执行到这里，说明代码逻辑存在 bug
                    panic!(
                        "INTERNAL BUG in gpui-rsx codegen: Expr node reached unreachable code path. \
                         All Expr nodes should have been consumed in the consecutive_exprs loop (lines 134-140). \
                         This indicates a logic error in generate_children_methods()."
                    )
                }
            }
            i += 1;
        }
    }
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
        let auto_id = next_auto_id(&element.name.to_string(), &element.attributes);
        quote! { #tag.id(#auto_id) }
    } else {
        tag
    }
}

/// HTML 标签 → `div()`，特殊标签 → 同名函数，自定义组件 → 同名函数调用
fn generate_tag(name: &syn::Ident) -> TokenStream {
    match name.to_string().as_str() {
        // 特殊标签：保留为同名函数调用
        "svg" => quote! { svg() },
        "img" => quote! { img() },
        "canvas" => quote! { canvas() },
        // HTML 标签：统一映射为 div()
        "div" | "span" | "section" | "article" | "header" | "footer" | "main" | "nav" | "aside"
        | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "label" | "a" | "button" | "input"
        | "textarea" | "select" | "form" | "ul" | "ol" | "li" => {
            quote! { div() }
        }
        _ => quote! { #name() },
    }
}

/// 检查属性列表中是否存在需要 `Stateful<Div>` 的属性
fn needs_stateful_id(attributes: &[RsxAttribute]) -> bool {
    attributes.iter().any(|attr| match attr {
        RsxAttribute::Value { name, .. } | RsxAttribute::Flag(name) => {
            let n = name.to_string();
            NEEDS_ID_ATTRS.iter().any(|&s| n == s)
        }
        _ => false,
    })
}

/// 生成确定性自动 ID 字符串
///
/// 使用标签名和属性名的哈希 + 全局计数器，减少编译顺序敏感度。
fn next_auto_id(tag: &str, attributes: &[RsxAttribute]) -> String {
    let mut hasher = DefaultHasher::new();
    tag.hash(&mut hasher);
    for attr in attributes {
        match attr {
            RsxAttribute::Flag(name) | RsxAttribute::Value { name, .. } => {
                name.to_string().hash(&mut hasher);
            }
            _ => {}
        }
    }
    // 保留计数器确保同签名元素不冲突
    let n = AUTO_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    n.hash(&mut hasher);
    let hash = hasher.finish();
    format!("__rsx_{tag}_{hash:x}")
}
