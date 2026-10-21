//! `#[derive(IntoElement)]` for the GPUI stub. The stub covers `RenderOnce` types with a
//! blanket impl, so the derive only has to exist.

use proc_macro::TokenStream;

#[proc_macro_derive(IntoElement)]
pub fn derive_into_element(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
