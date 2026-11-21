use gpui_rsx_core::{ClassMode, RsxBody, SemanticDiagnosticCode, generate_body_with_diagnostics};

#[test]
fn custom_constructor_paths_and_base_expressions_bypass_native_requirements() {
    let source =
        "<><ui::img/><img base={make_image()}/><canvas prepaint={prepaint} paint={paint}/></>";
    let body: RsxBody = syn::parse_str(source).unwrap();
    let expansion = generate_body_with_diagnostics(&body, ClassMode::STRICT);
    assert!(expansion.diagnostics.is_empty());
    assert!(!expansion.tokens.to_string().contains("compile_error"));
}

#[test]
fn explicit_keys_and_ids_fix_nested_loop_errors() {
    let source = "<div>{for row in rows {<><button key={row.id} onClick={handler}/><div id={row.id} class=\"active:bg-red-500\"/></>}}</div>";
    let body: RsxBody = syn::parse_str(source).unwrap();
    let expansion = generate_body_with_diagnostics(&body, ClassMode::STRICT);
    assert!(expansion.diagnostics.is_empty());
    assert!(!expansion.tokens.to_string().contains("compile_error"));
}

#[test]
fn sibling_errors_share_codes_spans_and_compile_messages() {
    let source = "<><img/><canvas/>{for row in rows {<button onClick={handler}/><div onClick={handler}/>}}</>";
    let body: RsxBody = syn::parse_str(source).unwrap();
    let expansion = generate_body_with_diagnostics(&body, ClassMode::PERMISSIVE);
    assert_eq!(expansion.diagnostics.len(), 4);
    assert_eq!(
        expansion
            .tokens
            .to_string()
            .matches("compile_error")
            .count(),
        4
    );
    let codes = expansion
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert_eq!(
        codes,
        [
            SemanticDiagnosticCode::MissingRequiredAttribute,
            SemanticDiagnosticCode::MissingRequiredAttribute,
            SemanticDiagnosticCode::MissingLoopKey,
            SemanticDiagnosticCode::MissingLoopKey
        ]
    );
    let tags = expansion
        .diagnostics
        .iter()
        .map(|diagnostic| &source[diagnostic.span.byte_range()])
        .collect::<Vec<_>>();
    assert_eq!(tags, ["img", "canvas", "button", "div"]);
    for diagnostic in expansion.diagnostics {
        assert_eq!(
            syn::parse2::<syn::ItemMacro>(diagnostic.to_compile_error())
                .unwrap()
                .mac
                .path
                .segments
                .last()
                .unwrap()
                .ident,
            "compile_error"
        );
        assert!(!diagnostic.message.is_empty());
    }
}

#[test]
fn uniform_list_and_composable_class_arrays_participate_in_semantic_diagnostics() {
    let source = "<><uniform_list/><uniform_list count={10}/>{for item in items {<uniform_list count={10} render={render_rows}/><div class={[\"flex\", active && \"active:opacity-75\"]}/>}}</>";
    let body: RsxBody = syn::parse_str(source).unwrap();
    let expansion = generate_body_with_diagnostics(&body, ClassMode::STRICT);
    let codes: Vec<_> = expansion.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(
        codes,
        [
            SemanticDiagnosticCode::MissingRequiredAttribute,
            SemanticDiagnosticCode::MissingRequiredAttribute,
            SemanticDiagnosticCode::MissingLoopKey,
            SemanticDiagnosticCode::MissingLoopKey,
        ]
    );
    assert!(
        expansion.diagnostics[0]
            .message
            .contains("requires `count`")
    );
    assert!(
        expansion.diagnostics[1]
            .message
            .contains("requires `render`")
    );

    // Keyed uniform_list and keyed stateful class array pass cleanly.
    let valid = "<div>{for item in items {<uniform_list key={item.id} count={10} render={render_rows}/><div key={item.id} class={[\"flex\", active && \"active:opacity-75\"]}/>}}</div>";
    let valid_body: RsxBody = syn::parse_str(valid).unwrap();
    let valid_expansion = generate_body_with_diagnostics(&valid_body, ClassMode::STRICT);
    assert!(valid_expansion.diagnostics.is_empty());
}

#[test]
fn constructor_args_reject_base_conflicts_and_native_tags() {
    let conflict = syn::parse_str::<RsxBody>(r#"<Button("save") base={Button::new("other")} />"#)
        .err()
        .expect("conflicting args and base must fail")
        .to_string();
    assert!(
        conflict.contains("Cannot combine constructor arguments"),
        "{conflict}"
    );

    for native in [r#"<div("id") />"#, r#"<svg("icon.svg") />"#] {
        let body: RsxBody = syn::parse_str(native).unwrap();
        let expansion = generate_body_with_diagnostics(&body, ClassMode::PERMISSIVE);
        let tokens = expansion.tokens.to_string();
        assert!(tokens.contains("compile_error"), "{native}: {tokens}");
        assert!(
            tokens.contains("does not accept parenthesized constructor arguments"),
            "{native}: {tokens}"
        );
    }
}

#[test]
fn inline_dynamic_helper_includes_static_theme_tokens() {
    let theme = std::sync::Arc::new(
        gpui_rsx_core::Theme::parse(
            "[colors]\nbrand = \"#2563eb\"\n[radius]\ncard = \"12px\"\n[spacing]\ngutter = \"18px\"\n",
        )
        .unwrap(),
    );
    let mode = ClassMode::STRICT.with_theme(Some(theme));
    let expanded = gpui_rsx_core::expand_str("<div class={cls} />", mode)
        .unwrap()
        .to_string();
    assert!(expanded.contains("\"bg-brand\""), "{expanded}");
    assert!(expanded.contains("\"rounded-card\""), "{expanded}");
    assert!(expanded.contains("\"p-gutter\""), "{expanded}");
}
