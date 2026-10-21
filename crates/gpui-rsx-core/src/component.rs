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

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

enum PropKind {
    Required,
    Default(Option<Box<syn::Expr>>),
    Children,
}

struct Prop {
    ident: syn::Ident,
    ty: syn::Type,
    kind: PropKind,
    docs: Vec<syn::Attribute>,
}

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
    let mut input: syn::ItemStruct = syn::parse2(item)?;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.span(),
            "#[component] does not support generic structs yet; use `base={...}` with a hand-written constructor",
        ));
    }
    let syn::Fields::Named(fields) = &mut input.fields else {
        return Err(syn::Error::new(
            input.ident.span(),
            "#[component] requires a struct with named fields",
        ));
    };

    let mut props = Vec::new();
    for field in fields.named.iter_mut() {
        let ident = field.ident.clone().expect("named field");
        let mut kind = PropKind::Required;
        let mut kept = Vec::new();
        for attr in field.attrs.drain(..) {
            if attr.path().is_ident("children") {
                kind = PropKind::Children;
            } else if attr.path().is_ident("prop") {
                kind = parse_prop_attr(&attr)?;
            } else {
                kept.push(attr);
            }
        }
        let docs = kept
            .iter()
            .filter(|a| a.path().is_ident("doc"))
            .cloned()
            .collect();
        field.attrs = kept;
        props.push(Prop {
            ident,
            ty: field.ty.clone(),
            kind,
            docs,
        });
    }
    if props
        .iter()
        .filter(|p| matches!(p.kind, PropKind::Children))
        .count()
        > 1
    {
        return Err(syn::Error::new(
            input.ident.span(),
            "#[component] supports at most one #[children] field",
        ));
    }

    let vis = &input.vis;
    let name = &input.ident;
    let builder = format_ident!("{}Builder", name);
    let markers = format_ident!("__rsx_{}_props", to_snake(&name.to_string()));

    let required: Vec<&Prop> = props
        .iter()
        .filter(|p| matches!(p.kind, PropKind::Required))
        .collect();
    let type_params: Vec<syn::Ident> = required
        .iter()
        .map(|p| format_ident!("__{}", to_pascal(&p.ident.to_string())))
        .collect();
    // rustc reports a missing prop as "`MissingRequiredProp: title` was not satisfied",
    // so the marker trait is named exactly like the prop.
    let has_traits: Vec<syn::Ident> = required.iter().map(|p| p.ident.clone()).collect();

    // Builder fields.
    let builder_fields = props.iter().map(|p| {
        let ident = &p.ident;
        let ty = &p.ty;
        match p.kind {
            PropKind::Required => quote! { #ident: ::core::option::Option<#ty> },
            _ => quote! { #ident: #ty },
        }
    });
    let initial_fields = props.iter().map(|p| {
        let ident = &p.ident;
        match &p.kind {
            PropKind::Required => quote! { #ident: ::core::option::Option::None },
            PropKind::Default(Some(expr)) => quote! { #ident: (#expr).into() },
            PropKind::Default(None) | PropKind::Children => {
                quote! { #ident: ::core::default::Default::default() }
            }
        }
    });
    let missing = type_params
        .iter()
        .map(|_| quote! { #markers::MissingRequiredProp });
    let missing2 = missing.clone();

    // Setters.
    let mut setters = Vec::new();
    for prop in &props {
        let ident = &prop.ident;
        let ty = &prop.ty;
        let docs = &prop.docs;
        match &prop.kind {
            PropKind::Required => {
                let position = required
                    .iter()
                    .position(|r| r.ident == *ident)
                    .expect("required prop");
                let output_params = type_params.iter().enumerate().map(|(i, param)| {
                    if i == position {
                        quote! { #markers::Set }
                    } else {
                        quote! { #param }
                    }
                });
                let moved = props.iter().map(|p| {
                    let field = &p.ident;
                    if p.ident == *ident {
                        quote! { #field: ::core::option::Option::Some(value.into()) }
                    } else {
                        quote! { #field: self.#field }
                    }
                });
                setters.push(quote! {
                    impl<#(#type_params),*> #builder<#(#type_params),*> {
                        #(#docs)*
                        #[must_use]
                        pub fn #ident(self, value: impl ::core::convert::Into<#ty>) -> #builder<#(#output_params),*> {
                            #builder {
                                #(#moved,)*
                                __rsx_state: ::core::marker::PhantomData,
                            }
                        }
                    }
                });
            }
            PropKind::Default(_) if is_bool(ty) => {
                let conditional = format_ident!("{}_if", ident);
                setters.push(quote! {
                    impl<#(#type_params),*> #builder<#(#type_params),*> {
                        #(#docs)*
                        #[must_use]
                        pub fn #ident(mut self) -> Self {
                            self.#ident = true;
                            self
                        }

                        #[doc = concat!("Sets `", stringify!(#ident), "` from a runtime condition (`", stringify!(#ident), "If={cond}` in RSX).")]
                        #[must_use]
                        pub fn #conditional(mut self, value: bool) -> Self {
                            self.#ident = value;
                            self
                        }
                    }
                });
            }
            PropKind::Default(_) => {
                setters.push(quote! {
                    impl<#(#type_params),*> #builder<#(#type_params),*> {
                        #(#docs)*
                        #[must_use]
                        pub fn #ident(mut self, value: impl ::core::convert::Into<#ty>) -> Self {
                            self.#ident = value.into();
                            self
                        }
                    }
                });
            }
            PropKind::Children => {
                setters.push(quote! {
                    impl<#(#type_params),*> #builder<#(#type_params),*> {
                        /// Appends one child element.
                        #[must_use]
                        pub fn child(mut self, child: impl IntoElement) -> Self {
                            self.#ident.push(IntoElement::into_any_element(child));
                            self
                        }

                        /// Appends several child elements.
                        #[must_use]
                        pub fn children(
                            mut self,
                            children: impl ::core::iter::IntoIterator<Item = impl IntoElement>,
                        ) -> Self {
                            self.#ident
                                .extend(children.into_iter().map(IntoElement::into_any_element));
                            self
                        }
                    }
                });
            }
        }
    }

    let bounds = type_params
        .iter()
        .zip(&has_traits)
        .map(|(param, has)| quote! { #param: #markers::#has });
    let bounds2 = bounds.clone();
    let build_fields = props.iter().map(|p| {
        let ident = &p.ident;
        match p.kind {
            PropKind::Required => quote! {
                #ident: match self.#ident {
                    ::core::option::Option::Some(value) => value,
                    ::core::option::Option::None => unreachable!("typestate guarantees required props"),
                }
            },
            _ => quote! { #ident: self.#ident },
        }
    });

    let marker_traits = required.iter().zip(&has_traits).map(|(prop, has)| {
        let message = format!("missing required prop `{}` on `<{}>`", prop.ident, name);
        let label = format!("`{}` was never set", prop.ident);
        let note = format!(
            "add `{}={{...}}` to the `<{}>` element (or call `.{}(...)` on the builder)",
            prop.ident, name, prop.ident
        );
        quote! {
            #[diagnostic::on_unimplemented(message = #message, label = #label, note = #note)]
            pub trait #has {}
            impl #has for Set {}
        }
    });

    let builder_doc = format!("Props builder generated by `#[component]` for [`{name}`].");
    let ctor_doc = format!("Starts building a [`{name}`]; RSX calls this for `<{name} ...>`.");

    Ok(quote! {
        #input

        #[doc(hidden)]
        #[allow(non_snake_case, non_camel_case_types)]
        #vis mod #markers {
            pub struct MissingRequiredProp;
            pub struct Set;
            #(#marker_traits)*
        }

        #[doc = #builder_doc]
        #[must_use]
        #vis struct #builder<#(#type_params = #markers::MissingRequiredProp),*> {
            #(#builder_fields,)*
            __rsx_state: ::core::marker::PhantomData<(#(#type_params,)*)>,
        }

        #[doc = #ctor_doc]
        #[allow(non_snake_case)]
        #vis fn #name() -> #builder<#(#missing),*> {
            #builder::<#(#missing2),*> {
                #(#initial_fields,)*
                __rsx_state: ::core::marker::PhantomData,
            }
        }

        #(#setters)*

        impl<#(#bounds),*> #builder<#(#type_params),*> {
            /// Finishes the builder. Only available once every required prop is set.
            pub fn build(self) -> #name {
                #name {
                    #(#build_fields,)*
                }
            }
        }

        impl<#(#bounds2),*> IntoElement for #builder<#(#type_params),*> {
            type Element = <#name as IntoElement>::Element;

            fn into_element(self) -> Self::Element {
                IntoElement::into_element(self.build())
            }
        }

    })
}

