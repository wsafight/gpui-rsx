//! Resolved style operations shared by GPUI code generation and static previews.
//! Expressions remain opaque: inspecting an operation never executes user Rust.
use proc_macro2::{Span, TokenStream};
use quote::quote;

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum StyleLength {
    Px(f32),
    Rem(f32),
    Relative(f32),
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum StyleValue {
    Length(StyleLength),
    Color {
        hex: u32,
        alpha: bool,
    },
    Number(f32),
    /// An opaque Rust expression. A preview may recognize literal px/rems calls,
    /// but must never evaluate the expression.
    Expression(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleOperation {
    method: &'static str,
    value: Option<StyleValue>,
}

impl StyleOperation {
    pub(crate) fn new(method: &'static str, value: Option<StyleValue>) -> Self {
        Self { method, value }
    }

    pub const fn method(&self) -> &'static str {
        self.method
    }
    pub fn value(&self) -> Option<&StyleValue> {
        self.value.as_ref()
    }

    /// Generates the same method chain fragment used by the macro.
    pub fn to_tokens(&self) -> TokenStream {
        let ident = syn::Ident::new(self.method, Span::call_site());
        let Some(value) = &self.value else {
            return quote! { .#ident() };
        };
        let value = match value {
            StyleValue::Length(StyleLength::Px(value)) => quote! { px(#value) },
            StyleValue::Length(StyleLength::Rem(value)) => quote! { rems(#value) },
            StyleValue::Length(StyleLength::Relative(value)) => quote! { relative(#value) },
            StyleValue::Color { hex, alpha: false } => quote! { rgb(#hex) },
            StyleValue::Color { hex, alpha: true } => quote! { rgba(#hex) },
            StyleValue::Number(value) if value.is_finite() => quote! { #value },
            StyleValue::Number(_) => {
                return syn::Error::new(Span::call_site(), "style number must be finite")
                    .to_compile_error();
            }
            StyleValue::Expression(expr) => match expr.parse::<TokenStream>() {
                Ok(tokens) => tokens,
                Err(error) => {
                    return syn::Error::new(
                        Span::call_site(),
                        format!("invalid style expression: {error}"),
                    )
                    .to_compile_error();
                }
            },
        };
        quote! { .#ident(#value) }
    }
}
