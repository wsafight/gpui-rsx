use gpui_rsx_core::{
    ClassMode, Theme, expand_component_with_path, expand_listener_with_path, expand_str,
    expand_update_with_path, generate_runtime_module,
};
use quote::quote;
use std::sync::Arc;

#[test]
fn configured_path_reaches_erasure_empty_loops_auto_ids_and_dynamic_keys() {
    let theme = Arc::new(Theme::parse("[options]\ngpui-path = \"::engine\"\n").unwrap());
    for source in [
        "<>{for _ in 0..0 {}}</>",
        "<><div key={key} onClick={handler}/></>",
        "<Card onClick={handler}/>",
        "<div>{for item in items {<Card key={item.id} onClick={handler}/>{for _ in 0..0 {}}}}</div>",
    ] {
        let output = expand_str(
            source,
            ClassMode::PERMISSIVE.with_theme(Some(theme.clone())),
        )
        .unwrap()
        .to_string();
        assert!(output.contains(":: engine ::"), "{output}");
        assert!(!output.contains(":: gpui ::"), "{output}");
    }
}

#[test]
fn configured_paths_do_not_rewrite_user_expressions() {
    let theme = Arc::new(Theme::parse("[options]\ngpui-path = \"::engine\"\n").unwrap());
    let output = expand_str(
        "<div base={::gpui::custom()} onClick={handler}/>",
        ClassMode::PERMISSIVE.with_theme(Some(theme)),
    )
    .unwrap()
    .to_string();
    assert!(output.contains(":: gpui :: custom"));
    assert!(output.contains(":: engine :: ElementId"));
}

#[test]
fn components_listeners_updates_and_shared_runtime_use_the_same_path() {
    let path = syn::parse_quote!(::engine);
    let component = expand_component_with_path(
        quote!(),
        quote!(
            fn Card(#[children] children: Vec<engine::AnyElement>) -> impl engine::IntoElement {
                engine::div().children(children)
            }
        ),
        &path,
    )
    .to_string();
    assert!(component.contains(":: engine :: ViewElement"));
    assert!(!component.contains(":: gpui ::"));
    let listener = expand_listener_with_path(quote!(cx, notify, handler), &path).to_string();
    assert!(listener.contains(":: engine :: Context"));
    assert!(!listener.contains(":: gpui ::"));
    let update = expand_update_with_path(quote!(view, cx, handler), &path).to_string();
    assert!(update.contains(":: engine :: Context"));
    assert!(!update.contains(":: gpui ::"));
    let runtime = generate_runtime_module(&path).to_string();
    assert!(runtime.contains(":: engine ::"));
    assert!(!runtime.contains(":: gpui ::"));
}

#[test]
fn invalid_or_relative_configured_paths_are_rejected() {
    for path in [
        "engine",
        "self::ui",
        "super::engine",
        "engine()",
        "::engine<u8>",
        "crate::ui::<u8>",
        "::engine::",
        "",
    ] {
        let error = Theme::parse(&format!("[options]\ngpui-path = {path:?}\n")).unwrap_err();
        assert!(error.contains("rsx.toml:2:"), "{error}");
        assert!(error.contains("absolute Rust path"), "{error}");
    }
}

#[test]
fn crate_root_reexports_are_valid_generated_paths() {
    let theme = Arc::new(Theme::parse("[options]\ngpui-path = \"crate::ui\"\n").unwrap());
    let output = expand_str(
        "<div key={key} onClick={handler}/>",
        ClassMode::PERMISSIVE.with_theme(Some(theme)),
    )
    .unwrap()
    .to_string();
    assert!(output.contains("crate :: ui :: SharedString"));
    assert!(!output.contains(":: gpui ::"));
}

#[test]
fn configured_path_component_impl_is_track_caller_and_default_path_uses_the_derive() {
    let source = quote!(
        fn Card() -> impl gpui::IntoElement {
            gpui::div()
        }
    );
    let configured =
        expand_component_with_path(quote!(), source.clone(), &syn::parse_quote!(::engine))
            .to_string();
    assert!(
        configured.contains("# [track_caller] fn into_element"),
        "{configured}"
    );
    assert!(!configured.contains("derive"), "{configured}");

    let default =
        expand_component_with_path(quote!(), source, &syn::parse_quote!(::gpui)).to_string();
    assert!(
        default.contains("derive (:: gpui :: IntoElement)"),
        "{default}"
    );
    assert!(!default.contains("ViewElement"), "{default}");
}

#[test]
fn gpui_path_falls_back_to_gpui_and_follows_the_theme() {
    use quote::ToTokens;
    let text = |path: syn::Path| path.to_token_stream().to_string();
    assert_eq!(text(Theme::default().gpui_syn_path()), ":: gpui");
    assert_eq!(text(ClassMode::PERMISSIVE.gpui_path()), ":: gpui");

    let theme = Arc::new(Theme::parse("[options]\ngpui-path = \"crate::ui\"\n").unwrap());
    assert_eq!(text(theme.gpui_syn_path()), "crate :: ui");
    assert_eq!(
        text(ClassMode::STRICT.with_theme(Some(theme)).gpui_path()),
        "crate :: ui"
    );
}
