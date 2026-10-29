use gpui_rsx_core::ClassMode;
use gpui_rsx_tools::{
    analyze_body, analyze_rust_document,
    completion::{CompletionContext, context},
    update_rust_document,
};

#[test]
fn local_props_and_quick_fixes_produce_valid_rsx() {
    let source =
        "#[component] struct Card { first_name: String } fn f(){rsx!{<Card firstName={name}/>};}";
    assert!(
        analyze_rust_document(source, ClassMode::PERMISSIVE)
            .diagnostics
            .is_empty()
    );
    let source = r#"#[component] fn Card(title: String, #[prop(default)] active: bool) -> impl IntoElement { div() } fn view(){rsx!{<Card/>};}"#;
    let analysis = analyze_rust_document(source, ClassMode::PERMISSIVE);
    assert_eq!(analysis.components[0].props.len(), 2);
    assert!(analysis.components[0].props[1].flag);
    let d = analysis
        .diagnostics
        .iter()
        .find(|d| d.message.contains("required prop"))
        .unwrap();
    let mut fixed = source.to_owned();
    fixed.replace_range(d.start..d.end, d.suggestion.as_ref().unwrap());
    assert!(
        analyze_rust_document(&fixed, ClassMode::PERMISSIVE)
            .diagnostics
            .is_empty()
    );
    for body in [
        "<div></span>",
        "<div>{for row in rows { <div focusable/> }}</div>",
    ] {
        let diagnostics = analyze_body(body, ClassMode::PERMISSIVE);
        let d = diagnostics.iter().find(|d| d.suggestion.is_some()).unwrap();
        let mut fixed = body.to_owned();
        fixed.replace_range(d.start..d.end, d.suggestion.as_ref().unwrap());
        assert!(
            analyze_body(&fixed, ClassMode::PERMISSIVE).is_empty(),
            "{fixed}"
        );
    }
}

#[test]
fn completion_recognizes_incomplete_markup_and_ignores_rust() {
    let loop_body = "<div>{for item in items { <span cla /> }}</div>";
    let cursor = loop_body.find("cla").unwrap() + 3;
    assert_eq!(
        context(loop_body, cursor),
        Some(CompletionContext::Attribute {
            tag: "span".into(),
            start: cursor - 3,
            end: cursor
        })
    );
    assert_eq!(
        context("< Card", 1),
        Some(CompletionContext::Tag { start: 1, end: 6 })
    );
    assert_eq!(context("< /* comment */ Card", 6), None);
    assert_eq!(
        context("<Car", 4),
        Some(CompletionContext::Tag { start: 1, end: 4 })
    );
    assert_eq!(
        context("<ui::Card tit", 13),
        Some(CompletionContext::Attribute {
            tag: "ui::Card".into(),
            start: 10,
            end: 13
        })
    );
    for (source, cursor) in [
        ("<div id=\"abc\"/>", 10),
        ("<div id={foo.bar}/>", 13),
        ("<div // comment\n/>", 14),
        ("<div />", 7),
    ] {
        assert!(context(source, cursor).is_none(), "{source}");
    }
}

#[test]
fn incremental_cache_tracks_class_kinds_colors_and_component_changes() {
    let source =
        "fn f(){rsx!{<div class=\"flex bg-red-500/50\"/>};rsx_strict!{<div class=\"bad\"/>};}";
    let previous = analyze_rust_document(source, ClassMode::PERMISSIVE);
    assert_eq!(previous.class_tokens[1].rgba, Some(0xef444480));
    assert_eq!(
        previous.class_tokens[2].kind,
        gpui_rsx_tools::ClassTokenKind::Invalid
    );
    let edited = source.replace("flex", "flex-col");
    let incremental = update_rust_document(&edited, ClassMode::PERMISSIVE, source, &previous);
    let full = analyze_rust_document(&edited, ClassMode::PERMISSIVE);
    assert_eq!(incremental.class_tokens, full.class_tokens);
    let changed = "#[component] struct Card{title:String} fn f(){rsx!{<Card/>};}";
    assert!(
        update_rust_document(changed, ClassMode::PERMISSIVE, source, &previous)
            .diagnostics
            .iter()
            .any(|d| d.message.contains("title"))
    );
}
