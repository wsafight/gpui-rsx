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

use super::attribute::{AttrHints, generate_attr_methods_with_mode, static_class_expr_needs_id};
use super::class::{ClassMode, parse_class_string_with_mode};
use super::tables::{
    HTML_TAGS, is_stateful_attr, is_stateful_class, lookup_attr_flag_method, lookup_tag_default,
};
use crate::diagnostics::{for_loop_missing_key_error, missing_required_attribute_error};

#[derive(Default)]
struct AttrAnalysis {
    name: Option<String>,
    static_class: Option<String>,
    needs_id: bool,
}

impl AttrAnalysis {
    fn hints<'a>(&'a self, unknown_class_handler: Option<&'a syn::Expr>) -> AttrHints<'a> {
        AttrHints {
            name: self.name.as_deref(),
            static_class: self.static_class.as_deref(),
            unknown_class_handler,
        }
    }
}

fn analyze_attr(attr: &RsxAttribute, mode: ClassMode) -> AttrAnalysis {
    match attr {
        RsxAttribute::Value { name, value: _ } if name == "id" || name == "key" => {
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
            let needs_id = if let Some(class) = static_class.as_deref() {
                class.split_ascii_whitespace().any(is_stateful_class)
            } else {
                static_class_expr_needs_id(value, mode)
            };

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
        RsxAttribute::StateClass { method, .. } => {
            let name = method.to_string();
            let needs_id = is_stateful_attr(&name);
            AttrAnalysis {
                name: Some(name),
                needs_id,
                ..AttrAnalysis::default()
            }
        }
        _ => AttrAnalysis::default(),
    }
}

use crate::parser::{RsxAttribute, RsxBody, RsxElement, RsxElementName, RsxNode};
use proc_macro2::{Span, TokenStream};
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
    generate_body_with_erased_roots(body, mode, false)
}

pub fn generate_fragment_with_mode(body: &RsxBody, mode: ClassMode) -> TokenStream {
    generate_body_with_erased_roots(body, mode, true)
}

