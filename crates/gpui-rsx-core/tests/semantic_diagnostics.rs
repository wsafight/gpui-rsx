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
