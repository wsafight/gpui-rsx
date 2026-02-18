//! 元素代码生成
//!
//! 将 RSX 元素转换为 GPUI 方法链代码：
//! - 基础标签构造
//! - 自动 ID 管理
//! - 子节点聚合优化
//! - Fragment 和 For 循环支持
//!
//! 优化：
//! - 缓存 `Ident::to_string()` 避免重复堆分配
//! - 使用 match-based `is_stateful_attr()` 替代双重线性扫描
//! - 使用 `lookup_tag_default()` 替代 `.iter().find()` 线性查找
//! - `generate_attr_methods` 直接 push 到调用方 Vec
//! - 复用 `consecutive_exprs` Vec 避免循环内反复分配

use super::attribute::generate_attr_methods;
use super::class::parse_class_string;
use super::tables::{is_stateful_attr, lookup_tag_default};
use crate::parser::{RsxAttribute, RsxBody, RsxElement, RsxNode};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::cell::Cell;

// 编译期自动 ID 计数器（每次宏展开递增，保证唯一）
//
// proc macro 运行在单线程环境中，使用 `thread_local! + Cell` 替代 `AtomicUsize`，
// 语义更准确且无原子操作开销。
//
// 已知限制：增量编译下的 ID 稳定性
// 计数器在单次编译进程中单调递增。若增量编译只重新处理部分文件，
// 宏展开顺序可能改变，导致自动生成的 ID 与上次编译不同。
// 对于依赖 ID 做焦点/状态追踪的元素，建议显式指定 `id` 属性以保证稳定性。
thread_local! {
    static AUTO_ID_COUNTER: Cell<usize> = const { Cell::new(0) };
}


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
///
/// 多子节点使用 `vec![]` 而非数组 `[...]`：数组要求所有元素同类型，
/// 当 body 中混合不同元素类型时（如 `div()` 和自定义组件）会产生类型错误。
fn generate_for_loop(binding: &syn::Pat, iter: &syn::Expr, body: &[RsxNode]) -> TokenStream {
    let body_exprs: Vec<TokenStream> = body.iter().map(generate_node).collect();
    if body_exprs.len() == 1 {
        let single = &body_exprs[0];
        quote! { (#iter).into_iter().map(|#binding| #single) }
    } else {
        quote! { (#iter).into_iter().flat_map(|#binding| vec![#(#body_exprs),*]) }
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
    // 缓存标签名字符串，避免多次 to_string() 堆分配
    let tag_str = element.name.to_string();

    // 单次遍历提取所有需要的信息
    let mut user_id = None;
    let mut has_styled = false;
    let mut needs_id = false;

    for attr in &element.attributes {
        match attr {
            RsxAttribute::Value { name, value } if name == "id" => {
                user_id = Some(value);
            }
            RsxAttribute::Flag(name) if name == "styled" => {
                has_styled = true;
            }
            RsxAttribute::Value { name, .. } | RsxAttribute::Flag(name) => {
                if !needs_id {
                    // 缓存属性名字符串，避免循环内重复 to_string()
                    let attr_str = name.to_string();
                    needs_id = is_stateful_attr(&attr_str);
                }
            }
            _ => {}
        }
    }

    // 生成基础元素和 id
    let tag = generate_tag(&tag_str, &element.name);
    let base = if let Some(id_value) = user_id {
        quote! { #tag.id(#id_value) }
    } else if needs_id {
        let auto_id = next_auto_id(&tag_str);
        quote! { #tag.id(#auto_id) }
    } else {
        tag
    };

    // 快速路径：无属性且无子节点时直接返回基础元素
    if element.attributes.is_empty() && element.children.is_empty() {
        return base;
    }

    // 预分配方法链容量（属性数 + 子节点数的估计值）
    let mut methods: Vec<TokenStream> =
        Vec::with_capacity(element.attributes.len() + element.children.len());

    // styled 标志 → 注入标签默认样式（在用户属性之前）
    if has_styled {
        if let Some(class_str) = lookup_tag_default(&tag_str) {
            methods.extend(parse_class_string(class_str));
        }
    }

    // 属性 → 方法调用（直接 push 到 methods，避免中间 Vec 分配）
    for attr in &element.attributes {
        generate_attr_methods(attr, &mut methods);
    }

    // 子节点 → .child() / .children() 调用（含聚合优化）
    generate_children_methods(&element.children, &mut methods);

    quote! { #base #(#methods)* }
}

/// 生成子节点的方法链片段
///
/// 当连续 3+ 个 Expr 子节点时，合并为单个 `.children([...])` 调用。
/// `consecutive_exprs` 在循环外分配，每轮用 `.clear()` 复用，减少堆分配。
fn generate_children_methods(children: &[RsxNode], methods: &mut Vec<TokenStream>) {
    let mut i = 0;
    let mut consecutive_exprs: Vec<TokenStream> = Vec::with_capacity(4);

    while i < children.len() {
        // 收集连续的 Expr 子节点（复用 Vec）
        consecutive_exprs.clear();
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
            for expr in &consecutive_exprs {
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
                    // Expr 节点应该已在上面的 consecutive_exprs 循环中处理
                    unreachable!(
                        "BUG in gpui-rsx codegen: Expr node should have been consumed above"
                    )
                }
            }
            i += 1;
        }
    }
}

/// HTML 标签 → `div()`，特殊标签 → 同名函数，自定义组件 → 同名函数调用
///
/// 接受预缓存的 `tag_str` 避免重复 `to_string()`
fn generate_tag(tag_str: &str, name: &syn::Ident) -> TokenStream {
    match tag_str {
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

/// 生成确定性自动 ID 字符串
///
/// 使用全局计数器 + 标签名生成唯一 ID，替代原先的 SipHash 计算。
/// 计数器在单次编译过程中保证唯一性，但在增量编译下可能因展开顺序变化而改变。
/// 若元素的状态追踪依赖稳定 ID，请在 RSX 中显式指定 `id` 属性。
fn next_auto_id(tag: &str) -> String {
    AUTO_ID_COUNTER.with(|c| {
        let n = c.get();
        c.set(n + 1);
        format!("__rsx_{tag}_{n}")
    })
}
