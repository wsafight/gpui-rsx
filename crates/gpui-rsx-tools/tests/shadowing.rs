use gpui_rsx_core::ClassMode;
use gpui_rsx_tools::{
    MACROS, analyze_rust_document, format, macro_calls, macro_invocations, nested_macro_calls,
    update_rust_document,
};

fn bodies(source: &str) -> Vec<&str> {
    nested_macro_calls(source)
        .unwrap()
        .into_iter()
        .map(|call| source[call.start..call.end].trim())
        .collect()
}

fn assert_untouched(source: &str) {
    assert!(macro_calls(source).unwrap().is_empty(), "{source}");
    assert!(nested_macro_calls(source).unwrap().is_empty(), "{source}");
    assert!(macro_invocations(source).unwrap().is_empty(), "{source}");
    let analysis = analyze_rust_document(source, ClassMode::STRICT);
    assert!(analysis.calls.is_empty(), "{source}");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.class_tokens.is_empty());
    assert!(analysis.literals.is_empty());
    assert_eq!(format::rust(source).unwrap(), source);
    assert_eq!(format::migrate_rust(source).unwrap(), source);
}

#[test]
fn qualified_calls_require_the_complete_gpui_rsx_path() {
    for macro_name in MACROS {
        for prefix in ["gpui_rsx", "::gpui_rsx"] {
            let source =
                format!("fn render(){{{prefix}::{macro_name}!{{<div class=\"gap-8\"/>}};}}");
            let calls = macro_calls(&source).unwrap();
            assert_eq!(calls.len(), 1, "{source}");
            assert_eq!(calls[0].name, *macro_name);
            assert_eq!(
                &source[calls[0].start..calls[0].end],
                "<div class=\"gap-8\"/>"
            );
        }
        for path in [
            format!("other_crate::{macro_name}"),
            format!("::other_crate::{macro_name}"),
            format!("other_crate::gpui_rsx::{macro_name}"),
            format!("gpui_rsx::nested::{macro_name}"),
            format!("::{macro_name}"),
        ] {
            let source = format!("fn render(){{{path}!{{<div class=\"gap-8 flex-colum\"/>}};}}");
            assert_untouched(&source);
        }
    }
}

#[test]
fn foreign_imports_block_every_bare_canonical_fallback() {
    for local in MACROS {
        for original in [*local, "widget"] {
            let source = format!(
                "use other_crate::{original} as {local}; fn render(){{{local}!{{<div class=\"gap-8 flex-colum\"/>}};}}"
            );
            assert_untouched(&source);
        }
        for prefix in ["other_crate", "::other_crate"] {
            let source = format!(
                "use {prefix}::{local}; fn render(){{{local}!{{<div class=\"gap-8 flex-colum\"/>}};}}"
            );
            assert_untouched(&source);
        }
    }
}

#[test]
fn foreign_imports_do_not_remove_legal_gpui_rsx_bindings() {
    let source = r#"use gpui_rsx::{rsx as view, rsx_tw};
use other_crate::rsx;
fn render(){
    view!{<div/>};
    rsx_tw!{<span/>};
    rsx!{<foreign/>};
    gpui_rsx::rsx!{<label/>};
    ::gpui_rsx::rsx_strict!{<section/>};
}"#;
    let calls = nested_macro_calls(source).unwrap();
    assert_eq!(
        calls
            .iter()
            .map(|call| call.name.as_str())
            .collect::<Vec<_>>(),
        ["rsx", "rsx_tw", "rsx", "rsx_strict"]
    );
    assert_eq!(
        bodies(source),
        ["<div/>", "<span/>", "<label/>", "<section/>"]
    );
}

#[test]
fn local_macros_shadow_every_exported_name_and_import_alias() {
    for original in MACROS {
        for local in [*original, "view"] {
            let source = format!(
                "use gpui_rsx::{original} as {local};\nmacro_rules! {local} {{ ($($tokens:tt)*) => {{ stringify!($($tokens)*) }}; }}\nfn render(){{{local}!{{<div class=\"gap-8 flex-colum\"/>}};}}"
            );
            assert_untouched(&source);
        }
    }
}

