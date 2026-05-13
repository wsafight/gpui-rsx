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
use super::tables::{is_multi_arg_method, lookup_attr_flag_method, lookup_attr_method};
use crate::parser::RsxAttribute;
use proc_macro2::TokenStream;
use quote::quote;

/// 属性生成阶段可复用的扫描结果，避免同一属性名或静态 class 字符串重复分配。
#[derive(Clone, Copy, Default)]
pub(crate) struct AttrHints<'a> {
    pub(crate) name: Option<&'a str>,
    pub(crate) static_class: Option<&'a str>,
}

pub(crate) fn generate_attr_methods_with_mode(
    attr: &RsxAttribute,
    hints: AttrHints<'_>,
    out: &mut Vec<TokenStream>,
    mode: ClassMode,
) {
    match attr {
        // id / key 已在 generate_element 中处理，跳过避免重复生成方法调用
        RsxAttribute::Value { name, .. } if name == "id" || name == "key" => {}

        RsxAttribute::Flag(name) => {
            if name != "styled" {
                let name_storage;
                let name_str = if let Some(name) = hints.name {
                    name
                } else {
                    name_storage = name.to_string();
                    &name_storage
                };
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

                // 情况 2：动态表达式 → 生成运行时解析代码
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
    }
}
