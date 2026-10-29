//! Closure setters accept `impl Fn...` for boxed/shared callbacks so closure signatures infer.
use proc_macro2::TokenStream;
use quote::quote;

pub(super) struct Setter {
    pub value_type: TokenStream,
    pub converted: TokenStream,
}

/// A typed binding supplies the coercion site needed by `Box::new` / `Rc::new` closures.
/// Ordinary defaults still accept `Into` conversions, such as `&str` to `SharedString`.
pub(super) fn default_value(ty: &syn::Type, expr: &syn::Expr, explicit: bool) -> TokenStream {
    if callback_parts(ty).is_some_and(|(_, object)| explicit || is_callable(object)) {
        quote! {{ let __rsx_default: #ty = #expr; __rsx_default }}
    } else {
        quote! { (#expr).into() }
    }
}

fn is_callable(object: &syn::TypeTraitObject) -> bool {
    object.bounds.iter().any(|bound| {
        matches!(bound, syn::TypeParamBound::Trait(bound) if bound.path.segments.last().is_some_and(|segment| matches!(segment.ident.to_string().as_str(), "Fn" | "FnMut" | "FnOnce")))
    })
}

/// `explicit` is `#[prop(callback)]`: accept any `Box<dyn Trait>` / `Rc<dyn Trait>`, such as a
/// trait alias `trait OnPress: Fn(..) {}`, whose closure signature rustc infers via supertraits.
pub(super) fn setter(
    ty: &syn::Type,
    generics: &syn::Generics,
    explicit: bool,
) -> syn::Result<Setter> {
    let ordinary = || {
        if generics
            .type_params()
            .any(|param| matches!(ty, syn::Type::Path(path) if path.path.is_ident(&param.ident)))
        {
            Setter {
                value_type: quote! { #ty },
                converted: quote! { value },
            }
        } else {
            Setter {
                value_type: quote! { impl ::core::convert::Into<#ty> },
                converted: quote! { value.into() },
            }
        }
    };
    let Some((container, object)) = callback_parts(ty) else {
        if explicit {
            return Err(syn::Error::new_spanned(
                ty,
                "#[prop(callback)] needs a written-out `Box<dyn Trait>` or `Rc<dyn Trait>`; \
                 type aliases hide the closure signature, so declare \
                 `trait OnPress: Fn(..) {}` with a blanket impl and use `Rc<dyn OnPress>`",
            ));
        }
        return Ok(ordinary());
    };
    if !is_callable(object) && !explicit {
        return Ok(ordinary());
    }
    let mut bounds = object.bounds.clone();
    if !bounds
        .iter()
        .any(|bound| matches!(bound, syn::TypeParamBound::Lifetime(_)))
    {
        bounds.push(syn::parse_quote!('static));
    }
    // Taking the closure bound directly lets rustc infer argument types and higher-ranked
    // lifetimes (`|_, _, _|` for GPUI handlers); a custom conversion trait would hide it.
    Ok(Setter {
        value_type: quote! { impl #bounds },
        converted: quote! { #container::new(value) },
    })
}

/// Splits `Box<dyn ...>` / `Rc<dyn ...>` into the bare container path and the trait object.
fn callback_parts(ty: &syn::Type) -> Option<(syn::Path, &syn::TypeTraitObject)> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    let last = path.path.segments.last()?;
    if last.ident != "Box" && last.ident != "Rc" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    let Some(syn::GenericArgument::Type(syn::Type::TraitObject(object))) = args.args.first() else {
        return None;
    };
    let mut container = path.path.clone();
    container.segments.last_mut()?.arguments = syn::PathArguments::None;
    Some((container, object))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setter_for(ty: &str, generics: &str) -> Setter {
        let ty: syn::Type = syn::parse_str(ty).unwrap();
        let generics: syn::Generics = syn::parse_str(generics).unwrap();
        setter(&ty, &generics, false).unwrap()
    }

    #[test]
    fn ordinary_props_accept_any_into_value() {
        let setter = setter_for("String", "");
        assert!(setter.value_type.to_string().contains("Into < String >"));
        assert_eq!(setter.converted.to_string(), "value . into ()");
    }

    #[test]
    fn generic_props_are_passed_through_unchanged() {
        let setter = setter_for("T", "<T>");
        assert_eq!(setter.value_type.to_string(), "T");
        assert_eq!(setter.converted.to_string(), "value");
    }

    #[test]
    fn containers_of_a_generic_still_use_into() {
        let setter = setter_for("Vec<T>", "<T>");
        assert!(setter.value_type.to_string().contains("Into < Vec < T > >"));
        assert!(setter.converted.to_string().contains(". into ()"));
    }

    #[test]
    fn boxed_and_shared_callbacks_take_closures_with_a_static_bound() {
        for (ty, container) in [
            ("Box<dyn Fn(u8)>", "Box"),
            ("Rc<dyn FnMut(u8)>", "Rc"),
            ("std::rc::Rc<dyn Fn(u8)>", "std :: rc :: Rc"),
        ] {
            let setter = setter_for(ty, "");
            let value = setter.value_type.to_string();
            assert!(value.starts_with("impl"), "{value}");
            assert!(value.contains("(u8)"), "{value}");
            assert_eq!(value.matches("'static").count(), 1, "{value}");
            assert_eq!(
                setter.converted.to_string(),
                format!("{container} :: new (value)")
            );
        }
    }

    #[test]
    fn callbacks_keep_existing_bounds_and_lifetimes() {
        let setter = setter_for("Rc<dyn Fn() + Send + 'static>", "");
        let value = setter.value_type.to_string();
        assert!(value.contains("Send"), "{value}");
        assert_eq!(value.matches("'static").count(), 1, "{value}");
    }

    #[test]
    fn only_callable_trait_objects_get_closure_conversion() {
        for ty in [
            "Box<dyn Clone>",
            "Box<String>",
            "Box",
            "Rc",
            "&'static str",
            "Result<Box<dyn Fn()>, String>",
        ] {
            let setter = setter_for(ty, "");
            assert!(
                setter.converted.to_string().contains("into"),
                "{ty}: {}",
                setter.converted
            );
        }
    }

    #[test]
    fn explicit_callbacks_accept_trait_alias_objects() {
        let ty: syn::Type = syn::parse_str("Rc<dyn OnPress>").unwrap();
        let generics = syn::Generics::default();
        let setter = setter(&ty, &generics, true).unwrap();
        assert_eq!(setter.value_type.to_string(), "impl OnPress + 'static");
        assert_eq!(setter.converted.to_string(), "Rc :: new (value)");
        assert!(
            setter_for("Rc<dyn OnPress>", "")
                .converted
                .to_string()
                .contains("into")
        );
    }

    #[test]
    fn explicit_callbacks_reject_aliases() {
        let ty: syn::Type = syn::parse_str("OnPress").unwrap();
        let error = setter(&ty, &syn::Generics::default(), true)
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("trait OnPress"), "{error}");
    }
}
