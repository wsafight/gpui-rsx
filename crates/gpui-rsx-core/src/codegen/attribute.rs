//! 属性处理
//!
//! 将 RSX 属性转换为 GPUI 方法调用：
//! - Flag 属性 → 无参方法调用
//! - Value 属性 → 带参数方法调用
//! - class 属性 → 展开为多个样式方法
//! - 事件处理器 → 映射到正确的 GPUI 方法
//! - when/whenSome → 条件渲染方法
//!
//! 优化：
//! - 使用 match-based `lookup_attr_method()` 替代双重线性扫描
//! - 直接 push 到调用方 Vec，避免中间 Vec 分配

use super::class::{ClassMode, parse_class_string_with_mode};
use super::runtime::generate_dynamic_class_code_with_mode;
use super::tables::{is_stateful_class, lookup_attr_flag_method, lookup_attr_method_info};
use crate::parser::RsxAttribute;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

/// 属性生成阶段可复用的扫描结果，避免同一属性名或静态 class 字符串重复分配。
#[derive(Clone, Copy, Default)]
pub(crate) struct AttrHints<'a> {
    pub(crate) name: Option<&'a str>,
    pub(crate) static_class: Option<&'a str>,
    /// `onUnknownClass={handler}` on the same element, used by dynamic classes.
    pub(crate) unknown_class_handler: Option<&'a syn::Expr>,
}

/// Whether a statically expanded class expression uses a class that needs an element ID.
/// Includes supported static edges of format templates; fully dynamic tokens need no ID.
pub(crate) fn static_class_expr_needs_id(expr: &syn::Expr, mode: ClassMode) -> bool {
    if let Some(edges) = super::format_class::static_edges(expr, mode) {
        return edges
            .prefix
            .split_ascii_whitespace()
            .chain(edges.suffix.split_ascii_whitespace())
            .any(is_stateful_class);
    }
    static_class_literals(expr).is_some_and(|literals| {
        literals
            .iter()
            .any(|lit| lit.value().split_ascii_whitespace().any(is_stateful_class))
    })
}

