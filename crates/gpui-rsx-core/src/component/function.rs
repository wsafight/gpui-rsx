//! `#[component]` on functions: props become a generated struct rendered via `RenderOnce`.
use super::builder::expand_struct;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt;

pub(super) fn expand_function(
    mut input: syn::ItemFn,
    gates: &[syn::Attribute],
) -> syn::Result<TokenStream> {
    let sig = &input.sig;
    if sig.asyncness.is_some()
        || sig.constness.is_some()
        || sig.unsafety.is_some()
        || sig.abi.is_some()
        || sig.variadic.is_some()
    {
        return Err(syn::Error::new_spanned(
            sig,
            "#[component] requires a safe, synchronous Rust function",
        ));
    }
    if matches!(sig.output, syn::ReturnType::Default) {
        return Err(syn::Error::new_spanned(
            sig,
            "component functions must return an element, e.g. `-> impl IntoElement`",
        ));
    }

    let mut render_generics = input.sig.generics.clone();
    add_static_bounds(&mut render_generics);
    let render_args: Vec<_> = render_generics
        .params
        .iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(param) => {
                let name = &param.ident;
                Some(quote! { #name })
            }
            syn::GenericParam::Const(param) => {
                let name = &param.ident;
                Some(quote! { #name })
            }
            _ => None,
        })
        .collect();
    let (render_impl, render_type, render_where) = render_generics.split_for_impl();
    let name = input.sig.ident.clone();
    let vis = input.vis.clone();
    let attrs: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| {
            attr_is_one_of(
                attr,
                &[
                    "doc",
                    "allow",
                    "warn",
                    "deny",
                    "forbid",
                    "expect",
                    "deprecated",
                ],
            )
        })
        .cloned()
        .collect();
    let render_fn = format_ident!("__rsx_render_{}", name.unraw());
    let mut fields = Vec::new();
    let mut arguments = Vec::new();
    let (mut has_window, mut has_app) = (false, false);
    for arg in &mut input.sig.inputs {
        let syn::FnArg::Typed(arg) = arg else {
            return Err(syn::Error::new_spanned(
                arg,
                "component functions cannot take `self`",
            ));
        };
        let syn::Pat::Ident(pattern) = &*arg.pat else {
            return Err(syn::Error::new_spanned(
                &arg.pat,
                "component parameters must be named props; destructuring is not supported",
            ));
        };
        if pattern.by_ref.is_some() || pattern.subpat.is_some() {
            return Err(syn::Error::new_spanned(
                &arg.pat,
                "component parameters must be named props",
            ));
        }
        let ident = &pattern.ident;
        let ty = &arg.ty;
        if arg.attrs.iter().any(|attr| attr.path().is_ident("context")) {
            if arg.attrs.len() != 1 || !matches!(arg.attrs[0].meta, syn::Meta::Path(_)) {
                return Err(syn::Error::new_spanned(
                    arg,
                    "#[context] cannot have options or be combined with prop attributes",
                ));
            }
            let syn::Type::Reference(reference) = &**ty else {
                return Err(context_error(ty));
            };
            let syn::Type::Path(path) = &*reference.elem else {
                return Err(context_error(ty));
            };
            let last = path.path.segments.last().map(|segment| &segment.ident);
            let (seen, argument) = match last {
                Some(ident) if ident == "Window" => (&mut has_window, quote! { __rsx_window }),
                Some(ident) if ident == "App" => (&mut has_app, quote! { __rsx_cx }),
                _ => return Err(context_error(ty)),
            };
            if reference.mutability.is_none() {
                return Err(context_error(ty));
            }
            if std::mem::replace(seen, true) {
                return Err(syn::Error::new_spanned(
                    arg,
                    "each context type may appear only once",
                ));
            }
            arguments.push(argument);
        } else {
            if let syn::Type::Reference(reference) = &**ty
                && !reference
                    .lifetime
                    .as_ref()
                    .is_some_and(|lifetime| lifetime.ident == "static")
            {
                return Err(syn::Error::new_spanned(
                    ty,
                    "component props must own their values or use 'static references; use #[context] for &mut Window or &mut App",
                ));
            }
            if matches!(&**ty, syn::Type::ImplTrait(_)) {
                return Err(syn::Error::new_spanned(
                    ty,
                    "component props need concrete types; use a boxed callback instead of `impl Trait`",
                ));
            }
            let prop_attrs = &arg.attrs;
            if prop_attrs
                .iter()
                .any(|attr| !attr_is_one_of(attr, &["prop", "children", "doc"]))
            {
                return Err(syn::Error::new_spanned(
                    arg,
                    "component props support only #[prop(...)] and #[children] annotations",
                ));
            }
            fields.push(quote! { #(#prop_attrs)* #ident: #ty });
            arguments.push(quote! { self.#ident });
        }
        // Prop annotations belong to generated fields, not the private render function.
        arg.attrs.clear();
    }
    input.sig.ident = render_fn.clone();
    input.vis = syn::Visibility::Inherited;
    // Docs and deprecation belong to the public constructor, not the private render function.
    input
        .attrs
        .retain(|attr| !attr.path().is_ident("doc") && !attr.path().is_ident("deprecated"));
    input
        .attrs
        .push(syn::parse_quote!(#[allow(non_snake_case)]));
    // A function can use generic types only in its body. Keep those parameters present
    // in the generated props type even when no public prop mentions them.
    let phantom_types: Vec<_> = render_generics
        .params
        .iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(param) => {
                let name = &param.ident;
                Some(quote! { fn() -> *const #name })
            }
            syn::GenericParam::Lifetime(param) => {
                let lifetime = &param.lifetime;
                Some(quote! { &#lifetime () })
            }
            _ => None,
        })
        .collect();
    if !phantom_types.is_empty() {
        let mut phantom_name = format_ident!("__rsx_generics");
        while input.sig.inputs.iter().any(|arg| matches!(arg, syn::FnArg::Typed(arg) if matches!(&*arg.pat, syn::Pat::Ident(p) if p.ident == phantom_name))) {
            phantom_name = format_ident!("{}_", phantom_name);
        }
        fields.push(quote! { #[doc(hidden)] #[prop(default)] #phantom_name: ::core::marker::PhantomData<(#(#phantom_types,)*)> });
    }
    let props_where = &render_generics.where_clause;
    let props: syn::ItemStruct = syn::parse2(quote! {
        #(#gates)*
        #(#attrs)*
        #[derive(::gpui::IntoElement)]
        #vis struct #name #render_generics #props_where { #(#fields,)* }
    })?;
    let builder = expand_struct(props, gates)?;
    Ok(quote! {
        #builder
        #input

        #(#gates)*
        #[allow(deprecated)]
        impl #render_impl ::gpui::RenderOnce for #name #render_type #render_where {
            fn render(self, __rsx_window: &mut ::gpui::Window, __rsx_cx: &mut ::gpui::App) -> impl ::gpui::IntoElement {
                #render_fn::<#(#render_args),*>(#(#arguments),*)
            }
        }
    })
}

fn add_static_bounds(generics: &mut syn::Generics) {
    for param in &mut generics.params {
        match param {
            syn::GenericParam::Type(param) => {
                let predicate = generics.where_clause.as_mut().and_then(|clause| {
                    clause
                        .predicates
                        .iter_mut()
                        .find_map(|predicate| match predicate {
                            syn::WherePredicate::Type(predicate)
                                if matches!(&predicate.bounded_ty,
                            syn::Type::Path(path) if path.path.is_ident(&param.ident)) =>
                            {
                                Some(predicate)
                            }
                            _ => None,
                        })
                });
                let is_static = |bounds: &syn::punctuated::Punctuated<
                    syn::TypeParamBound,
                    syn::Token![+],
                >| {
                    bounds.iter().any(|bound| matches!(bound, syn::TypeParamBound::Lifetime(lifetime) if lifetime.ident == "static"))
                };
                if is_static(&param.bounds)
                    || predicate.as_ref().is_some_and(|p| is_static(&p.bounds))
                {
                    continue;
                }
                if param.bounds.is_empty()
                    && let Some(predicate) = predicate
                {
                    predicate.bounds.push(syn::parse_quote!('static));
                } else {
                    param.bounds.push(syn::parse_quote!('static));
                }
            }
            syn::GenericParam::Lifetime(param)
                if !param.bounds.iter().any(|l| l.ident == "static") =>
            {
                param.bounds.push(syn::parse_quote!('static));
            }
            _ => {}
        }
    }
}

fn attr_is_one_of(attr: &syn::Attribute, names: &[&str]) -> bool {
    names.iter().any(|name| attr.path().is_ident(name))
}

fn context_error(ty: &syn::Type) -> syn::Error {
    syn::Error::new_spanned(ty, "#[context] requires `&mut Window` or `&mut App`")
}
