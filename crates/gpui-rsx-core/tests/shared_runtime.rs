use gpui_rsx_core::{ClassMode, SpacingScale, Theme, expand_str, generate_runtime_module};

#[test]
fn configured_expansions_call_shared_helpers_without_inlining_a_matcher() {
    let theme = Theme::parse("[options]\ndynamic-runtime = \"crate::rsx_runtime\"\n").unwrap();
    for (scale, helper) in [
        (SpacingScale::Px, "px"),
        (SpacingScale::Tailwind, "tailwind"),
    ] {
        let mode = ClassMode::STRICT
            .with_scale(scale)
            .with_theme(Some(theme.clone().into()));
        let expanded = expand_str("<div class={classes} />", mode)
            .unwrap()
            .to_string();
        assert!(
            expanded.contains(&format!("crate :: rsx_runtime :: {helper}")),
            "{expanded}"
        );
        assert!(!expanded.contains("fn __rsx_apply_class"), "{expanded}");
        assert!(
            expanded.contains("unsupported dynamic class"),
            "strict handler stays at the call site"
        );
        assert!(expanded.len() < 2000, "{expanded}");
    }
    let inline = expand_str("<div class={classes} />", ClassMode::PERMISSIVE)
        .unwrap()
        .to_string();
    assert!(inline.contains("fn __rsx_apply_class"));
}

#[test]
fn shared_runtime_options_require_a_module_path() {
    for invalid in [
        "",
        "bad-path",
        "helper::<u8>",
        "helper()",
        "helper; panic!()",
    ] {
        let config = format!("[options]\ndynamic-runtime = \"{invalid}\"\n");
        let error = Theme::parse(&config).unwrap_err();
        assert!(error.contains("rsx.toml:2"), "{error}");
        assert!(error.contains("module path"), "{error}");
    }
}

#[test]
fn generated_module_is_valid_rust_with_concrete_style_helpers() {
    let path = syn::parse_quote!(::custom_gpui);
    let code = generate_runtime_module(&path);
    let file = syn::parse2::<syn::File>(code).unwrap();
    let syn::Item::Mod(module) = &file.items[0] else {
        panic!("expected runtime module")
    };
    let helpers = module
        .content
        .as_ref()
        .unwrap()
        .1
        .iter()
        .filter_map(|item| {
            if let syn::Item::Fn(helper) = item {
                Some(helper)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(helpers.len(), 2);
    for helper in helpers {
        assert!(helper.sig.generics.params.is_empty());
        let matcher = helper
            .block
            .stmts
            .iter()
            .find_map(|stmt| {
                if let syn::Stmt::Item(syn::Item::Fn(matcher)) = stmt {
                    Some(matcher)
                } else {
                    None
                }
            })
            .expect("expected matcher");
        assert!(
            matcher.sig.generics.params.is_empty(),
            "no per-element monomorphization"
        );
    }
}