fn generate_body_with_erased_roots(
    body: &RsxBody,
    mode: ClassMode,
    erase_roots: bool,
) -> TokenStream {
    let mut dynamic_used = false;
    let code = generate_body_checked(body, mode.clone(), &mut dynamic_used, erase_roots)
        .unwrap_or_else(|err| err);
    if dynamic_used && mode.dynamic_runtime_path().is_none() {
        let helper = super::runtime::get_cached_dynamic_class_helper(mode);
        quote! { { #helper #code } }
    } else {
        code
    }
}

/// Like [`generate_body_with_mode`] but without the dynamic class helper, which is about
/// 1,400 lines; tools that only analyze or display the call site use this. The flag reports
/// whether the omitted helper (`__rsx_apply_class`) is referenced.
pub fn generate_body_without_runtime(body: &RsxBody, mode: ClassMode) -> (TokenStream, bool) {
    generate_body_without_runtime_with_erased_roots(body, mode, false)
}

pub fn generate_fragment_without_runtime(body: &RsxBody, mode: ClassMode) -> (TokenStream, bool) {
    generate_body_without_runtime_with_erased_roots(body, mode, true)
}

fn generate_body_without_runtime_with_erased_roots(
    body: &RsxBody,
    mode: ClassMode,
    erase_roots: bool,
) -> (TokenStream, bool) {
    let inline_runtime = mode.dynamic_runtime_path().is_none();
    let mut dynamic_used = false;
    let code =
        generate_body_checked(body, mode, &mut dynamic_used, erase_roots).unwrap_or_else(|err| err);
    (code, dynamic_used && inline_runtime)
}

pub fn generate_body_expansion_preview(body: &RsxBody, mode: ClassMode) -> String {
    generate_body_with_mode(body, mode).to_string()
}

fn generate_body_checked(
    body: &RsxBody,
    mode: ClassMode,
    dynamic_used: &mut bool,
    erase_roots: bool,
) -> CodegenResult {
    match body {
        RsxBody::Single(element) => {
            let expr = generate_element_checked(element, false, mode, dynamic_used)?;
            if erase_roots {
                Ok(collect_any_elements(
                    &[(expr, false)],
                    quote! { ::gpui::AnyElement },
                ))
            } else {
                Ok(expr)
            }
        }
        RsxBody::Fragment(children) => {
            let child_exprs = generate_flattened_nodes(children, false, mode, dynamic_used)?;
            if erase_roots || child_exprs.iter().any(|(_, yields_many)| *yields_many) {
                // A loop or spread contributes many elements, so erase types and collect.
                return Ok(collect_any_elements(
                    &child_exprs,
                    quote! { ::gpui::AnyElement },
                ));
            }
            let exprs = child_exprs.iter().map(|(expr, _)| expr);
            // Fragment 保持 vec![] —— 返回类型是用户可见 API
            Ok(quote! { vec![#(#exprs),*] })
        }
    }
}

/// 生成单个子节点的代码
///
/// 确保生成的代码具有正确的类型推断，支持 IntoElement trait
fn generate_node_checked(
    node: &RsxNode,
    require_loop_key: bool,
    mode: ClassMode,
    dynamic_used: &mut bool,
) -> CodegenResult {
    match node {
        RsxNode::Element(elem) => {
            generate_element_checked(elem, require_loop_key, mode, dynamic_used)
        }
        // Every caller iterates `flattened_nodes`, which never yields a fragment group.
        RsxNode::Fragment(_) => unreachable!("nested fragment was not flattened"),
        // 表达式会被自动推断类型，GPUI 的 .child() 接受 impl IntoElement
        RsxNode::Expr(expr) => Ok(expr.to_token_stream()),
        RsxNode::Spread(expr) => Ok(expr.to_token_stream()),
        RsxNode::For {
            binding,
            iter,
            body,
        } => generate_for_loop_checked(binding, iter, body, mode, dynamic_used),
    }
}

/// 生成 for 循环的迭代器代码
///
/// 单个子节点 → `.map()`，多个子节点 → `.flat_map()` + `AnyElement` 数组；
/// 循环体中含嵌套 `for` 或展开 `{...iter}` 时改用 `.flat_map()`，按源码顺序收集子节点。
///
/// 多子节点使用 `AnyElement` 做类型擦除后放入数组，避免每轮循环分配 `Vec`，
/// 同时允许循环体内混合不同具体元素类型（如 `div()` 和自定义组件）。
/// 直接嵌套的迭代器在每轮外层循环内收集，避免借用外层 pattern 的闭包逃逸。
///
/// 安全检查：循环体内所有 stateful 元素（含深层嵌套）都必须提供 `id` 或 `key`，
/// 否则每次迭代会生成相同的自动 ID，导致 GPUI 状态冲突，因此在此阶段给出编译错误。
fn generate_for_loop_checked(
    binding: &syn::Pat,
    iter: &syn::Expr,
    body: &[RsxNode],
    mode: ClassMode,
    dynamic_used: &mut bool,
) -> CodegenResult {
    let body_exprs = generate_flattened_nodes(body, true, mode, dynamic_used)?;
    if body_exprs.is_empty() {
        Ok(quote! {
            (#iter).into_iter().flat_map(|#binding| ::core::iter::empty::<::gpui::AnyElement>())
        })
    } else if let [(single, yields_many)] = body_exprs.as_slice() {
        if *yields_many {
            // Consume nested iterators while the outer pattern is still alive. Their
            // closures may borrow that binding (or mutable state from the caller).
            Ok(quote! {
                (#iter).into_iter().flat_map(|#binding| {
                    ::core::iter::IntoIterator::into_iter(#single).collect::<::std::vec::Vec<_>>()
                })
            })
        } else {
            Ok(quote! { (#iter).into_iter().map(|#binding| #single) })
        }
    } else if body_exprs.iter().any(|(_, yields_many)| *yields_many) {
        let collected = collect_any_elements(&body_exprs, quote! { _ });
        Ok(quote! {
            (#iter).into_iter().flat_map(|#binding| #collected)
        })
    } else {
        let exprs = body_exprs.iter().map(|(expr, _)| expr);
        Ok(quote! {
            (#iter).into_iter().flat_map(|#binding| [#((#exprs).into_any_element()),*])
        })
    }
}

/// Generates flattened nodes, marking each one that yields many elements: a nested `for`
/// or a spread produces an iterator of elements rather than one element.
fn generate_flattened_nodes(
    nodes: &[RsxNode],
    require_loop_key: bool,
    mode: ClassMode,
    dynamic_used: &mut bool,
) -> Result<Vec<(TokenStream, bool)>, TokenStream> {
    flattened_nodes(nodes)
        .map(|node| {
            let expr = generate_node_checked(node, require_loop_key, mode.clone(), dynamic_used)?;
            let yields_many = matches!(node, RsxNode::For { .. } | RsxNode::Spread(_));
            Ok((expr, yields_many))
        })
        .collect()
}

/// Consume each iterator before evaluating the next node, preserving construction order
/// and releasing mutable borrows between siblings. The result uses one shared collection.
fn collect_any_elements(exprs: &[(TokenStream, bool)], element_type: TokenStream) -> TokenStream {
    let children = syn::Ident::new("__rsx_children", Span::mixed_site());
    let append = exprs.iter().map(|(expr, yields_many)| {
        if *yields_many {
            quote! {
                #children.extend(::core::iter::IntoIterator::into_iter(#expr)
                    .map(|__rsx_child| __rsx_child.into_any_element()));
            }
        } else {
            quote! { #children.push((#expr).into_any_element()); }
        }
    });
    quote! {{
        let mut #children = ::std::vec::Vec::<#element_type>::new();
        #(#append)*
        #children
    }}
}

/// Flatten syntax groups during expansion; nested fragments allocate no runtime Vec.
fn flattened_nodes(nodes: &[RsxNode]) -> impl Iterator<Item = &RsxNode> {
    let mut current = nodes.iter();
    // No allocation on the common path without fragment groups.
    let mut parents = Vec::new();
    std::iter::from_fn(move || {
        loop {
            match current.next() {
                Some(RsxNode::Fragment(children)) => {
                    parents.push(std::mem::replace(&mut current, children.iter()));
                }
                Some(node) => return Some(node),
                None => current = parents.pop()?,
            }
        }
    })
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
    dynamic_used: &mut bool,
) -> CodegenResult {
    // 缓存标签名字符串，避免多次 to_string() 堆分配
    let tag_str = element.name.to_string();

    // 快速路径：无属性且无子节点时，跳过所有扫描直接返回基础标签
    if element.attributes.is_empty() && element.children.is_empty() {
        return generate_tag(&tag_str, &element.name, None, None, None);
    }

    // 单次遍历提取所有需要的信息，同时生成用户属性方法。
    let mut user_id = None;
    let mut user_key = None;
    let mut base_expr = None;
    let mut img_source = None;
    let mut canvas_prepaint = None;
    let mut canvas_paint = None;
    let mut has_styled = false;
    let mut needs_id = false;
    let unknown_class_handler = element.attributes.iter().find_map(|attr| match attr {
        RsxAttribute::Value { name, value } if name == "onUnknownClass" => Some(value),
        _ => None,
    });

    // 预分配方法链容量：
    // - 每个属性乘以 2（class 属性平均展开 3-4 个方法，其余属性 1 个）
    // - 加上子节点数
    let mut methods: Vec<TokenStream> =
        Vec::with_capacity(element.attributes.len() * 2 + element.children.len());

    for attr in &element.attributes {
        match attr {
            RsxAttribute::Value { name, value } if name == "id" => {
                user_id = Some(value);
            }
            RsxAttribute::Value { name, value } if name == "key" => {
                user_key = Some(value);
            }
            RsxAttribute::Value { name, value } if name == "base" => {
                base_expr = Some(value);
            }
            RsxAttribute::Value { name, value }
                if tag_str == "img" && (name == "src" || name == "source") =>
            {
                img_source = Some(value);
            }
            RsxAttribute::Value { name, value } if tag_str == "canvas" && name == "prepaint" => {
                canvas_prepaint = Some(value);
            }
            RsxAttribute::Value { name, value } if tag_str == "canvas" && name == "paint" => {
                canvas_paint = Some(value);
            }
            RsxAttribute::Value { name, value } if tag_str == "svg" && name == "src" => {
                methods.push(quote! { .path(#value) });
            }
            RsxAttribute::Flag(name) if name == "styled" => {
                has_styled = true;
            }
            RsxAttribute::Value { name, .. } if name == "onUnknownClass" => {}
            _ => {
                let analysis = analyze_attr(attr, mode.clone());
                if !needs_id && analysis.needs_id {
                    needs_id = true;
                }
                generate_attr_methods_with_mode(
                    attr,
                    analysis.hints(unknown_class_handler),
                    &mut methods,
                    mode.clone(),
                    dynamic_used,
                );
            }
        }
    }

    if require_loop_key && needs_id && user_id.is_none() && user_key.is_none() {
        return Err(for_loop_missing_key_error(&element.name.path, &tag_str).to_compile_error());
    }

    // 生成基础元素和 id：
    //  1. 显式 id              → 直接使用，优先级最高
    //  2. 需要 id + key 存在   → 自动 ID 前缀 + key（运行时拼接，保证循环内唯一）
    //  3. 需要 id，无 key       → 纯源码位置的自动 ID
    //  4. 不需要 id            → 不注入（key 在此情况下静默忽略）
    let tag = if let Some(base) = base_expr {
        quote! { #base }
    } else {
        generate_tag(
            &tag_str,
            &element.name,
            img_source,
            canvas_prepaint,
            canvas_paint,
        )?
    };
    let base = if let Some(id_value) = user_id {
        quote! { #tag.id(#id_value) }
    } else if needs_id {
        let auto_id = if let Some(key_expr) = user_key {
            make_keyed_auto_id(&element.name, key_expr)
        } else {
            make_auto_id(&element.name)
        };
        // Custom constructors and base expressions may return RenderOnce builders whose
        // event setters must run before rendering. Only actual GPUI elements need an ID.
        let native_tag = element.name.as_single_ident().is_some()
            && (HTML_TAGS.contains(&tag_str.as_str())
                || matches!(tag_str.as_str(), "svg" | "img" | "canvas"));
        if base_expr.is_some() || !native_tag {
            super::auto_id::with_auto_id(tag, auto_id)
        } else {
            quote! { #tag.id(#auto_id) }
        }
    } else {
        tag
    };

    // styled 标志 → 注入标签默认样式（在用户属性之前）
    let default_methods: Vec<TokenStream> =
        if has_styled && let Some(class_str) = lookup_tag_default(&tag_str) {
            parse_class_string_with_mode(class_str, mode.clone(), proc_macro2::Span::call_site())
        } else {
            Vec::new()
        };

    // 子节点 → .child() / .children() 调用（含聚合优化）
    generate_children_methods(
        &element.children,
        require_loop_key,
        &mut methods,
        mode.clone(),
        dynamic_used,
    )?;

    Ok(quote! { #base #(#default_methods)* #(#methods)* })
}

/// 生成子节点的方法链片段
fn generate_children_methods(
    children: &[RsxNode],
    require_loop_key: bool,
    methods: &mut Vec<TokenStream>,
    mode: ClassMode,
    dynamic_used: &mut bool,
) -> Result<(), TokenStream> {
    for (expr, yields_many) in
        generate_flattened_nodes(children, require_loop_key, mode, dynamic_used)?
    {
        methods.push(if yields_many {
            quote! { .children(#expr) }
        } else {
            quote! { .child(#expr) }
        });
    }
    Ok(())
}

/// HTML 标签 → `div()`，特殊标签 → 同名函数，自定义组件 → 同名函数调用
///
/// 接受预缓存的 `tag_str` 避免重复 `to_string()`
fn generate_tag(
    tag_str: &str,
    name: &RsxElementName,
    img_source: Option<&syn::Expr>,
    canvas_prepaint: Option<&syn::Expr>,
    canvas_paint: Option<&syn::Expr>,
) -> CodegenResult {
    if name.as_single_ident().is_none() {
        let path = &name.path;
        return Ok(quote! { #path() });
    }

    let path = &name.path;
    Ok(match tag_str {
        // 特殊标签：保留为同名函数调用
        "svg" => quote! { svg() },
        "img" => {
            let Some(source) = img_source else {
                return Err(missing_required_attribute_error(
                    &name.path,
                    "img",
                    "src",
                    r#"<img src={"path/to/image.png"} />"#,
                )
                .to_compile_error());
            };
            quote! { img(#source) }
        }
        "canvas" => {
            let Some(prepaint) = canvas_prepaint else {
                return Err(missing_required_attribute_error(
                    &name.path,
                    "canvas",
                    "prepaint",
                    r#"<canvas prepaint={|bounds, window, cx| state} paint={|bounds, state, window, cx| { ... }} />"#,
                )
                .to_compile_error());
            };
            let Some(paint) = canvas_paint else {
                return Err(missing_required_attribute_error(
                    &name.path,
                    "canvas",
                    "paint",
                    r#"<canvas prepaint={|bounds, window, cx| state} paint={|bounds, state, window, cx| { ... }} />"#,
                )
                .to_compile_error());
            };
            quote! { canvas(#prepaint, #paint) }
        }
        // HTML 标签：统一映射为 div()（与 spec/工具共用同一份 HTML_TAGS 列表）
        tag if HTML_TAGS.contains(&tag) => quote! { div() },
        _ => quote! { #path() },
    })
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
fn make_auto_id(tag_name: &RsxElementName) -> TokenStream {
    let span = tag_name.span();
    let loc = span.start(); // 需要 proc-macro2 的 span-locations 特性
    let id_suffix = format!("__rsx_{}_L{}C{}", tag_name, loc.line, loc.column);
    quote! { concat!(file!(), "::", #id_suffix) }
}

/// 生成带 `key` 的复合自动 ID（用于循环场景）
///
/// 字面量 key 使用 concat!；具体整数值使用命名整数 ID（无字符串分配）。
/// 其他 Display 值保留格式化路径，表达式借用求值一次。
fn make_keyed_auto_id(tag_name: &RsxElementName, key_expr: &syn::Expr) -> TokenStream {
    let span = tag_name.span();
    let loc = span.start();
    // 编译期常量前缀，包含文件路径 + 源码位置，格式如：
    //   "src/views/list.rs::__rsx_li_L42C8_"
    let prefix_suffix = format!("::__rsx_{}_L{}C{}_", tag_name, loc.line, loc.column);
    if let Some(static_suffix) = static_key_suffix(key_expr) {
        return quote! { concat!(file!(), #prefix_suffix, #static_suffix) };
    }
    // 运行时将 key 追加到前缀后，生成如：
    //   "src/views/list.rs::__rsx_li_L42C8_item_42"
    super::key::dynamic_key(&prefix_suffix, key_expr)
}

fn static_key_suffix(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) => Some(lit.value()),
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit),
            ..
        }) => lit
            .base10_parse::<u128>()
            .ok()
            .map(|value| value.to_string()),
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Bool(lit),
            ..
        }) => Some(lit.value.to_string()),
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Char(lit),
            ..
        }) => Some(lit.value().to_string()),
        syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => match &*unary.expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit),
                ..
            }) => lit
                .base10_parse::<u128>()
                .ok()
                .map(|value| format!("-{value}")),
            _ => None,
        },
        syn::Expr::Paren(expr) => static_key_suffix(&expr.expr),
        syn::Expr::Group(expr) => static_key_suffix(&expr.expr),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element_name(name: &str) -> RsxElementName {
        RsxElementName {
            path: syn::parse_str(name).expect("valid element name"),
        }
    }

    #[test]
    fn img_requires_source() {
        let error = generate_tag("img", &element_name("img"), None, None, None)
            .expect_err("img without src must fail")
            .to_string();

        assert!(error.contains("Element `<img>` requires `src`"));
    }

    #[test]
    fn canvas_requires_prepaint_and_paint() {
        let callback: syn::Expr = syn::parse_quote!(callback);
        let name = element_name("canvas");

        let missing_prepaint = generate_tag("canvas", &name, None, None, Some(&callback))
            .expect_err("canvas without prepaint must fail")
            .to_string();
        assert!(missing_prepaint.contains("Element `<canvas>` requires `prepaint`"));

        let missing_paint = generate_tag("canvas", &name, None, Some(&callback), None)
            .expect_err("canvas without paint must fail")
            .to_string();
        assert!(missing_paint.contains("Element `<canvas>` requires `paint`"));
    }

    #[test]
    fn static_key_suffix_supports_literal_display_types() {
        let cases: [(syn::Expr, &str); 6] = [
            (syn::parse_quote!("item"), "item"),
            (syn::parse_quote!(42), "42"),
            (syn::parse_quote!(true), "true"),
            (syn::parse_quote!('x'), "x"),
            (syn::parse_quote!(-7), "-7"),
            (syn::parse_quote!((9)), "9"),
        ];

        for (expr, expected) in cases {
            assert_eq!(static_key_suffix(&expr).as_deref(), Some(expected));
        }
    }

    #[test]
    fn static_key_suffix_keeps_dynamic_values_at_runtime() {
        let dynamic: syn::Expr = syn::parse_quote!(item.id);
        let non_integer_negative: syn::Expr = syn::parse_quote!(-1.5);

        assert_eq!(static_key_suffix(&dynamic), None);
        assert_eq!(static_key_suffix(&non_integer_negative), None);
    }

    fn expand(source: &str) -> String {
        let body = syn::parse_str::<RsxBody>(source).expect("valid RSX body");
        generate_body_with_mode(&body, ClassMode::PERMISSIVE).to_string()
    }

    #[test]
    fn every_html_tag_maps_to_div() {
        for tag in HTML_TAGS {
            let tokens = generate_tag(tag, &element_name(tag), None, None, None)
                .expect("HTML tags need no attributes")
                .to_string();
            assert_eq!(tokens, "div ()", "<{tag}> should map to div()");
        }
    }

    #[test]
    fn unknown_tags_stay_function_calls() {
        let cases = [
            ("marquee", "marquee ()"),
            ("MyWidget", "MyWidget ()"),
            ("ui::Card", "ui :: Card ()"),
        ];
        for (tag, expected) in cases {
            let tokens = generate_tag(tag, &element_name(tag), None, None, None)
                .expect("custom tags need no attributes")
                .to_string();
            assert_eq!(tokens, expected, "<{tag}>");
        }
    }

    #[test]
    fn nested_and_spread_loop_bodies_are_flattened() {
        // A nested `for` as the only body node must flat_map, not produce iterators of iterators.
        let nested = expand("<ul>{for row in rows {{for cell in row {<li>{cell}</li>}}}}</ul>");
        assert_eq!(
            nested,
            "div () . children ((rows) . into_iter () . flat_map (| row | { :: core :: iter :: IntoIterator :: into_iter ((row) . into_iter () . map (| cell | div () . child (cell))) . collect :: < :: std :: vec :: Vec < _ >> () }))"
        );

        // A spread body yields an iterator too.
        let spread = expand("<ul>{for row in rows {{...row}}}</ul>");
        assert_eq!(
            spread,
            "div () . children ((rows) . into_iter () . flat_map (| row | { :: core :: iter :: IntoIterator :: into_iter (row) . collect :: < :: std :: vec :: Vec < _ >> () }))"
        );

        // Mixed bodies collect each piece before constructing its next sibling.
        let mixed = expand("<ul>{for row in rows {<li/> {...row}}}</ul>");
        assert!(mixed.contains("flat_map"), "{mixed}");
        assert!(mixed.contains(". push ("), "{mixed}");
        assert!(mixed.contains(". extend ("), "{mixed}");
        assert!(!mixed.contains("vec !"), "{mixed}");
    }

    #[test]
    fn plain_loop_bodies_keep_map_and_arrays() {
        let single = expand("<ul>{for row in rows {<li>{row}</li>}}</ul>");
        assert!(single.contains(". map (| row |"), "{single}");
        assert!(!single.contains("flat_map"), "{single}");

        let multiple = expand("<ul>{for row in rows {<li>{row}</li><li>{row}</li>}}</ul>");
        assert!(multiple.contains("flat_map"), "{multiple}");
        assert!(multiple.contains("into_any_element ()"), "{multiple}");
    }

    #[test]
    fn fragment_groups_have_the_same_expansion_as_ungrouped_children() {
        for (grouped, plain) in [
            (
                "<div><><span/><><span/></></></div>",
                "<div><span/><span/></div>",
            ),
            ("<><div/><><div/><div/></></>", "<><div/><div/><div/></>"),
            (
                "<div>{for x in xs {<><span/>{...row}</>}}</div>",
                "<div>{for x in xs {<span/>{...row}}}</div>",
            ),
        ] {
            assert_eq!(expand(grouped), expand(plain));
        }
    }

    #[test]
    fn root_fragments_with_loops_or_spreads_collect_any_elements() {
        for source in [
            "<>{for x in xs {<div/>}}</>",
            "<><div/><>{for x in xs {<div/>}}</></>",
            "<><div/>{...rest}</>",
        ] {
            let expanded = expand(source);
            assert!(!expanded.contains("vec !"), "{source}: {expanded}");
            assert!(
                expanded.contains("Vec :: < :: gpui :: AnyElement >"),
                "{source}: {expanded}"
            );
            assert!(expanded.contains(". extend ("), "{source}: {expanded}");
        }
        assert!(expand("<><div/><span/></>").starts_with("vec !"));
    }
}