#[test]
fn shadowing_starts_at_the_definition_and_does_not_hide_qualified_calls() {
    let source = r#"use gpui_rsx::rsx_strict as view;
fn before(){view!{<div class="gap-8"/>};}
macro_rules! view { ($($tokens:tt)*) => { stringify!($($tokens)*) }; }
fn after(){view!{<div class="gap-12"/>};gpui_rsx::rsx!{<span class="gap-16"/>};}"#;
    assert_eq!(
        bodies(source),
        ["<div class=\"gap-8\"/>", "<span class=\"gap-16\"/>"]
    );
    let calls = macro_calls(source).unwrap();
    assert_eq!(calls[0].name, "rsx_strict");
    assert_eq!(calls[1].name, "rsx");
    assert_eq!(
        format::migrate_rust(source).unwrap(),
        source.replace("gap-8", "gap-2").replace("gap-16", "gap-4")
    );
}

#[test]
fn block_shadowing_is_textual_and_does_not_leak_to_siblings() {
    let source = r#"use gpui_rsx::rsx as view;
fn render(){
    { view!{<div/>}; macro_rules! view { () => { () }; } view!{ignored}; }
    { view!{<span/>}; }
    view!{<label/>};
}"#;
    assert_eq!(bodies(source), ["<div/>", "<span/>", "<label/>"]);
}

#[test]
fn modules_inherit_textual_shadowing_without_exporting_local_definitions() {
    let source = r#"use gpui_rsx::rsx as view;
mod local { macro_rules! view { () => { () }; } use gpui_rsx::rsx as view; fn render(){view!{ignored};} }
fn before(){view!{<div/>};}
macro_rules! view { () => { () }; }
mod child { use gpui_rsx::rsx as view; fn render(){view!{ignored};gpui_rsx::rsx!{<span/>};} }
fn after(){view!{ignored};}"#;
    assert_eq!(bodies(source), ["<div/>", "<span/>"]);
}

#[test]
fn definition_bodies_keep_real_rsx_but_not_recursive_shadowed_calls() {
    let source = r#"use gpui_rsx::{rsx as view, rsx_tw as tw};
macro_rules! view { () => { (view!{ignored}, tw!{<span class="gap-4"/>}) }; }
fn render(){view!{ignored};}"#;
    assert_eq!(bodies(source), ["<span class=\"gap-4\"/>"]);
    assert_eq!(nested_macro_calls(source).unwrap()[0].name, "rsx_tw");
    assert_eq!(format::migrate_rust(source).unwrap(), source);
}

#[test]
fn unicode_raw_identifiers_and_all_definition_delimiters_shadow_aliases() {
    for local in ["视图", "r#type", "rsx"] {
        for (open, close) in [("{", "}"), ("(", ")"), ("[", "]")] {
            let source = format!(
                "use gpui_rsx::rsx_strict as {local};\nmacro_rules! {local} {open} ($($tokens:tt)*) => {{ stringify!($($tokens)*) }}; {close};\nfn render(){{{local}!{{<div class=\"gap-8 flex-colum\"/>}};}}"
            );
            assert_untouched(&source);
        }
    }
}

#[test]
fn raw_and_plain_spellings_share_the_same_shadowed_binding() {
    for local in ["view", "rsx"] {
        let raw = format!("r#{local}");
        for imported in [local, raw.as_str()] {
            for defined in [local, raw.as_str()] {
                for called in [local, raw.as_str()] {
                    let source = format!(
                        "use gpui_rsx::rsx_strict as {imported};\nmacro_rules! {defined} {{ ($($tokens:tt)*) => {{ stringify!($($tokens)*) }}; }}\nfn render(){{{called}!{{<div class=\"gap-8 flex-colum\"/>}};}}"
                    );
                    assert_untouched(&source);
                }
            }
        }
    }
}

