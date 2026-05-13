//! 元素代码生成
//!
//! 将 RSX 元素转换为 GPUI 方法链代码：
//! - 基础标签构造
//! - 自动 ID 管理（支持 `key` 属性组合 ID）
//! - 子节点聚合优化
//! - Fragment 和 For 循环支持（for 循环中的 stateful 元素须提供 `id` 或 `key`）
//!
//! 优化：
//! - 缓存 `Ident::to_string()` 避免重复堆分配
//! - 使用 match-based `is_stateful_attr()` 替代双重线性扫描
//! - 使用 `lookup_tag_default()` 替代 `.iter().find()` 线性查找
//! - `generate_attr_methods` 直接 push 到调用方 Vec
//! - 每个子节点独立生成 `.child()` 调用，避免数组类型统一约束
//! - 自动 ID 基于源码 span 位置（行号 + 列号），增量编译下保持稳定

use super::attribute::{AttrHints, generate_attr_methods_with_mode};
use super::class::{ClassMode, parse_class_string_with_mode};
use super::tables::{
    is_stateful_attr, is_stateful_class, lookup_attr_flag_method, lookup_tag_default,
};
use crate::diagnostics::for_loop_missing_key_error;

#[derive(Default)]
struct AttrAnalysis {
    name: Option<String>,
    static_class: Option<String>,
    needs_id: bool,
}

impl AttrAnalysis {
    fn hints(&self) -> AttrHints<'_> {
        AttrHints {
            name: self.name.as_deref(),
            static_class: self.static_class.as_deref(),
        }
    }
}

fn analyze_attr(attr: &RsxAttribute) -> AttrAnalysis {
    match attr {
        RsxAttribute::Value { name, value } if name == "id" || name == "key" => {
            AttrAnalysis::default()
        }
        RsxAttribute::Value { name, value } if name == "class" => {
            let static_class = if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                ..
            }) = value
            {
                Some(lit_str.value())
            } else {
                None
            };
            let needs_id = static_class
                .as_deref()
                .is_some_and(|s| s.split_ascii_whitespace().any(is_stateful_class));

            AttrAnalysis {
                static_class,
                needs_id,
                ..AttrAnalysis::default()
            }
        }
        RsxAttribute::Value { name, .. } => {
            let name = name.to_string();
            let needs_id = is_stateful_attr(&name);
            AttrAnalysis {
                name: Some(name),
                needs_id,
                ..AttrAnalysis::default()
            }
        }
        RsxAttribute::Flag(name) if name == "styled" => AttrAnalysis::default(),
        RsxAttribute::Flag(name) => {
            let name = name.to_string();
            let needs_id = is_stateful_attr(&name)
                || lookup_attr_flag_method(&name).is_some_and(is_stateful_attr);
            AttrAnalysis {
                name: Some(name),
                needs_id,
                ..AttrAnalysis::default()
            }
        }
        _ => AttrAnalysis::default(),
    }
}

use crate::parser::{RsxAttribute, RsxBody, RsxElement, RsxNode};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

type CodegenResult = Result<TokenStream, TokenStream>;

/// 生成 GPUI 代码（入口）
///
/// 将解析后的 RSX AST 转换为 GPUI 的类型安全代码。
///
/// # 返回值
/// - 单个元素：返回实现 `IntoElement` 的表达式
/// - Fragment：返回 `Vec<impl IntoElement>`
pub fn generate_body_with_mode(body: &RsxBody, mode: ClassMode) -> TokenStream {
    generate_body_checked(body, mode).unwrap_or_else(|err| err)
}

pub fn generate_body_expansion_preview(body: &RsxBody, mode: ClassMode) -> String {
    generate_body_with_mode(body, mode).to_string()
}

