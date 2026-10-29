//! Typestate props builders shared by struct and function components.
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt;
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
    callback: bool,
    docs: Vec<syn::Attribute>,
}

/// `gates` are the item's `#[cfg]` attributes, which `input` already carries itself.
pub(super) fn expand_struct(
    mut input: syn::ItemStruct,
    gates: &[syn::Attribute],
) -> syn::Result<TokenStream> {
    let syn::Fields::Named(fields) = &mut input.fields else {
        return Err(syn::Error::new(
            input.ident.span(),
            "#[component] requires a struct with named fields",
        ));
    };

    let mut props = Vec::new();
    for field in fields.named.iter_mut() {
        let ident = field.ident.clone().expect("named field");
        let mut kind = None;
        let mut callback = false;
        let mut kept = Vec::new();
        for attr in field.attrs.drain(..) {
            let parsed = if attr.path().is_ident("children") {
                if !matches!(attr.meta, syn::Meta::Path(_)) {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "#[children] takes no arguments",
                    ));
                }
                PropKind::Children
            } else if attr.path().is_ident("prop") {
                let (parsed, is_callback) = parse_prop_attr(&attr)?;
                callback |= is_callback;
                match parsed {
                    Some(parsed) => parsed,
                    None => continue,
                }
            } else {
                kept.push(attr);
                continue;
            };
            if kind.replace(parsed).is_some() {
                return Err(syn::Error::new_spanned(
                    &field.ident,
                    "use only one #[prop(...)] or #[children] annotation per prop",
                ));
            }
        }
        if callback && matches!(kind, Some(PropKind::Children)) {
            return Err(syn::Error::new_spanned(
                &field.ident,
                "#[prop(callback)] cannot be combined with #[children]",
            ));
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
            kind: kind.unwrap_or(PropKind::Required),
            callback,
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

    let original_parameters: Vec<_> = input.generics.params.iter().cloned().collect();
    let impl_parameters: Vec<_> = original_parameters
        .iter()
        .cloned()
        .map(|mut param| {
            match &mut param {
                syn::GenericParam::Type(param) => param.default = None,
                syn::GenericParam::Const(param) => param.default = None,
                _ => {}
            }
            param
        })
        .collect();
    let original_arguments: Vec<_> = original_parameters
        .iter()
        .map(|param| match param {
            syn::GenericParam::Lifetime(param) => {
                let lifetime = &param.lifetime;
                quote! { #lifetime }
            }
            syn::GenericParam::Type(param) => {
                let ident = &param.ident;
                quote! { #ident }
            }
            syn::GenericParam::Const(param) => {
                let ident = &param.ident;
                quote! { #ident }
            }
        })
        .collect();
    let where_clause = &input.generics.where_clause;
    let constructor_generics = if impl_parameters.is_empty() {
        quote! {}
    } else {
        quote! { <#(#impl_parameters),*> }
    };
    let vis = &input.vis;
    let name = &input.ident;
    let display = name.unraw();
    let builder = format_ident!("{}Builder", display);
    let markers = format_ident!("__rsx_{}_props", to_snake(&name.unraw().to_string()));
    // Generated code names the component itself, so only the constructor users call carries
    // the deprecation; the generated impls allow it.
    let deprecated: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("deprecated"))
        .collect();

    let required: Vec<&Prop> = props
        .iter()
        .filter(|p| matches!(p.kind, PropKind::Required))
        .collect();
    // `a_b` and `a__b` both PascalCase to `AB`; suffix a counter on a clash so the
    // generated generics stay distinct.
    let mut seen: std::collections::HashSet<String> =
        original_arguments.iter().map(ToString::to_string).collect();
    let type_params: Vec<syn::Ident> = required
        .iter()
        .map(|p| {
            let pascal = to_pascal(&p.ident.unraw().to_string());
            let mut name = pascal.clone();
            let mut n = 1;
            while !seen.insert(name.clone()) {
                name = format!("{pascal}{n}");
                n += 1;
            }
            format_ident!("__{}", name)
        })
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
            PropKind::Default(Some(expr)) => {
                let value = super::callback::default_value(&p.ty, expr, p.callback);
                quote! { #ident: #value }
            }
            PropKind::Default(None) | PropKind::Children => {
                quote! { #ident: ::core::default::Default::default() }
            }
        }
    });
    let missing_types: Vec<_> = required
        .iter()
        .map(|prop| format_ident!("MissingRequiredProp_{}_on_{}", prop.ident.unraw(), display))
        .collect();
    let defaults = type_params
        .iter()
        .zip(&missing_types)
        .map(|(param, missing)| quote! { #param = #markers::#missing });
    let missing = missing_types
        .iter()
        .map(|missing| quote! { #markers::#missing });
    let missing2 = missing.clone();

    // Setters.
    let mut setters = Vec::new();
    for prop in &props {
        let ident = &prop.ident;
        let ty = &prop.ty;
        let docs = &prop.docs;
        let setter = super::callback::setter(ty, &input.generics, prop.callback)?;
        let value_type = setter.value_type;
        let converted = setter.converted;
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
                        quote! { #field: ::core::option::Option::Some(#converted) }
                    } else {
                        quote! { #field: self.#field }
                    }
                });
                setters.push(quote! {
                    impl<#(#impl_parameters,)* #(#type_params),*> #builder<#(#original_arguments,)* #(#type_params),*> #where_clause {
                        #(#docs)*
                        #[must_use]
                        pub fn #ident(self, value: #value_type) -> #builder<#(#original_arguments,)* #(#output_params),*> {
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
                    impl<#(#impl_parameters,)* #(#type_params),*> #builder<#(#original_arguments,)* #(#type_params),*> #where_clause {
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
                    impl<#(#impl_parameters,)* #(#type_params),*> #builder<#(#original_arguments,)* #(#type_params),*> #where_clause {
                        #(#docs)*
                        #[must_use]
                        pub fn #ident(mut self, value: #value_type) -> Self {
                            self.#ident = #converted;
                            self
                        }
                    }
                });
            }
            PropKind::Children => {
                setters.push(quote! {
                    impl<#(#impl_parameters,)* #(#type_params),*> #builder<#(#original_arguments,)* #(#type_params),*> #where_clause {
                        /// Appends one child element.
                        #[must_use]
                        pub fn child(mut self, child: impl ::gpui::IntoElement) -> Self {
                            self.#ident.push(::gpui::IntoElement::into_any_element(child));
                            self
                        }

                        /// Appends several child elements.
                        #[must_use]
                        pub fn children(
                            mut self,
                            children: impl ::core::iter::IntoIterator<Item = impl ::gpui::IntoElement>,
                        ) -> Self {
                            self.#ident
                                .extend(children.into_iter().map(::gpui::IntoElement::into_any_element));
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
    let mut element_generics = input.generics.clone();
    for (param, has) in type_params.iter().zip(&has_traits) {
        element_generics
            .params
            .push(syn::parse_quote!(#param: #markers::#has));
    }
    element_generics
        .make_where_clause()
        .predicates
        .push(syn::parse_quote!(#name<#(#original_arguments),*>: ::gpui::IntoElement));
    let (element_impl, element_args, element_where) = element_generics.split_for_impl();
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
        let message = format!("missing required prop `{}` on `<{}>`", prop.ident, display);
        let label = format!("`{}` was never set", prop.ident);
        let note = format!(
            "add `{}={{...}}` to the `<{}>` element (or call `.{}(...)` on the builder)",
            prop.ident, display, prop.ident
        );
        quote! {
            #[diagnostic::on_unimplemented(message = #message, label = #label, note = #note)]
            pub trait #has {}
            impl #has for Set {}
        }
    });

    let setters = setters
        .iter()
        .map(|setter| quote! { #(#gates)* #[allow(deprecated)] #setter });
    let builder_doc = format!("Props builder generated by `#[component]` for [`{display}`].");
    let ctor_doc =
        format!("Starts building a [`{display}`]; RSX calls this for `<{display} ...>`.");

    Ok(quote! {
        #input

        #(#gates)*
        #[doc(hidden)]
        #[allow(non_snake_case, non_camel_case_types)]
        #vis mod #markers {
            #(pub struct #missing_types;)*
            pub struct Set;
            #(#marker_traits)*
        }

        #(#gates)*
        #[doc = #builder_doc]
        #[must_use]
        #[allow(deprecated)]
        #vis struct #builder<#(#original_parameters,)* #(#defaults),*> #where_clause {
            #(#builder_fields,)*
            __rsx_state: ::core::marker::PhantomData<(fn() -> #name<#(#original_arguments),*>, #(#type_params,)*)>,
        }

        #(#gates)*
        #[doc = #ctor_doc]
        #(#deprecated)*
        #[allow(non_snake_case, deprecated)]
        #vis fn #name #constructor_generics () -> #builder<#(#original_arguments,)* #(#missing),*> #where_clause {
            #builder::<#(#original_arguments,)* #(#missing2),*> {
                #(#initial_fields,)*
                __rsx_state: ::core::marker::PhantomData,
            }
        }

        #(#setters)*

        #(#gates)*
        #[allow(deprecated)]
        impl<#(#impl_parameters,)* #(#bounds),*> #builder<#(#original_arguments,)* #(#type_params),*> #where_clause {
            /// Finishes the builder. Only available once every required prop is set.
            pub fn build(self) -> #name<#(#original_arguments),*> {
                #name {
                    #(#build_fields,)*
                }
            }
        }

        #(#gates)*
        #[allow(deprecated)]
        impl #element_impl ::gpui::IntoElement for #builder #element_args #element_where {
            type Element = <#name<#(#original_arguments),*> as ::gpui::IntoElement>::Element;

            fn into_element(self) -> Self::Element {
                ::gpui::IntoElement::into_element(self.build())
            }
        }

    })
}

/// Returns the prop kind (if `default` was given) and whether `callback` was given.
fn parse_prop_attr(attr: &syn::Attribute) -> syn::Result<(Option<PropKind>, bool)> {
    let mut kind = None;
    let mut callback = false;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("callback") {
            callback = true;
            Ok(())
        } else if meta.path.is_ident("default") {
            if meta.input.peek(syn::Token![=]) {
                let expr: syn::Expr = meta.value()?.parse()?;
                kind = Some(PropKind::Default(Some(Box::new(expr))));
            } else {
                kind = Some(PropKind::Default(None));
            }
            Ok(())
        } else {
            Err(meta
                .error("unknown prop option; expected `default`, `default = expr` or `callback`"))
        }
    })?;
    if kind.is_none() && !callback {
        return Err(syn::Error::new(
            attr.span(),
            "expected #[prop(default)], #[prop(default = expr)] or #[prop(callback)]",
        ));
    }
    Ok((kind, callback))
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
