use dioxus_rsx::CallBody;
use dioxus_rsx_hotreload::diff_rsx;

fn markup_files(macro_path: &str) -> (syn::File, syn::File) {
    let old = format!(
        "use gpui_rsx::rsx as gpui_view;
         fn content() {{ {macro_path}! {{ <div>{{\"before\"}}</div> }} }}"
    );
    let new = old.replace("before", "after");
    (
        syn::parse_file(&old).unwrap(),
        syn::parse_file(&new).unwrap(),
    )
}

#[test]
fn gpui_rsx_is_collected_but_rejected_by_the_dioxus_parser() {
    for macro_path in ["rsx", "gpui_rsx::rsx"] {
        let (old, new) = markup_files(macro_path);
        let changes = diff_rsx(&new, &old).expect("dx treats rsx as template changes");
        assert_eq!(changes.len(), 1);
        for change in changes {
            assert!(syn::parse2::<CallBody>(change.old.tokens).is_err());
            assert!(syn::parse2::<CallBody>(change.new.tokens).is_err());
        }
    }
}

#[test]
fn aliases_and_other_macro_names_are_rust_changes() {
    for macro_path in [
        "gpui_view",
        "view",
        "gpui_rsx::rsx_strict",
        "gpui_rsx::rsx_permissive",
        "gpui_rsx::rsx_tw",
        "gpui_rsx::rsx_fragment",
    ] {
        let (old, new) = markup_files(macro_path);
        assert!(
            diff_rsx(&new, &old).is_none(),
            "{macro_path} should request Rust compilation"
        );
    }
}

#[test]
fn builder_edits_are_rust_changes() {
    let old = syn::parse_file("fn content() { div().child(\"before\") }").unwrap();
    let new = syn::parse_file("fn content() { div().child(\"after\") }").unwrap();
    assert!(diff_rsx(&new, &old).is_none());
}

#[test]
fn dioxus_templates_are_collected_and_parse_successfully() {
    let old = syn::parse_file("fn content() { rsx! { div { \"before\" } } }").unwrap();
    let new = syn::parse_file("fn content() { rsx! { div { \"after\" } } }").unwrap();
    let changes = diff_rsx(&new, &old).unwrap();
    assert_eq!(changes.len(), 1);
    for change in changes {
        assert!(syn::parse2::<CallBody>(change.old.tokens).is_ok());
        assert!(syn::parse2::<CallBody>(change.new.tokens).is_ok());
    }
}
