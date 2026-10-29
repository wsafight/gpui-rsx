//! Markup completions use GPUI's generated method catalog and local component signatures.
use super::document::Document;
use gpui_rsx_tools::{
    call_at,
    completion::{self, CompletionContext},
    components,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(super) fn complete(document: &Document, cursor: usize) -> Value {
    let empty = json!({"isIncomplete":false,"items":[]});
    let Some(call) = call_at(&document.analysis.calls, cursor) else {
        return empty;
    };
    let Some(context) =
        completion::context(&document.source[call.start..call.end], cursor - call.start)
    else {
        return empty;
    };
    let mut candidates = BTreeMap::new();
    let (start, end) = match context {
        CompletionContext::Tag { start, end } => {
            for &name in gpui_rsx_core::codegen::tables::HTML_TAGS {
                candidates.insert(name.to_owned(), (10, "GPUI element".to_owned()));
            }
            for c in &document.analysis.components {
                candidates.insert(c.name.clone(), (7, "Local #[component]".to_owned()));
            }
            (start, end)
        }
        CompletionContext::Attribute { tag, start, end } => {
            for name in [
                "class",
                "id",
                "key",
                "base",
                "when",
                "whenSome",
                "whenClass",
                "hoverClass",
                "activeClass",
                "focusClass",
                "onUnknownClass",
            ] {
                candidates.insert(name.to_owned(), (10, "RSX attribute".to_owned()));
            }
            if let Some(component) = components::find(&document.analysis.components, &tag) {
                for p in &component.props {
                    candidates.insert(
                        p.name.clone(),
                        (
                            5,
                            format!("{}{}", p.ty, if p.required { " (required)" } else { "" }),
                        ),
                    );
                    if p.flag {
                        candidates
                            .insert(format!("{}_if", p.name), (5, "bool condition".to_owned()));
                    }
                }
            } else {
                for method in gpui_rsx_core::codegen::tables::GPUI_METHODS {
                    let mut parts = method.name.split('_');
                    let mut camel = parts.next().unwrap_or_default().to_owned();
                    for part in parts {
                        let mut chars = part.chars();
                        if let Some(c) = chars.next() {
                            camel.push(c.to_ascii_uppercase());
                            camel.push_str(chars.as_str());
                        }
                    }
                    candidates.insert(camel, (2, format!("GPUI method {}", method.name)));
                    candidates.insert(
                        method.name.to_owned(),
                        (2, format!("GPUI method ({} arguments)", method.arity)),
                    );
                }
            }
            (start, end)
        }
    };
    let start = call.start + start;
    let end = call.start + end;
    let prefix = &document.source[start..cursor];
    let matches: Vec<_> = candidates
        .into_iter()
        .filter(|(name, _)| name.starts_with(prefix))
        .collect();
    json!({"isIncomplete":matches.len()>200,"items":matches.into_iter().take(200).map(|(name, (kind, detail))| json!({"label":name,"kind":kind,"detail":detail,"textEdit":{"range":document.range(start,end),"newText":name}})).collect::<Vec<_>>()})
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_rsx_core::ClassMode;
    fn labels(source: &str, needle: &str) -> Vec<String> {
        let document = Document::with_mode(source, ClassMode::PERMISSIVE);
        let result = complete(&document, source.find(needle).unwrap() + needle.len());
        result["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["label"].as_str().unwrap().to_owned())
            .collect()
    }
    #[test]
    fn completes_local_tags_props_and_gpui_methods_while_editing() {
        let component = "#[component] fn Card(title: String, #[prop(default)] compact: bool) -> impl IntoElement { div() } ";
        let source = format!("{component} fn f(){{rsx!{{<Ca}};}}");
        assert!(labels(&source, "<Ca").contains(&"Card".to_owned()));
        let source = format!("{component} fn f(){{rsx!{{<Card tit}};}}");
        assert_eq!(labels(&source, "<Card tit"), ["title"]);
        let source = format!("{component} fn f(){{rsx!{{<Card compact_}};}}");
        assert_eq!(labels(&source, "compact_"), ["compact_if"]);
        assert!(labels("fn f(){rsx!{<div on_cl};}", "on_cl").contains(&"on_click".to_owned()));
    }
}
