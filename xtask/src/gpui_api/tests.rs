use super::sources::package_root;
use super::styles::{count_params, literal_fns};
use super::trait_signatures;
use proc_macro2::TokenStream;
use std::path::Path;

#[test]
fn resolves_snapshot_sources_instead_of_the_kit_facade_and_rejects_ambiguity() {
    let mut metadata = serde_json::json!({"packages": [
        {"name": "gpui-kit", "manifest_path": "/registry/gpui-kit-0.8.0/Cargo.toml"},
        {"name": "gpui-pre", "manifest_path": "/registry/gpui-pre-0.3.9/Cargo.toml"},
        {"name": "gpui-pre-macros", "manifest_path": "/registry/gpui-pre-macros-0.3.9/Cargo.toml"},
    ]});
    assert_eq!(
        package_root(&metadata, "gpui").unwrap(),
        Path::new("/registry/gpui-pre-0.3.9")
    );
    assert_eq!(
        package_root(&metadata, "gpui_macros").unwrap(),
        Path::new("/registry/gpui-pre-macros-0.3.9")
    );
    metadata["packages"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "name": "gpui", "manifest_path": "/git/zed/crates/gpui/Cargo.toml",
        }));
    assert!(
        package_root(&metadata, "gpui")
            .unwrap_err()
            .contains("found 2")
    );
}

#[test]
fn signatures_detect_type_and_generic_bound_changes_with_equal_arity() {
    let a: syn::ItemTrait =
        syn::parse_quote! { trait A { fn set<T: Into<Hsla>>(self, value:T)->Self; } };
    let b: syn::ItemTrait =
        syn::parse_quote! { trait A { fn set<T: Into<Pixels>>(self, value:T)->Self; } };
    assert_ne!(trait_signatures(&a), trait_signatures(&b));
}

#[test]
fn counts_params_without_receiver() {
    let params: TokenStream = "mut self, a: impl Into<X<Y, Z>>, b: C".parse().unwrap();
    assert_eq!(count_params(params), 2);
    let params: TokenStream = "mut self".parse().unwrap();
    assert_eq!(count_params(params), 0);
}

#[test]
fn finds_literal_quote_fns() {
    let tokens: TokenStream =
            "#[doc = \"x\"] #visibility fn relative(mut self) -> Self { self } #visibility fn #name(mut self) {} fn border_color<C>(mut self, c: C) -> Self where C: Into<Hsla> { self }"
                .parse()
                .unwrap();
    let mut out = Vec::new();
    literal_fns(tokens, &mut out);
    let names: Vec<_> = out.iter().map(|m| (m.name.as_str(), m.arity)).collect();
    assert_eq!(names, [("relative", 0), ("border_color", 1)]);
}