fn generate_body_checked(body: &RsxBody, mode: ClassMode) -> CodegenResult {
    match body {
        RsxBody::Single(element) => generate_element_checked(element, false, mode),
        RsxBody::Fragment(children) => {
            let child_exprs: Vec<TokenStream> = children
                .iter()
                .map(|node| generate_node_checked(node, false, mode))
                .collect::<Result<_, _>>()?;
            // Fragment 保持 vec![] —— 返回类型是用户可见 API
            Ok(quote! { vec![#(#child_exprs),*] })
        }
    }
}

/// 生成单个子节点的代码
///
/// 确保生成的代码具有正确的类型推断，支持 IntoElement trait
fn generate_node_checked(node: &RsxNode, require_loop_key: bool, mode: ClassMode) -> CodegenResult {
    match node {
        RsxNode::Element(elem) => generate_element_checked(elem, require_loop_key, mode),
        // 表达式会被自动推断类型，GPUI 的 .child() 接受 impl IntoElement
        RsxNode::Expr(expr) => Ok(expr.to_token_stream()),
        RsxNode::Spread(expr) => Ok(expr.to_token_stream()),
        RsxNode::For {
            binding,
            iter,
            body,
        } => generate_for_loop_checked(binding, iter, body, mode),
    }
}

/// 生成 for 循环的迭代器代码
///
/// 单个子节点 → `.map()`，多个子节点 → `.flat_map()` + `AnyElement` 数组
///
/// 多子节点使用 `AnyElement` 做类型擦除后放入数组，避免每轮循环分配 `Vec`，
/// 同时允许循环体内混合不同具体元素类型（如 `div()` 和自定义组件）。
///
/// 安全检查：循环体内所有 stateful 元素（含深层嵌套）都必须提供 `id` 或 `key`，
/// 否则每次迭代会生成相同的自动 ID，导致 GPUI 状态冲突，因此在此阶段给出编译错误。
fn generate_for_loop_checked(
    binding: &syn::Pat,
    iter: &syn::Expr,
    body: &[RsxNode],
    mode: ClassMode,
) -> CodegenResult {
    let body_exprs: Vec<TokenStream> = body
        .iter()
        .map(|node| generate_node_checked(node, true, mode))
        .collect::<Result<_, _>>()?;
    if body_exprs.len() == 1 {
        let single = &body_exprs[0];
        Ok(quote! { (#iter).into_iter().map(|#binding| #single) })
    } else {
        Ok(quote! {
            (#iter).into_iter().flat_map(|#binding| [#((#body_exprs).into_any_element()),*])
        })
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
fn generate_element_checked(
    element: &RsxElement,
    require_loop_key: bool,
    mode: ClassMode,
) -> CodegenResult {
    // 缓存标签名字符串，避免多次 to_string() 堆分配
    let tag_str = element.name.to_string();

    // 快速路径：无属性且无子节点时，跳过所有扫描直接返回基础标签
    if element.attributes.is_empty() && element.children.is_empty() {
        return Ok(generate_tag(&tag_str, &element.name));
    }

    // 单次遍历提取所有需要的信息，同时生成用户属性方法。
    let mut user_id = None;
    let mut user_key = None;
    let mut has_styled = false;
    let mut needs_id = false;

    // 预分配方法链容量：
    // - 每个属性乘以 2（class 属性平均展开 3-4 个方法，其余属性 1 个）
    // - 加上子节点数
    let mut methods: Vec<TokenStream> =
        Vec::with_capacity(element.attributes.len() * 2 + element.children.len());

    for attr in &element.attributes {
        let analysis = analyze_attr(attr);
        match attr {
            RsxAttribute::Value { name, value } if name == "id" => {
                user_id = Some(value);
            }
            RsxAttribute::Value { name, value } if name == "key" => {
                user_key = Some(value);
            }
            RsxAttribute::Flag(name) if name == "styled" => {
                has_styled = true;
            }
            _ => {
                if !needs_id && analysis.needs_id {
                    needs_id = true;
                }
                generate_attr_methods_with_mode(attr, analysis.hints(), &mut methods, mode);
            }
        }
    }

    if require_loop_key && needs_id && user_id.is_none() && user_key.is_none() {
        return Err(for_loop_missing_key_error(&element.name).to_compile_error());
    }

    // 生成基础元素和 id：
    //  1. 显式 id              → 直接使用，优先级最高
    //  2. 需要 id + key 存在   → 自动 ID 前缀 + key（运行时拼接，保证循环内唯一）
    //  3. 需要 id，无 key       → 纯源码位置的自动 ID
    //  4. 不需要 id            → 不注入（key 在此情况下静默忽略）
    let tag = generate_tag(&tag_str, &element.name);
    let base = if let Some(id_value) = user_id {
        quote! { #tag.id(#id_value) }
    } else if needs_id {
        if let Some(key_expr) = user_key {
            let keyed_id = make_keyed_auto_id(&element.name, key_expr);
            quote! { #tag.id(#keyed_id) }
        } else {
            let auto_id = make_auto_id(&element.name);
            quote! { #tag.id(#auto_id) }
        }
    } else {
        tag
    };

    // styled 标志 → 注入标签默认样式（在用户属性之前）
    let default_methods: Vec<TokenStream> =
        if has_styled && let Some(class_str) = lookup_tag_default(&tag_str) {
            parse_class_string_with_mode(class_str, mode).collect()
        } else {
            Vec::new()
        };

    // 子节点 → .child() / .children() 调用（含聚合优化）
    generate_children_methods(&element.children, require_loop_key, &mut methods, mode)?;

    Ok(quote! { #base #(#default_methods)* #(#methods)* })
}

/// 生成子节点的方法链片段
fn generate_children_methods(
    children: &[RsxNode],
    require_loop_key: bool,
    methods: &mut Vec<TokenStream>,
    mode: ClassMode,
) -> Result<(), TokenStream> {
    for node in children {
        match node {
            RsxNode::Expr(expr) => {
                methods.push(quote! { .child(#expr) });
            }
            RsxNode::Element(elem) => {
                let child_expr = generate_element_checked(elem, require_loop_key, mode)?;
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
                let for_expr = generate_for_loop_checked(binding, iter, body, mode)?;
                methods.push(quote! { .children(#for_expr) });
            }
        }
    }
    Ok(())
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

/// 生成基于源码位置的稳定自动 ID（无 key）
///
/// 格式：`concat!(file!(), "::", "__rsx_{tag}_L{line}C{col}")`
///
/// **稳定性**：只要元素源码位置不变，ID 不变（增量编译安全）。
/// **唯一性**：`file!()` 在用户侧展开，包含完整路径，跨文件全局唯一。
///
/// 若需要跨重构完全稳定的 ID，请使用 `id` 属性；
/// 若在循环内使用，请改用 `key` 属性。
fn make_auto_id(tag_ident: &syn::Ident) -> TokenStream {
    let span = tag_ident.span();
    let loc = span.start(); // 需要 proc-macro2 的 span-locations 特性
    let id_suffix = format!("__rsx_{}_L{}C{}", tag_ident, loc.line, loc.column);
    quote! { concat!(file!(), "::", #id_suffix) }
}

/// 生成带 `key` 的复合自动 ID（用于循环场景）
///
/// 格式：`format!("{file}::{prefix}_{key}", file!(), key_expr)`
///
/// `concat!(file!(), ...)` 在编译期求值（零开销），`key_expr` 在运行时拼接，
/// 使同一循环迭代内的每个元素获得唯一 ID。
/// `key_expr` 需实现 `std::fmt::Display`（数字、字符串、自定义类型均可）。
fn make_keyed_auto_id(tag_ident: &syn::Ident, key_expr: &syn::Expr) -> TokenStream {
    let span = tag_ident.span();
    let loc = span.start();
    // 编译期常量前缀，包含文件路径 + 源码位置，格式如：
    //   "src/views/list.rs::__rsx_li_L42C8_"
    let prefix_suffix = format!("::__rsx_{}_L{}C{}_", tag_ident, loc.line, loc.column);
    // 运行时将 key 追加到前缀后，生成如：
    //   "src/views/list.rs::__rsx_li_L42C8_item_42"
    quote! { format!(concat!(file!(), #prefix_suffix, "{}"), #key_expr) }
}
