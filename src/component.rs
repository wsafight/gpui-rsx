use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, FnArg, Ident, ItemFn, Pat, PatType, Type};

fn parse_prop<'a>(arg: &'a FnArg) -> syn::Result<(&'a Ident, &'a Type)> {
    match arg {
        FnArg::Typed(PatType { pat, ty, .. }) => match &**pat {
            Pat::Ident(pat_ident) => Ok((&pat_ident.ident, &**ty)),
            _ => Err(Error::new_spanned(
                pat,
                "Unsupported pattern in component argument",
            )),
        },
        FnArg::Receiver(_) => Err(Error::new_spanned(
            arg,
            "Methods (self) are not supported in functional components",
        )),
    }
}

pub fn generate_component(item: ItemFn) -> syn::Result<TokenStream> {
    let vis = &item.vis;
    let name = &item.sig.ident;
    let body = &item.block;

    let mut struct_fields = TokenStream::new();
    let mut builder_methods = TokenStream::new();
    let mut init_fields = TokenStream::new();
    let mut field_extracts = TokenStream::new();
    
    struct_fields.extend(quote! { children: ::std::vec::Vec<::gpui::AnyElement>, });
    init_fields.extend(quote! { children: ::std::vec::Vec::new(), });

    for arg in &item.sig.inputs {
        let (ident, ty) = parse_prop(arg)?;

        if ident == "children" {
            field_extracts.extend(quote! { let children = self.children; });
            continue;
        }

        let err_msg = format!("Missing required property `{ident}` for component `{name}`");
        struct_fields.extend(quote! { #ident: ::std::option::Option<#ty>, });

        builder_methods.extend(quote! {
            #vis fn #ident(mut self, #ident: #ty) -> Self {
                self.#ident = ::std::option::Option::Some(#ident);
                self
            }
        });

        init_fields.extend(quote! { #ident: ::std::option::Option::None, });

        field_extracts.extend(quote! { let #ident = self.#ident.expect(#err_msg); });
    }

    Ok(quote! {
        #vis struct #name {
            #struct_fields
        }

        #[allow(non_snake_case)]
        #vis fn #name() -> #name {
            #name {
                #init_fields
            }
        }

        impl #name {
            #builder_methods
        }

        impl ::gpui::ParentElement for #name {
            fn extend(&mut self, elements: impl ::std::iter::IntoIterator<Item = ::gpui::AnyElement>) {
                self.children.extend(elements);
            }
        }

        impl ::gpui::IntoElement for #name {
            type Element = ::gpui::AnyElement;

            fn into_element(self) -> Self::Element {
                #field_extracts

                let __rendered = (move || {
                    #body
                })();

                ::gpui::IntoElement::into_any_element(__rendered)
            }
        }
    })
}
