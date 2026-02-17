//! 属性处理
//!
//! 将 RSX 属性转换为 GPUI 方法调用：
//! - Flag 属性 → 无参方法调用
//! - Value 属性 → 带参数方法调用
//! - class 属性 → 展开为多个样式方法
//! - 事件处理器 → 映射到正确的 GPUI 方法
//! - when/whenSome → 条件渲染方法

use super::class::parse_class_string;
use super::tables::*;
use crate::parser::RsxAttribute;
use proc_macro2::TokenStream;
use quote::quote;

/// 生成属性的方法链片段（返回 `.method(args)` 列表）
pub(crate) fn generate_attr_methods(attr: &RsxAttribute) -> Vec<TokenStream> {
    match attr {
        // id 已在 generate_base 中处理，跳过避免重复
        RsxAttribute::Value { name, .. } if name == "id" => vec![],

        RsxAttribute::Flag(name) => {
            // invisible 标志特殊处理 → .visible(false)
            if name == "invisible" {
                vec![quote! { .visible(false) }]
            } else if name == "styled" {
                // styled 标志已在 generate_element 中处理，不生成 .styled()
                vec![]
            } else {
                vec![quote! { .#name() }]
            }
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
                // class 属性仅支持字符串字面量，动态值需要用独立属性
                return vec![
                    quote! { compile_error!("class attribute only supports string literals; use individual attributes (e.g. flex, bg={...}) for dynamic styling") },
                ];
            }

            // 事件处理器查表
            for &(camel, snake, method) in EVENT_HANDLERS {
                if method_name == camel || method_name == snake {
                    let method_ident = syn::Ident::new(method, name.span());
                    return vec![quote! { .#method_ident(#value) }];
                }
            }

            // 属性名称映射查表（camelCase -> snake_case）
            for &(camel, snake) in ATTRIBUTE_NAME_MAP {
                if method_name == camel {
                    let method_ident = syn::Ident::new(snake, name.span());
                    return vec![quote! { .#method_ident(#value) }];
                }
            }

            // 默认：直接作为方法调用
            vec![quote! { .#name(#value) }]
        }

        // when 条件渲染
        RsxAttribute::When { condition, closure } => {
            vec![quote! { .when(#condition, #closure) }]
        }

        // when_some 条件渲染
        RsxAttribute::WhenSome { option, closure } => {
            vec![quote! { .when_some(#option, #closure) }]
        }
    }
}
