use proc_macro2::TokenStream;
use quote::quote;

enum ElementMethod {
    Chain(TokenStream),
    Inline {
        element: syn::Ident,
        body: TokenStream,
    },
}

/// Ordered method fragments. Inline blocks bind the current receiver without putting user
/// expressions inside a generated closure, then continue the ordinary method chain.
pub(crate) struct ElementMethods {
    fragments: Vec<ElementMethod>,
}

impl ElementMethods {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            fragments: Vec::with_capacity(capacity),
        }
    }

    pub(crate) fn push(&mut self, tokens: TokenStream) {
        self.fragments.push(ElementMethod::Chain(tokens));
    }

    pub(crate) fn push_inline(&mut self, element: syn::Ident, body: TokenStream) {
        self.fragments.push(ElementMethod::Inline { element, body });
    }

    pub(crate) fn extend(&mut self, tokens: impl IntoIterator<Item = TokenStream>) {
        self.fragments
            .extend(tokens.into_iter().map(ElementMethod::Chain));
    }

    pub(crate) fn finish(self, base: TokenStream) -> TokenStream {
        let mut expression = base;
        for method in self.fragments {
            match method {
                ElementMethod::Chain(tokens) => expression.extend(tokens),
                ElementMethod::Inline { element, body } => {
                    expression = quote! {{
                        let #element = #expression;
                        #body
                    }};
                }
            }
        }
        expression
    }
}
