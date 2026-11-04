use gpui_rsx_core::ClassMode;
use gpui_rsx_tools::{analyze_rust_document, release_span_memory, update_rust_document};

fn check_edits(initial: String, edits: impl IntoIterator<Item = String>) {
    let mut source = initial;
    let mut previous = analyze_rust_document(&source, ClassMode::PERMISSIVE);
    for edited in edits {
        let updated = update_rust_document(&edited, ClassMode::PERMISSIVE, &source, &previous);
        let full = analyze_rust_document(&edited, ClassMode::PERMISSIVE);
        assert_eq!(updated.calls, full.calls);
        assert_eq!(updated.components, full.components);
        assert_eq!(updated.literals, full.literals);
        assert_eq!(updated.class_tokens, full.class_tokens);
        assert_eq!(updated.diagnostics, full.diagnostics);
        source = edited;
        previous = updated;
        release_span_memory();
    }
}

#[test]
fn large_documents_reuse_repeated_bodies_after_insertions_deletions_and_moves() {
    let initial = (0..320).map(|index| {
        format!("fn view_{index}(){{rsx!{{<div class=\"flex bg-red-500 w-[-4px] hover:opacity-50\"/>}};}}\n")
    }).collect::<String>();
    let changed = initial.replacen("bg-red-500", "bg-blue-500", 1);
    let prefixed = format!("// 中文😀\n{changed}");
    let removed = prefixed.replacen(initial.lines().nth(160).unwrap(), "", 1);
    let inserted =
        format!("{removed}\nfn extra(){{rsx_strict!{{<span class=\"dark:flex nonsense\"/>}};}}");
    check_edits(
        initial.clone(),
        [changed, prefixed, removed, inserted, initial],
    );
}

#[test]
fn nested_modes_diagnostics_and_component_changes_keep_their_owners() {
    let initial = r##"#[component] struct Card { title: String }
fn view(){rsx!{<div class="flex-colum bg-red-500">{rsx_strict!{<Card class="w-[-4px] nonsense"/>}}{rsx_tw!{<span class="gap-4"/>}}</div>};}
fn other(){rsx!{<div class="flex-colum bg-red-500"/>};}"##.to_owned();
    let changed = initial.replacen("flex-colum", "flex-col", 1);
    let inner = changed.replace("nonsense", "hover:bg-blue-500");
    let components = inner.replace("title: String", "title: String, count: usize");
    let reordered = components.lines().rev().collect::<Vec<_>>().join("\n");
    check_edits(
        initial.clone(),
        [changed, inner, components, reordered, initial],
    );
}