pub(crate) fn generate_attr_methods_with_mode(
    attr: &RsxAttribute,
    hints: AttrHints<'_>,
    out: &mut Vec<TokenStream>,
    mode: ClassMode,
    dynamic_used: &mut bool,
) {
    match attr {
        // id / key / base 已在 generate_element 中处理，跳过避免重复生成方法调用
        RsxAttribute::Value { name, .. }
            if name == "id" || name == "key" || name == "base" || name == "onUnknownClass" => {}

        RsxAttribute::Flag(name) => {
            if name != "styled" {
                let name_storage;
                let name_str = if let Some(name) = hints.name {
                    name
                } else {
                    name_storage = name.to_string();
                    &name_storage
                };
                if name_str == "grayscale" {
                    out.push(quote! { .grayscale(true) });
                    return;
                }
                if let Some(mapped) = lookup_attr_flag_method(name_str) {
                    let method_ident = syn::Ident::new(mapped, name.span());
                    out.push(quote! { .#method_ident() });
                    return;
                }
                // styled 标志已在 generate_element 中处理，不生成 .styled()
                let method = snake_case_ident(name);
                out.push(quote! { .#method() });
            }
        }

        RsxAttribute::Value { name, value } => {
            // class 属性 → 展开为多个样式方法（静态）或运行时解析（动态）
            if name == "class" {
                // 情况 1：字符串字面量 → 编译期解析（最优性能）。调用方对字面量总会提供 hint。
                if let Some(s) = hints.static_class {
                    out.extend(parse_class_string_with_mode(s, mode, value.span()));
                    return;
                }

                // 情况 2：条件表达式的每个分支都是字符串字面量 → 分支内静态展开。
                // 这是常见写法，避免为 `if active { "..." } else { "..." }`
                // 生成完整动态 class matcher。
                if let Some(static_expr) = generate_static_class_expr_code(value, mode.clone()) {
                    out.push(quote! { .map(|__el| #static_expr) });
                    return;
                }

                // 情况 3：动态表达式 → 生成运行时解析代码
                *dynamic_used = true;
                let dynamic_code =
                    generate_dynamic_class_code_with_mode(value, mode, hints.unknown_class_handler);
                out.push(quote! { .map(|__el| #dynamic_code) });
                return;
            }

            if name == "visible" {
                out.push(quote! {
                    .map(|__el| {
                        let __visible = #value;
                        if __visible {
                            __el.visible()
                        } else {
                            __el.invisible()
                        }
                    })
                });
                return;
            }

            // 使用 match-based 查找替代原先的双重线性扫描
            let name_storage;
            let name_str = if let Some(name) = hints.name {
                name
            } else {
                name_storage = name.to_string();
                &name_storage
            };
            if let Some(info) = lookup_attr_method_info(name_str) {
                let method_ident = syn::Ident::new(info.method, name.span());
                if info.multi_arg
                    && let syn::Expr::Tuple(tuple) = value
                {
                    let args = &tuple.elems;
                    out.push(quote! { .#method_ident(#args) });
                } else {
                    out.push(quote! { .#method_ident(#value) });
                }
                return;
            }

            // 默认：直接作为方法调用（camelCase 转为 snake_case，如组件 props `elevatedIf`）
            let method = snake_case_ident(name);
            out.push(quote! { .#method(#value) });
        }

        // when 条件渲染
        RsxAttribute::When { condition, closure } => {
            out.push(quote! { .when(#condition, #closure) });
        }

        // when_some 条件渲染
        RsxAttribute::WhenSome { option, closure } => {
            out.push(quote! { .when_some(#option, #closure) });
        }

        // whenClass 条件样式，仅支持静态 class 字符串。
        RsxAttribute::WhenClass {
            condition,
            class_lit,
        } => {
            let class_str = class_lit.value();
            let class_methods = parse_class_string_with_mode(&class_str, mode, class_lit.span());
            out.push(quote! { .when(#condition, |__el| __el #(#class_methods)* ) });
        }

        // GPUI state-style helpers. These methods receive StyleRefinement, which implements
        // Styled in real GPUI, so the same static class expansion can be reused here.
        RsxAttribute::StateClass { method, class_lit } => {
            let class_str = class_lit.value();
            let class_methods = parse_class_string_with_mode(&class_str, mode, class_lit.span());
            out.push(quote! { .#method(|__style| __style #(#class_methods)* ) });
        }
    }
}

/// Literal leaves of an expression that the macro can expand statically.
/// Conditions, match patterns/guards and strings inside runtime calls are excluded.
pub fn static_class_literals(expr: &syn::Expr) -> Option<Vec<&syn::LitStr>> {
    fn block(block: &syn::Block) -> Option<Vec<&syn::LitStr>> {
        static_class_literals(block_tail_expr(block)?)
    }
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) => Some(vec![lit]),
        syn::Expr::If(expr) => {
            let mut literals = block(&expr.then_branch)?;
            literals.extend(static_class_literals(&expr.else_branch.as_ref()?.1)?);
            Some(literals)
        }
        syn::Expr::Match(expr) => {
            let mut literals = Vec::new();
            for arm in &expr.arms {
                literals.extend(static_class_literals(&arm.body)?);
            }
            Some(literals)
        }
        syn::Expr::Paren(expr) => static_class_literals(&expr.expr),
        syn::Expr::Group(expr) => static_class_literals(&expr.expr),
        syn::Expr::Block(expr) => block(&expr.block),
        _ => None,
    }
}

fn generate_static_class_expr_code(expr: &syn::Expr, mode: ClassMode) -> Option<TokenStream> {
    match expr {
        syn::Expr::If(expr_if) => {
            let condition = &expr_if.cond;
            let then_code = generate_static_class_block_code(&expr_if.then_branch, mode.clone())?;
            let (_, else_expr) = expr_if.else_branch.as_ref()?;
            let else_code = generate_static_class_expr_code(else_expr, mode)?;
            Some(quote! {
                if #condition {
                    #then_code
                } else {
                    #else_code
                }
            })
        }
        syn::Expr::Match(expr_match) => {
            let expr = &expr_match.expr;
            let arms = expr_match
                .arms
                .iter()
                .map(|arm| {
                    let attrs = &arm.attrs;
                    let pat = &arm.pat;
                    let guard = if let Some((if_token, guard_expr)) = &arm.guard {
                        quote! { #if_token #guard_expr }
                    } else {
                        quote! {}
                    };
                    let body = generate_static_class_expr_code(&arm.body, mode.clone())?;
                    Some(quote! {
                        #(#attrs)*
                        #pat #guard => #body,
                    })
                })
                .collect::<Option<Vec<_>>>()?;
            Some(quote! {
                match #expr {
                    #(#arms)*
                }
            })
        }
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => {
            let class_str = lit_str.value();
            let methods = parse_class_string_with_mode(&class_str, mode, lit_str.span());
            Some(quote! { __el #(#methods)* })
        }
        syn::Expr::Paren(expr) => generate_static_class_expr_code(&expr.expr, mode),
        syn::Expr::Group(expr) => generate_static_class_expr_code(&expr.expr, mode),
        syn::Expr::Block(expr) => generate_static_class_block_code(&expr.block, mode),
        _ => None,
    }
}

fn generate_static_class_block_code(block: &syn::Block, mode: ClassMode) -> Option<TokenStream> {
    let expr = block_tail_expr(block)?;
    generate_static_class_expr_code(expr, mode)
}

fn block_tail_expr(block: &syn::Block) -> Option<&syn::Expr> {
    match block.stmts.as_slice() {
        [syn::Stmt::Expr(expr, None)] => Some(expr),
        _ => None,
    }
}

/// `elevatedIf` → `elevated_if`; snake_case names are returned unchanged.
fn snake_case_ident(name: &syn::Ident) -> syn::Ident {
    use syn::ext::IdentExt;
    // Convert the unraw text: `Ident::new` rejects a `r#` prefix.
    let text = name.unraw().to_string();
    if !text.bytes().any(|b| b.is_ascii_uppercase()) {
        return name.clone();
    }
    let mut snake = String::with_capacity(text.len() + 4);
    for (i, c) in text.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                snake.push('_');
            }
            snake.push(c.to_ascii_lowercase());
        } else {
            snake.push(c);
        }
    }
    if name.to_string().starts_with("r#") {
        syn::Ident::new_raw(&snake, name.span())
    } else {
        syn::Ident::new(&snake, name.span())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::element::generate_body_with_mode;

    fn expand(source: &str) -> String {
        let body = syn::parse_str::<crate::parser::RsxBody>(source).expect("valid RSX body");
        generate_body_with_mode(&body, ClassMode::PERMISSIVE).to_string()
    }

    #[test]
    fn snake_case_ident_handles_camel_raw_and_plain_names() {
        let ident = |source: &str| -> syn::Ident { syn::parse_str(source).unwrap() };
        assert_eq!(
            snake_case_ident(&ident("elevatedIf")).to_string(),
            "elevated_if"
        );
        assert_eq!(snake_case_ident(&ident("plain")).to_string(), "plain");
        // Raw identifiers keep their `r#` prefix; `Ident::new` would panic on it.
        assert_eq!(
            snake_case_ident(&ident("r#myAttr")).to_string(),
            "r#my_attr"
        );
        assert_eq!(snake_case_ident(&ident("r#type")).to_string(), "r#type");
    }

    #[test]
    fn raw_identifier_attributes_expand_without_panicking() {
        assert_eq!(expand("<div r#myAttr={1} />"), "div () . r#my_attr (1)");
        assert_eq!(expand("<div r#onFoo />"), "div () . r#on_foo ()");
        // A raw keyword keeps working as a method name.
        assert_eq!(expand(r#"<div r#type="x" />"#), r#"div () . r#type ("x")"#);
    }

    #[test]
    fn grayscale_flag_takes_a_boolean_argument() {
        assert_eq!(expand("<div grayscale />"), "div () . grayscale (true)");
    }

    #[test]
    fn static_class_branches_expand_each_arm_at_compile_time() {
        // `if` / `else` branches become one `.map` with a static expansion per branch.
        let conditional = expand(r#"<div class={if active {"flex"} else {"grid"}} />"#);
        assert!(conditional.contains(". map ("), "{conditional}");
        assert!(conditional.contains(". flex ()"), "{conditional}");
        assert!(conditional.contains(". grid ()"), "{conditional}");
        assert!(!conditional.contains("__rsx_apply_class"), "{conditional}");

        // `match` arms (including guards and attributes) keep their patterns.
        let matched =
            expand(r#"<div class={match size { 0 => "p-0", n if n > 4 => "p-4", _ => "p-2" }} />"#);
        assert!(matched.contains("match size"), "{matched}");
        assert!(matched.contains("if n > 4"), "{matched}");
        assert!(matched.contains(". p (px (0f32))"), "{matched}");
        assert!(matched.contains(". p (px (4f32))"), "{matched}");
        assert!(matched.contains(". p (px (2f32))"), "{matched}");

        // Parentheses, groups and plain blocks are unwrapped before expansion.
        for source in [
            r#"<div class={("flex")} />"#,
            r#"<div class={{ "flex" }} />"#,
            r#"<div class={("grid")} />"#,
        ] {
            let expanded = expand(source);
            assert!(
                !expanded.contains("__rsx_apply_class"),
                "{source}: {expanded}"
            );
            assert!(
                expanded.contains(". flex ()") || expanded.contains(". grid ()"),
                "{source}: {expanded}"
            );
        }
    }

    #[test]
    fn dynamic_class_expressions_fall_back_to_the_runtime_matcher() {
        // A non-literal expression cannot be expanded statically.
        let dynamic = expand("<div class={classes} />");
        assert!(dynamic.contains("__rsx_apply_class"), "{dynamic}");

        // A branch mixing literals with a runtime call is dynamic as a whole.
        let mixed = expand(r#"<div class={if active {"flex"} else {compute()}} />"#);
        assert!(mixed.contains("__rsx_apply_class"), "{mixed}");
    }

    #[test]
    fn dynamic_helper_is_shared_across_children_loops_and_fragments() {
        for source in [
            "<div class={outer}><div class={inner}/></div>",
            "<><div class={a}/><div class={b}/></>",
            "<div class={outer}>{for item in items { <div class={item.classes}/> }}</div>",
        ] {
            let expanded = expand(source);
            assert_eq!(
                expanded.matches("fn __rsx_apply_class").count(),
                1,
                "{source}"
            );
            assert_eq!(
                expanded.matches("fn __rsx_unknown_class_handler").count(),
                2,
                "{source}"
            );
            syn::parse_str::<syn::Expr>(&expanded).unwrap();
        }
    }

    #[test]
    fn static_class_literals_collect_every_leaf() {
        let literals = |source: &str| -> Option<Vec<String>> {
            let expr: syn::Expr = syn::parse_str(source).unwrap();
            static_class_literals(&expr).map(|lits| lits.iter().map(|l| l.value()).collect())
        };
        assert_eq!(literals(r#""flex""#).unwrap(), ["flex"]);
        assert_eq!(
            literals(r#"if a {"flex"} else {"grid"}"#).unwrap(),
            ["flex", "grid"]
        );
        assert_eq!(
            literals(r#"match n { 0 => "p-0", _ => "p-2" }"#).unwrap(),
            ["p-0", "p-2"]
        );
        assert_eq!(literals(r#"("flex")"#).unwrap(), ["flex"]);
        assert_eq!(literals(r#"{ "flex" }"#).unwrap(), ["flex"]);
        // Missing else branches and non-literals are not static.
        assert_eq!(literals(r#"if a {"flex"}"#), None);
        assert_eq!(literals("compute()"), None);
    }

    #[test]
    fn static_class_expressions_detect_stateful_classes() {
        let needs_id = |source: &str| -> Option<bool> {
            let expr: syn::Expr = syn::parse_str(source).unwrap();
            static_class_expr_needs_id(&expr, ClassMode::PERMISSIVE).into()
        };
        assert_eq!(needs_id(r#""flex""#), Some(false));
        assert_eq!(needs_id(r#""active:opacity-75""#), Some(true));
        assert_eq!(
            needs_id(r#"if a {"flex"} else {"active:opacity-75"}"#),
            Some(true)
        );
        assert_eq!(
            needs_id(r#"match n { 0 => "flex", _ => "grid" }"#),
            Some(false)
        );
        assert_eq!(
            needs_id(r#"if a {"active:opacity-75"} else {"flex"}"#),
            Some(true)
        );
        assert_eq!(needs_id("classes"), Some(false));
        // Partly dynamic expressions expand at runtime, which needs no ID.
        assert_eq!(
            needs_id(r#"if a {"active:opacity-75"} else {classes}"#),
            Some(false)
        );

        // Stateful branches inside an element still trigger an auto ID.
        let expanded = expand(r#"<div class={if a {"flex"} else {"active:opacity-75"}} />"#);
        assert!(expanded.contains(". id ("), "{expanded}");
    }

    #[test]
    fn block_tail_expr_requires_a_single_trailing_expression() {
        let tail = |source: &str| {
            let block: syn::Block = syn::parse_str(source).unwrap();
            block_tail_expr(&block).map(|e| quote! { #e }.to_string())
        };
        assert_eq!(tail(r#"{ "flex" }"#).as_deref(), Some("\"flex\""));
        // Only a single trailing expression counts; statements make it dynamic.
        assert_eq!(tail(r#"{ let x = 1; "flex" }"#), None);
        assert_eq!(tail("{ let x = 1; }"), None);
    }

    #[test]
    fn when_and_state_class_attributes_reuse_static_expansion() {
        let when = expand(r#"<div whenClass={(active, "p-4 text-white")} />"#);
        assert!(when.contains(". when (active"), "{when}");
        assert!(when.contains(". p (px (4f32))"), "{when}");
        assert!(when.contains(". text_color"), "{when}");

        let state = expand(r#"<div hoverClass="bg-blue-500" />"#);
        assert!(state.contains(". hover (| __style |"), "{state}");
        assert!(state.contains("__style . bg"), "{state}");
    }

    #[test]
    fn visible_value_attribute_uses_a_boolean_branch() {
        let expanded = expand("<div visible={is_visible} />");
        assert!(expanded.contains(". visible ()"), "{expanded}");
        assert!(expanded.contains(". invisible ()"), "{expanded}");
        assert!(expanded.contains("if __visible"), "{expanded}");
    }

    #[test]
    fn multi_argument_attributes_expand_tuple_elements() {
        let expanded = expand("<div on_mouse_down={(button, handler)} />");
        assert!(
            expanded.contains(". on_mouse_down (button , handler)"),
            "{expanded}"
        );
    }

    #[test]
    fn click_attributes_support_both_gpui_signatures() {
        for attr in ["onClick", "on_click"] {
            for (value, expected) in [
                ("(ranges, handler)", ". on_click (ranges , handler)"),
                ("handler", ". on_click (handler)"),
                ("(handler)", ". on_click ((handler))"),
                ("|_, _, _| {}", ". on_click (| _ , _ , _ | { })"),
            ] {
                let expanded = expand(&format!("<div {attr}={{{value}}} />"));
                assert!(expanded.contains(expected), "{attr}={value}: {expanded}");
            }
        }
    }

    #[test]
    fn single_argument_attributes_preserve_tuple_values() {
        for attr in ["bg", "customProp"] {
            let expanded = expand(&format!("<div {attr}={{(first, second)}} />"));
            assert!(expanded.contains("((first , second))"), "{expanded}");
        }
    }

    #[test]
    fn id_key_base_and_unknown_handler_attributes_are_not_methods() {
        // These are consumed by element generation, so no stray `.id()`/`.key()` appears twice.
        let expanded =
            expand(r#"<div id="x" key={k} base={custom} onUnknownClass={|c| report(c)} />"#);
        assert_eq!(expanded.matches(". id (").count(), 1, "{expanded}");
        assert!(!expanded.contains(". key ("), "{expanded}");
        assert!(!expanded.contains(". base ("), "{expanded}");
        assert!(!expanded.contains(". on_unknown_class ("), "{expanded}");
    }
}
