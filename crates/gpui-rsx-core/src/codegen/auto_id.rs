//! Resolve auto IDs at the call site without wrapping component event setters.
use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn with_auto_id(element: TokenStream, id: TokenStream) -> TokenStream {
    quote! {{
        struct __RsxAutoId<T>(::core::cell::Cell<::core::option::Option<T>>);
        trait __RsxWithAutoId {
            type Output;
            fn __rsx_with_auto_id<I: ::core::convert::Into<::gpui::ElementId>>(
                self,
                id: I,
            ) -> Self::Output;
        }
        // Method lookup prefers the value receiver when Stateful<T> supports GPUI events.
        impl<T: ::gpui::InteractiveElement + ::gpui::Element> __RsxWithAutoId for __RsxAutoId<T> {
            type Output = ::gpui::Stateful<T>;
            fn __rsx_with_auto_id<I: ::core::convert::Into<::gpui::ElementId>>(
                self,
                id: I,
            ) -> Self::Output {
                ::gpui::InteractiveElement::id(self.0.take().expect("RSX element consumed once"), id)
            }
        }
        // Autoref fallback leaves component builders intact.
        impl<T> __RsxWithAutoId for &__RsxAutoId<T> {
            type Output = T;
            fn __rsx_with_auto_id<I: ::core::convert::Into<::gpui::ElementId>>(
                self,
                _id: I,
            ) -> Self::Output {
                self.0.take().expect("RSX element consumed once")
            }
        }
        __RsxAutoId(::core::cell::Cell::new(::core::option::Option::Some(#element)))
            .__rsx_with_auto_id(#id)
    }}
}
