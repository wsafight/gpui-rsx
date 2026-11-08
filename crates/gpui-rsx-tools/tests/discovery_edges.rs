use gpui_rsx_core::ClassMode;
use gpui_rsx_tools::{
    DiagnosticSeverity, MACROS, analyze_rust_document, format, macro_calls, macro_invocations,
    mode_for, nested_macro_calls,
};

#[test]
fn every_exported_macro_keeps_its_mode_when_renamed() {
    for macro_name in MACROS {
        let source = format!(
            "use ::gpui_rsx::{{{macro_name} as view}}; fn render(){{view!{{<div class=\"gap-4\"/>}};}}"
        );
        let calls = macro_calls(&source).unwrap();
        assert_eq!(calls.len(), 1, "{macro_name}");
        assert_eq!(calls[0].name, *macro_name);
        assert_eq!(
            &source[calls[0].start..calls[0].end],
            "<div class=\"gap-4\"/>"
        );
        for default in [ClassMode::STRICT, ClassMode::PERMISSIVE] {
            let aliased = mode_for(&calls[0].name, default.clone());
            let original = mode_for(macro_name, default);
            assert_eq!(aliased.is_strict(), original.is_strict());
            assert_eq!(aliased.scale(), original.scale());
        }
    }
}

#[test]
fn later_imports_and_nested_groups_obey_rust_item_scope() {
    let source = r#"fn outer(){view!{<div/>}; {
        use gpui_rsx::{rsx_strict as view}; view!{<span/>};
    } view!{<label/>};}
    use gpui_rsx::{self, rsx_tw as view};"#;
    let names = nested_macro_calls(source)
        .unwrap()
        .into_iter()
        .map(|call| call.name)
        .collect::<Vec<_>>();
    assert_eq!(names, ["rsx_tw", "rsx_strict", "rsx_tw"]);
}

#[test]
fn alias_imports_in_literals_and_comments_do_not_change_bindings() {
    let source = r##"use gpui_rsx::rsx_strict as view;
    const NOTE: &str = r#"use gpui_rsx::rsx_tw as view; view!{ignored}"#;
    /* use other::widget as view; */
    fn render(){view!{<div class="flex-colum"/>};}"##;
    let analysis = analyze_rust_document(source, ClassMode::PERMISSIVE);
    assert_eq!(analysis.calls.len(), 1);
    assert_eq!(analysis.calls[0].name, "rsx_strict");
    assert!(analysis.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == DiagnosticSeverity::Error
            && diagnostic.suggestion.as_deref() == Some("flex-col")
    }));
}

#[test]
fn unicode_and_raw_identifiers_preserve_alias_spellings_and_ranges() {
    let source = "use gpui_rsx::{rsx as 视图, rsx_tw as r#type};\nfn render(){视图!{<div>{r#type!{<span/>}}</div>};}";
    let outer = macro_calls(source).unwrap();
    let nested = nested_macro_calls(source).unwrap();
    let spelled = macro_invocations(source).unwrap();
    assert_eq!(outer.len(), 1);
    assert_eq!(nested.len(), 2);
    assert_eq!(spelled[0].name, "视图");
    assert_eq!(spelled[1].name, "r#type");
    assert_eq!(nested[1].name, "rsx_tw");
    assert_eq!(&source[nested[1].start..nested[1].end], "<span/>");
    assert!(nested[0].start < nested[1].start && nested[1].end < nested[0].end);
    for (canonical, original) in nested.iter().zip(&spelled) {
        assert_eq!(
            (canonical.start, canonical.end),
            (original.start, original.end)
        );
    }
}

#[test]
fn unrelated_crate_aliases_and_child_modules_do_not_inherit_bindings() {
    let source = r#"use gpui_rsx::rsx as view;
    mod child { fn render(){view!{ignored};} }
    mod sibling { use other::widget as view; fn render(){view!{ignored};} }
    fn outer(){use gpui_rsx as markup; use markup::rsx as other_view;
        other_view!{ignored}; view!{<div/>};}"#;
    let calls = macro_calls(source).unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(&source[calls[0].start..calls[0].end], "<div/>");
}

#[test]
fn recovery_hides_unfinished_literals_and_nested_comments() {
    for suffix in [
        "let text = r##\"view!{ignored}",
        "let text = b\"view!{ignored}",
        "let text = br#\"view!{ignored}",
        "let character = '\\u{4e2d} view!{ignored}",
        "/* outer /* inner */ view!{ignored}",
    ] {
        let source =
            format!("use gpui_rsx::rsx as view; fn render(){{view!{{<div/>}};}}\n{suffix}");
        let analysis = analyze_rust_document(&source, ClassMode::PERMISSIVE);
        assert_eq!(analysis.calls.len(), 1, "{suffix}");
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code.as_deref() == Some("rsx/rust-syntax") }),
            "{suffix}"
        );
        let formatted = format::rust(&source).unwrap();
        assert!(formatted.ends_with(suffix));
        assert!(format::check_rust_tokens(&source).is_err());
    }
}

#[test]
fn recovery_before_a_macro_preserves_crlf_and_unicode_offsets() {
    let source = "}\r\n// 中文😀\r\nuse gpui_rsx::rsx_strict as view;\r\nfn render(){view!{<div class=\"flex-colum\"/>};}";
    let analysis = analyze_rust_document(source, ClassMode::PERMISSIVE);
    assert_eq!(analysis.calls.len(), 1);
    let syntax = analysis
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_deref() == Some("rsx/rust-syntax"))
        .unwrap();
    assert_eq!(&source[syntax.start..syntax.end], "}");
    let class = analysis
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.suggestion.as_deref() == Some("flex-col"))
        .unwrap();
    assert_eq!(&source[class.start..class.end], "flex-colum");
    let formatted = format::rust(source).unwrap();
    assert!(formatted.starts_with("}\r\n// 中文😀\r\n"));
}

#[test]
fn exceeding_the_repair_budget_returns_a_bounded_error() {
    let source = format!("{}fn render(){{rsx!{{<div/>}};}}", ")".repeat(64));
    let analysis = analyze_rust_document(&source, ClassMode::PERMISSIVE);
    assert!(analysis.calls.is_empty());
    assert_eq!(analysis.diagnostics.len(), 1);
    let diagnostic = &analysis.diagnostics[0];
    assert_eq!(diagnostic.code.as_deref(), Some("rsx/rust-syntax"));
    assert_eq!(&source[diagnostic.start..diagnostic.end], ")");
    assert!(macro_calls(&source).is_err());
}
