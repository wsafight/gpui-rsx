//! `#[component]`: typestate props builders for RSX components.
//!
//! ```text
//! #[component]
//! #[derive(IntoElement)]
//! struct Card {
//!     title: SharedString,                 // required
//!     #[prop(default)] elevated: bool,     // optional flag
//!     #[prop(default = 2)] depth: u8,      // optional with default
//!     #[children] children: Vec<AnyElement>,
//! }
//!
//! rsx! { <Card title="Inbox" elevated>{"3 unread"}</Card> }
//! // → Card().title("Inbox").elevated().child("3 unread")
//! ```
//!
//! A struct with named fields lives in the type namespace only, so the macro can add a
//! function with the same name (`Card()`) in the value namespace. RSX already expands
//! `<Card ...>` to `Card()`, so components need no special syntax.
//!
//! Each required prop is a type parameter on the builder (`MissingRequiredProp` → `Set`). The builder
//! implements `IntoElement` only when every required prop is `Set`; the per-prop marker
//! traits carry `#[diagnostic::on_unimplemented]` so a missing prop is reported by name.

mod builder;
mod callback;
mod function;
#[cfg(test)]
mod tests;

use builder::expand_struct;
use function::expand_function;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

/// Expands `#[component]`. Errors are returned as `compile_error!` tokens.
pub fn expand_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    match expand(attr, item) {
        Ok(tokens) => tokens,
        Err(error) => error.to_compile_error(),
    }
}

fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !attr.is_empty() {
        return Err(syn::Error::new(
            attr.span(),
            "#[component] takes no arguments; configure props with #[prop(...)] and #[children]",
        ));
    }
    let input = syn::parse2::<syn::Item>(item)?;
    let gates: Vec<_> = match &input {
        syn::Item::Struct(input) => &input.attrs,
        syn::Item::Fn(input) => &input.attrs,
        _ => &[][..],
    }
    .iter()
    .filter(|attr| attr.path().is_ident("cfg"))
    .cloned()
    .collect();
    // Every generated item, including builders and impls, is gated along with its source.
    match input {
        syn::Item::Struct(input) => expand_struct(input, &gates),
        syn::Item::Fn(input) => expand_function(input, &gates),
        input => Err(syn::Error::new_spanned(
            input,
            "#[component] requires a struct with named fields or a synchronous function",
        )),
    }
}
