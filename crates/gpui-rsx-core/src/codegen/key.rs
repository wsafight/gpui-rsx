//! Borrow keys once; concrete integers use GPUI's allocation-free named integer IDs.
use proc_macro2::TokenStream;
use quote::quote;

use crate::parser::RsxElementName;

pub(super) fn make_auto_id(tag_name: &RsxElementName) -> TokenStream {
    let location = tag_name.span().start();
    let id_suffix = format!("__rsx_{}_L{}C{}", tag_name, location.line, location.column);
    quote! { concat!(file!(), "::", #id_suffix) }
}

pub(super) fn make_keyed_auto_id(
    tag_name: &RsxElementName,
    key_expr: &syn::Expr,
    gpui: &syn::Path,
) -> TokenStream {
    let location = tag_name.span().start();
    let prefix_suffix = format!(
        "::__rsx_{}_L{}C{}_",
        tag_name, location.line, location.column
    );
    if let Some(static_suffix) = static_key_suffix(key_expr) {
        return quote! { concat!(file!(), #prefix_suffix, #static_suffix) };
    }
    dynamic_key(&prefix_suffix, key_expr, gpui)
}

pub(super) fn static_key_suffix(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(literal),
            ..
        }) => Some(literal.value()),
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(literal),
            ..
        }) => literal
            .base10_parse::<u128>()
            .ok()
            .map(|value| value.to_string()),
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Bool(literal),
            ..
        }) => Some(literal.value.to_string()),
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Char(literal),
            ..
        }) => Some(literal.value().to_string()),
        syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => match &*unary.expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(literal),
                ..
            }) => literal
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

pub(super) fn dynamic_key(prefix: &str, expr: &syn::Expr, gpui: &syn::Path) -> TokenStream {
    quote! {{
        struct __RsxKey<'a, T: ?Sized>(&'a T);
        trait __RsxKeyId {
            type Id;
            fn __rsx_key_id(self, prefix: &'static str) -> Self::Id;
        }
        // Autoref fallback preserves every Display key without moving its value.
        impl<T: ::core::fmt::Display + ?Sized> __RsxKeyId for &__RsxKey<'_, T> {
            type Id = #gpui::SharedString;
            fn __rsx_key_id(self, prefix: &'static str) -> Self::Id {
                #gpui::SharedString::from(::std::format!("{}{}", prefix, self.0))
            }
        }
        // Let rustc repeat the identical impl rather than emitting its token tree ten times.
        macro_rules! __rsx_integer_keys {
            ($($ty:ty),*) => {$(
                impl __RsxKeyId for __RsxKey<'_, $ty> {
                    type Id = (&'static str, u64);
                    fn __rsx_key_id(self, prefix: &'static str) -> Self::Id {
                        (prefix, *self.0 as u64)
                    }
                }
            )*};
        }
        __rsx_integer_keys!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
        __RsxKey(&(#expr)).__rsx_key_id(concat!(file!(), #prefix))
    }}
}
