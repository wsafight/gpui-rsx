//! Attribute handling
//!
//! Converts RSX attributes to GPUI method calls:
//! - Flag attributes → parameterless method calls
//! - Value attributes → method calls with arguments
//! - class attribute → expanded into multiple style methods
//! - Event handlers → mapped to the correct GPUI methods
//! - when/whenSome → conditional rendering methods
//!
//! Optimizations:
//! - Use match-based `lookup_attr_method()` instead of double linear scan
//! - Push directly to caller Vec to avoid intermediate Vec allocations

use super::class::{ClassMode, parse_class_string_with_mode};
use super::runtime::generate_dynamic_class_code_with_mode;
use super::tables::{is_stateful_class, lookup_attr_flag_method, lookup_attr_method_info};
use crate::parser::RsxAttribute;
use proc_macro2::TokenStream;
use quote::quote;

/// Reusable scan results during attribute generation stage to avoid repeated allocations for the same attribute name or static class string.
#[derive(Clone, Copy, Default)]
pub(crate) struct AttrHints<'a> {
    pub(crate) name: Option<&'a str>,
    pub(crate) static_class: Option<&'a str>,
}

pub(crate) fn static_class_expr_needs_id(expr: &syn::Expr) -> bool {
    static_class_expr_has_stateful_class(expr).unwrap_or(false)
}

pub(crate) fn generate_attr_methods_with_mode(
    attr: &RsxAttribute,
    hints: AttrHints<'_>,
    out: &mut Vec<TokenStream>,
    mode: ClassMode,
) {
    match attr {
        // id / key / base are already handled in generate_element; skip to avoid duplicate method calls
        RsxAttribute::Value { name, .. } if name == "id" || name == "key" || name == "base" => {}

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
                // styled flag is already handled in generate_element; do not generate .styled()
                out.push(quote! { .#name() });
            }
        }

        RsxAttribute::Value { name, value } => {
            // class attribute → expanded to multiple style methods (static) or runtime parsing (dynamic)
            if name == "class" {
                // Case 1: String literal → compile-time parsing (optimal performance)
                if let Some(s) = hints.static_class {
                    out.extend(parse_class_string_with_mode(s, mode));
                    return;
                }

                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = value
                {
                    let s = lit_str.value();
                    out.extend(parse_class_string_with_mode(&s, mode));
                    return;
                }

                // Case 2: Each branch of a conditional expression is a string literal → statically expand within branches.
                // This is a common pattern, avoiding generating a full dynamic class matcher for `if active { "..." } else { "..." }`.
                if let Some(static_expr) = generate_static_class_expr_code(value, mode) {
                    out.push(quote! { .map(|__el| #static_expr) });
                    return;
                }

                // Case 3: Dynamic expression → generate runtime parsing code
                let dynamic_code = generate_dynamic_class_code_with_mode(value, mode);
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

            // Use match-based lookup instead of the original double linear scan
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

            if name_str == "ref" {
                out.push(quote! { .track_focus(#value) });
                return;
            }

            // Default: call directly as a method
            out.push(quote! { .#name(#value) });
        }

        // when conditional rendering
        RsxAttribute::When { condition, closure } => {
            out.push(quote! { .when(#condition, #closure) });
        }

        // when_some conditional rendering
        RsxAttribute::WhenSome { option, closure } => {
            out.push(quote! { .when_some(#option, #closure) });
        }

        // whenClass conditional styling, only supports static class strings.
        RsxAttribute::WhenClass {
            condition,
            class_lit,
        } => {
            let class_str = class_lit.value();
            let class_methods: Vec<_> = parse_class_string_with_mode(&class_str, mode).collect();
            out.push(quote! { .when(#condition, |__el| __el #(#class_methods)* ) });
        }

        // GPUI state-style helpers. These methods receive StyleRefinement, which implements
        // Styled in real GPUI, so the same static class expansion can be reused here.
        RsxAttribute::StateClass { method, class_lit } => {
            let class_str = class_lit.value();
            let class_methods: Vec<_> = parse_class_string_with_mode(&class_str, mode).collect();
            out.push(quote! { .#method(|__style| __style #(#class_methods)* ) });
        }
    }
}

fn generate_static_class_expr_code(expr: &syn::Expr, mode: ClassMode) -> Option<TokenStream> {
    match expr {
        syn::Expr::If(expr_if) => {
            let condition = &expr_if.cond;
            let then_code = generate_static_class_block_code(&expr_if.then_branch, mode)?;
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
                    let body = generate_static_class_expr_code(&arm.body, mode)?;
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
            let methods: Vec<_> = parse_class_string_with_mode(&class_str, mode).collect();
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

fn static_class_expr_has_stateful_class(expr: &syn::Expr) -> Option<bool> {
    match expr {
        syn::Expr::If(expr_if) => {
            if static_class_block_has_stateful_class(&expr_if.then_branch)? {
                return Some(true);
            }
            let (_, else_expr) = expr_if.else_branch.as_ref()?;
            static_class_expr_has_stateful_class(else_expr)
        }
        syn::Expr::Match(expr_match) => {
            for arm in &expr_match.arms {
                if static_class_expr_has_stateful_class(&arm.body)? {
                    return Some(true);
                }
            }
            Some(false)
        }
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => {
            let class = lit_str.value();
            Some(class.split_ascii_whitespace().any(is_stateful_class))
        }
        syn::Expr::Paren(expr) => static_class_expr_has_stateful_class(&expr.expr),
        syn::Expr::Group(expr) => static_class_expr_has_stateful_class(&expr.expr),
        syn::Expr::Block(expr) => static_class_block_has_stateful_class(&expr.block),
        _ => None,
    }
}

fn static_class_block_has_stateful_class(block: &syn::Block) -> Option<bool> {
    let expr = block_tail_expr(block)?;
    static_class_expr_has_stateful_class(expr)
}

fn block_tail_expr(block: &syn::Block) -> Option<&syn::Expr> {
    match block.stmts.as_slice() {
        [syn::Stmt::Expr(expr, None)] => Some(expr),
        _ => None,
    }
}
