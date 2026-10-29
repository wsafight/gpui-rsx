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

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(method: &'static str, value: Option<StyleValue>) -> String {
        StyleOperation::new(method, value).to_tokens().to_string()
    }

    #[test]
    fn accessors_expose_the_operation_and_its_value() {
        let operation = StyleOperation::new("w", Some(StyleValue::Length(StyleLength::Px(12.0))));
        assert_eq!(operation.method(), "w");
        assert_eq!(
            operation.value(),
            Some(&StyleValue::Length(StyleLength::Px(12.0)))
        );
        assert_eq!(StyleOperation::new("flex", None).value(), None);
    }

    #[test]
    fn lengths_use_their_own_gpui_constructors() {
        assert_eq!(
            tokens("w", Some(StyleValue::Length(StyleLength::Px(4.0)))),
            ". w (px (4f32))"
        );
        assert_eq!(
            tokens("w", Some(StyleValue::Length(StyleLength::Rem(0.75)))),
            ". w (rems (0.75f32))"
        );
        assert_eq!(
            tokens("w", Some(StyleValue::Length(StyleLength::Relative(0.5)))),
            ". w (relative (0.5f32))"
        );
    }

    #[test]
    fn colors_choose_rgb_or_rgba_from_alpha() {
        assert_eq!(
            tokens(
                "bg",
                Some(StyleValue::Color {
                    hex: 0x3b82f6,
                    alpha: false
                })
            ),
            ". bg (rgb (3900150u32))"
        );
        assert_eq!(
            tokens(
                "bg",
                Some(StyleValue::Color {
                    hex: 0x3b82f680,
                    alpha: true
                })
            ),
            ". bg (rgba (998438528u32))"
        );
    }

    #[test]
    fn finite_numbers_are_emitted_and_others_become_compile_errors() {
        assert_eq!(
            tokens("opacity", Some(StyleValue::Number(0.5))),
            ". opacity (0.5f32)"
        );
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let error = tokens("opacity", Some(StyleValue::Number(value)));
            assert!(error.contains("compile_error"), "{error}");
            assert!(error.contains("style number must be finite"), "{error}");
        }
    }

    #[test]
    fn expressions_are_passed_through_as_rust() {
        assert_eq!(
            tokens("w", Some(StyleValue::Expression("px(width)".into()))),
            ". w (px (width))"
        );
        let error = tokens("w", Some(StyleValue::Expression("((".into())));
        assert!(error.contains("compile_error"), "{error}");
        assert!(error.contains("invalid style expression"), "{error}");
    }

    #[test]
    fn valueless_operations_emit_plain_method_calls() {
        assert_eq!(tokens("flex", None), ". flex ()");
    }
}