#[test]
fn raw_imports_and_calls_keep_canonical_modes_and_original_spellings() {
    let source = "use r#gpui_rsx::r#rsx_strict as r#view; fn render(){view!{<div/>};r#view!{<span/>};r#gpui_rsx::r#rsx_tw!{<label/>};}";
    let calls = macro_calls(source).unwrap();
    assert_eq!(
        calls
            .iter()
            .map(|call| call.name.as_str())
            .collect::<Vec<_>>(),
        ["rsx_strict", "rsx_strict", "rsx_tw"]
    );
    let spelled = macro_invocations(source).unwrap();
    assert_eq!(
        spelled
            .iter()
            .map(|call| call.name.as_str())
            .collect::<Vec<_>>(),
        ["view", "r#view", "r#rsx_tw"]
    );
    assert_eq!(bodies(source), ["<div/>", "<span/>", "<label/>"]);
}

#[test]
fn syntax_recovery_does_not_restore_a_shadowed_import() {
    let source = r#"use gpui_rsx::rsx_strict as view;
macro_rules! view { () => { () }; }
fn render(){view!{<div class="flex-colum"/>};gpui_rsx::rsx!{<span/>};}"#;
    for suffix in [
        "fn editing(",
        "}",
        "let text = \"unfinished",
        "/* unfinished",
    ] {
        let damaged = format!("{source}\n{suffix}");
        let analysis = analyze_rust_document(&damaged, ClassMode::STRICT);
        assert_eq!(bodies(&damaged), ["<span/>"]);
        assert_eq!(analysis.diagnostics.len(), 1, "{:?}", analysis.diagnostics);
        assert_eq!(
            analysis.diagnostics[0].code.as_deref(),
            Some("rsx/rust-syntax")
        );
    }
}

#[test]
fn incremental_nested_calls_keep_shadowing_from_outside_the_edited_body() {
    let mut source = "macro_rules! rsx { () => { () }; } fn render(){rsx_tw!{<div>{rsx!{<span class=\"flex-colum\"/>}}</div>};}".to_owned();
    let mut previous = analyze_rust_document(&source, ClassMode::STRICT);
    assert_eq!(previous.calls.len(), 1);
    for changed in [
        source.replace("flex-colum", "gap-4"),
        source.replace(
            "<span class=\"flex-colum\"/>",
            "<label>{\"中文😀\"}</label>",
        ),
        source.replace("macro_rules! rsx { () => { () }; } ", ""),
        source.clone(),
    ] {
        let updated = update_rust_document(&changed, ClassMode::STRICT, &source, &previous);
        let full = analyze_rust_document(&changed, ClassMode::STRICT);
        assert_eq!(updated.calls, full.calls, "{changed}");
        assert_eq!(updated.diagnostics, full.diagnostics, "{changed}");
        assert_eq!(updated.class_tokens, full.class_tokens);
        assert_eq!(updated.literals, full.literals);
        assert_eq!(updated.components, full.components);
        source = changed;
        previous = updated;
    }
}

#[test]
fn incremental_body_edits_cannot_bypass_foreign_imports() {
    for import in [
        "use other_crate::rsx;",
        "use ::other_crate::rsx;",
        "use other_crate::widget as rsx;",
    ] {
        let source = format!(
            "{import} fn render(){{rsx_tw!{{<div>{{rsx!{{<span class=\"gap-4\"/>}}}}</div>}};}}"
        );
        let previous = analyze_rust_document(&source, ClassMode::STRICT);
        assert_eq!(previous.calls.len(), 1, "{source}");
        let changed = source.replace("gap-4", "gap-8");
        let updated = update_rust_document(&changed, ClassMode::STRICT, &source, &previous);
        let full = analyze_rust_document(&changed, ClassMode::STRICT);
        assert_eq!(updated.calls, full.calls, "{changed}");
        assert_eq!(updated.calls.len(), 1, "{changed}");
        assert_eq!(updated.diagnostics, full.diagnostics, "{changed}");
        assert_eq!(updated.class_tokens, full.class_tokens, "{changed}");
        assert_eq!(updated.literals, full.literals, "{changed}");
    }
}