fn parse_prop_attr(attr: &syn::Attribute) -> syn::Result<PropKind> {
    let mut kind = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("default") {
            if meta.input.peek(syn::Token![=]) {
                let expr: syn::Expr = meta.value()?.parse()?;
                kind = Some(PropKind::Default(Some(Box::new(expr))));
            } else {
                kind = Some(PropKind::Default(None));
            }
            Ok(())
        } else {
            Err(meta.error("unknown prop option; expected `default` or `default = expr`"))
        }
    })?;
    kind.ok_or_else(|| {
        syn::Error::new(
            attr.span(),
            "expected #[prop(default)] or #[prop(default = expr)]",
        )
    })
}

fn is_bool(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p) if p.qself.is_none() && p.path.is_ident("bool"))
}

fn to_snake(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn to_pascal(name: &str) -> String {
    name.split('_')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            chars
                .next()
                .map(|c| c.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_generics_and_tuple_structs() {
        let out =
            expand_component(TokenStream::new(), quote! { struct Card<T> { t: T } }).to_string();
        assert!(out.contains("does not support generic structs"));
        let out = expand_component(TokenStream::new(), quote! { struct Card(u8); }).to_string();
        assert!(out.contains("named fields"));
    }

    #[test]
    fn generates_builder_function_and_markers() {
        let out = expand_component(
            TokenStream::new(),
            quote! {
                pub struct TaskCard {
                    title: SharedString,
                    #[prop(default)] elevated: bool,
                    #[children] children: Vec<AnyElement>,
                }
            },
        )
        .to_string();
        assert!(out.contains("pub fn TaskCard ()"));
        assert!(out.contains("pub struct TaskCardBuilder"));
        assert!(out.contains("mod __rsx_task_card_props"));
        assert!(out.contains("missing required prop `title` on `<TaskCard>`"));
        assert!(out.contains("pub fn elevated_if"));
        assert!(!out.contains("# [prop"));
        assert!(!out.contains("# [children]"));
    }
}
