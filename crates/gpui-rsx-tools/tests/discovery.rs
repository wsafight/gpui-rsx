use gpui_rsx_core::{ClassMode, SpacingScale};
use gpui_rsx_tools::{
    analyze_rust_document, expand, format, macro_calls, mode_for, nested_macro_calls, preview,
    update_rust_document,
};

#[test]
fn aliases_keep_the_original_macro_modes_across_tools() {
    let source = r#"use gpui_rsx::{rsx_strict as view, rsx_tw as tw};
fn render(){view!{<div class="flex-colum"/>};tw!{<div class="gap-4"/>};}"#;
    let calls = macro_calls(source).unwrap();
    assert_eq!(
        gpui_rsx_tools::macro_invocations(source)
            .unwrap()
            .iter()
            .map(|call| call.name.as_str())
            .collect::<Vec<_>>(),
        ["view", "tw"]
    );
    assert_eq!(
        calls
            .iter()
            .map(|call| call.name.as_str())
            .collect::<Vec<_>>(),
        ["rsx_strict", "rsx_tw"]
    );
    let analysis = analyze_rust_document(source, ClassMode::PERMISSIVE);
    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("flex-colum"))
    );
    let tailwind = &calls[1];
    let mode = mode_for(&tailwind.name, ClassMode::PERMISSIVE);
    assert_eq!(mode.scale(), SpacingScale::Tailwind);
    let body = &source[tailwind.start..tailwind.end];
    assert!(expand(body, mode.clone()).unwrap().contains("16"));
    assert!(preview::render(body, mode).unwrap().contains("16px"));
    let formatted = format::rust(source).unwrap();
    assert!(formatted.contains("view!{"));
    assert!(formatted.contains("<div class=\"flex-colum\" />"));
    assert_eq!(format::rust(&formatted).unwrap(), formatted);
    let migrated = format::migrate_rust(source).unwrap();
    assert!(migrated.contains("tw!{<div class=\"gap-4\"/>}"));
}

#[test]
fn aliases_respect_blocks_modules_shadowing_and_qualified_calls() {
    let source = r#"use gpui_rsx::rsx_strict as view;
fn outer(){view!{<div/>}; {use other::widget as view;view!{ignored};} other::view!{ignored};}
mod child {fn render(){view!{ignored};} use gpui_rsx::rsx_tw as local;fn inner(){local!{<div/>};}}
fn sibling(){view!{<span/>};}"#;
    let calls = nested_macro_calls(source).unwrap();
    assert_eq!(
        calls
            .iter()
            .map(|call| call.name.as_str())
            .collect::<Vec<_>>(),
        ["rsx_strict", "rsx_tw", "rsx_strict"]
    );
}

#[test]
fn alias_names_do_not_override_a_qualified_macro_mode() {
    let source = "use gpui_rsx::rsx_strict as rsx_tw; fn render(){rsx_tw!{<div/>};gpui_rsx::rsx_tw!{<div/>};}";
    let calls = macro_calls(source).unwrap();
    assert_eq!(calls[0].name, "rsx_strict");
    assert_eq!(calls[1].name, "rsx_tw");
}

#[test]
fn incomplete_rust_keeps_balanced_macros_and_byte_positions() {
    for suffix in [
        "fn editing(",
        "fn editing(){",
        "}",
        "let text = \"unfinished",
    ] {
        let source =
            format!("// 中文😀\nfn render(){{rsx!{{<div class=\"flex-colum\"/>}};}}\n{suffix}");
        let analysis = analyze_rust_document(&source, ClassMode::STRICT);
        assert_eq!(analysis.calls.len(), 1, "{suffix}");
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code.as_deref() == Some("rsx/rust-syntax"))
        );
        let class = analysis
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("flex-colum"))
            .unwrap();
        assert_eq!(&source[class.start..class.end], "flex-colum");
        let formatted = format::rust(&source).unwrap();
        assert!(formatted.ends_with(suffix));
        assert!(formatted.contains("<div class=\"flex-colum\" />"));
    }
}

#[test]
fn recovery_does_not_discover_macros_in_literals_or_comments() {
    let source = r##"use gpui_rsx::rsx as view;
fn render(){view!{<span>{r#"rsx!{ignored}"#}</span>};}
// rsx!{ignored}
let text = "rsx!{ignored}";
fn editing("##;
    assert_eq!(macro_calls(source).unwrap().len(), 1);
    let unfinished = "fn render(){rsx!{<div/>};} let text = \"rsx!{ignored}";
    assert_eq!(macro_calls(unfinished).unwrap().len(), 1);
    let comment = "fn render(){rsx!{<div/>};} /* rsx!{ignored}";
    assert_eq!(macro_calls(comment).unwrap().len(), 1);
}

#[test]
fn alias_and_recovery_updates_match_full_analysis() {
    let mut source =
        "use gpui_rsx::rsx_strict as view; fn render(){view!{<div>{view!{<span/>}}</div>};}"
            .to_owned();
    let mut previous = analyze_rust_document(&source, ClassMode::PERMISSIVE);
    for changed in [
        source.replace("<span/>", "<span class=\"flex-colum\"/>"),
        format!("{source}\nfn editing("),
        source.replace("rsx_strict", "rsx_tw"),
        source.clone(),
    ] {
        let updated = update_rust_document(&changed, ClassMode::PERMISSIVE, &source, &previous);
        let full = analyze_rust_document(&changed, ClassMode::PERMISSIVE);
        assert_eq!(updated.calls, full.calls);
        assert_eq!(updated.diagnostics, full.diagnostics);
        assert_eq!(updated.class_tokens, full.class_tokens);
        assert_eq!(updated.literals, full.literals);
        source = changed;
        previous = updated;
    }
}
