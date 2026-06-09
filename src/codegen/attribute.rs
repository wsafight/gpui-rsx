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
use super::tables::{
    is_multi_arg_method, is_stateful_class, lookup_attr_flag_method, lookup_attr_method,
};
use crate::parser::RsxAttribute;
use proc_macro2::TokenStream;
use quote::quote;

/// 属性生成阶段可复用的扫描结果，避免同一属性名或静态 class 字符串重复分配。
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
        // id / key / base 已在 generate_element 中处理，跳过避免重复生成方法调用
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
                // styled 标志已在 generate_element 中处理，不生成 .styled()
                out.push(quote! { .#name() });
            }
        }

        RsxAttribute::Value { name, value } => {
            // class 属性 → 展开为多个样式方法（静态）或运行时解析（动态）
            if name == "class" {
                // 情况 1：字符串字面量 → 编译期解析（最优性能）
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

                // 情况 2：条件表达式的每个分支都是字符串字面量 → 分支内静态展开。
                // 这是常见写法，避免为 `if active { "..." } else { "..." }`
                // 生成完整动态 class matcher。
                if let Some(static_expr) = generate_static_class_expr_code(value, mode) {
                    out.push(quote! { .map(|__el| #static_expr) });
                    return;
                }

                // 情况 3：动态表达式 → 生成运行时解析代码
                let dynamic_code = generate_dynamic_class_code_with_mode(value, mode);
                out.push(quote! { .map(|__el| #dynamic_code) });
                return;
            }

            if name == "visible" {
                out.push(quote! { .when(#value, |__el| __el.visible()).when(!(#value), |__el| __el.invisible()) });
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
            if let Some(mapped) = lookup_attr_method(name_str) {
                let method_ident = syn::Ident::new(mapped, name.span());
                if is_multi_arg_method(mapped)
                    && let syn::Expr::Tuple(tuple) = value
                {
                    let args = &tuple.elems;
                    out.push(quote! { .#method_ident(#args) });
                } else {
                    out.push(quote! { .#method_ident(#value) });
                }
                return;
            }

            // 默认：直接作为方法调用
            out.push(quote! { .#name(#value) });
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
            let class_methods: Vec<_> = parse_class_string_with_mode(&class_str, mode).collect();
            out.push(quote! { .when(#condition, |__el| __el #(#class_methods)* ) });
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
