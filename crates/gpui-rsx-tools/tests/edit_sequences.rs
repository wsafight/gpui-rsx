use gpui_rsx_core::ClassMode;
use gpui_rsx_tools::{RustAnalysis, analyze_rust_document, update_rust_document};

fn check_sequence(edits: impl IntoIterator<Item = String>) {
    let mut source = String::new();
    let mut analysis = analyze_rust_document(&source, ClassMode::PERMISSIVE);
    for changed in edits {
        let incremental = update_rust_document(&changed, ClassMode::PERMISSIVE, &source, &analysis);
        let full = analyze_rust_document(&changed, ClassMode::PERMISSIVE);
        assert_analysis_eq(&incremental, &full, &changed);
        for diagnostic in &incremental.diagnostics {
            assert!(diagnostic.start <= diagnostic.end && diagnostic.end <= changed.len());
            assert!(changed.is_char_boundary(diagnostic.start));
            assert!(changed.is_char_boundary(diagnostic.end));
        }
        source = changed;
        analysis = incremental;
    }
}

fn assert_analysis_eq(actual: &RustAnalysis, expected: &RustAnalysis, source: &str) {
    assert_eq!(actual.calls, expected.calls, "{source}");
    assert_eq!(actual.diagnostics, expected.diagnostics, "{source}");
    assert_eq!(actual.class_tokens, expected.class_tokens, "{source}");
    assert_eq!(actual.literals, expected.literals, "{source}");
    assert_eq!(actual.components, expected.components, "{source}");
}

#[test]
fn typing_and_backspacing_every_unicode_boundary_matches_full_analysis() {
    let source = "use gpui_rsx::rsx_strict as 视图;\nfn render(){视图!{<div class=\"flex-colum\">{\"排查😀é\"}</div>};}";
    let prefixes = source
        .char_indices()
        .map(|(offset, _)| source[..offset].to_owned())
        .chain(std::iter::once(source.to_owned()))
        .collect::<Vec<_>>();
    check_sequence(
        prefixes
            .iter()
            .cloned()
            .chain(prefixes.iter().rev().cloned()),
    );
}

#[test]
fn replacing_imports_and_recovering_syntax_never_reuses_stale_modes() {
    let strict =
        "use gpui_rsx::rsx_strict as view; fn render(){view!{<div class=\"flex-colum gap-4\"/>};}";
    check_sequence([
        strict.to_owned(),
        strict.replace("rsx_strict", "rsx_tw"),
        format!("{strict}\nfn unfinished("),
        strict.replace("rsx_strict", "rsx_permissive"),
        strict.replace(
            "use gpui_rsx::rsx_strict as view;",
            "use other::widget as view;",
        ),
        strict.to_owned(),
        String::new(),
        strict.to_owned(),
    ]);
}

#[test]
fn edits_in_nested_aliases_and_unrelated_macros_keep_all_ranges_in_sync() {
    let source = "use gpui_rsx::{rsx as view, rsx_tw as tw}; fn render(){view!{<div>{tw!{<span class=\"gap-4\"/>}}</div>};view!{<label/>};}";
    check_sequence([
        source.to_owned(),
        source.replace("gap-4", "gap-12 text-red-500"),
        source.replace("<span class=\"gap-4\"/>", "<span>{\"中文😀\"}</span>"),
        source.replace("{tw!{<span class=\"gap-4\"/>}}", "{\"plain\"}"),
        source.replace("rsx_tw as tw", "rsx_strict as tw"),
        source.to_owned(),
    ]);
}

#[test]
fn adding_and_removing_shadowing_restores_import_modes_and_diagnostics() {
    let source =
        "use gpui_rsx::rsx_strict as view; fn render(){view!{<div class=\"flex-colum gap-4\"/>};}";
    let shadowed = source.replace(
        "fn render()",
        "macro_rules! view { () => { () }; } fn render()",
    );
    check_sequence([
        source.to_owned(),
        shadowed.clone(),
        shadowed.replace("flex-colum", "flex-col"),
        format!("{shadowed}\nfn editing("),
        source.to_owned(),
        source.replace("rsx_strict", "rsx_tw"),
    ]);
}
