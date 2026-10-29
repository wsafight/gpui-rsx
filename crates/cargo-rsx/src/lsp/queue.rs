//! Combine queued edits while preserving versions, incremental ranges and request ordering.
use super::{document::Document, protocol::apply_content_changes};
use serde_json::{Value, json};
use std::collections::HashMap;

pub(super) fn merge(left: &mut Value, right: &Value, docs: &HashMap<String, Document>) -> bool {
    if left["method"] != "textDocument/didChange" || right["method"] != left["method"] {
        return false;
    }
    let a = &left["params"]["textDocument"];
    let b = &right["params"]["textDocument"];
    let Some(uri) = a["uri"].as_str().filter(|uri| b["uri"] == *uri) else {
        return false;
    };
    let old = docs.get(uri);
    if a["version"]
        .as_i64()
        .zip(old.and_then(|doc| doc.version))
        .is_some_and(|(new, old)| new <= old)
    {
        return false;
    }
    if b["version"]
        .as_i64()
        .zip(a["version"].as_i64())
        .is_some_and(|(new, old)| new <= old)
    {
        return true;
    }
    let Some(first) = left["params"]["contentChanges"].as_array() else {
        return false;
    };
    let Some(second) = right["params"]["contentChanges"].as_array() else {
        return false;
    };
    let Ok(text) = apply_content_changes(old.map_or("", |doc| &doc.source), first)
        .and_then(|text| apply_content_changes(&text, second))
    else {
        return false;
    };
    left["params"]["textDocument"]["version"] = b["version"].clone();
    left["params"]["contentChanges"] = json!([{"text":text}]);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn combines_incremental_ranges_and_keeps_the_latest_version() {
        let uri = "file:///test.rs";
        let mut docs = HashMap::new();
        let mut doc = Document::with_mode("abc", gpui_rsx_core::ClassMode::PERMISSIVE);
        doc.version = Some(1);
        docs.insert(uri.into(), doc);
        let change = |version, from, to, text| json!({"method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":version},"contentChanges":[{"range":{"start":{"line":0,"character":from},"end":{"line":0,"character":to}},"text":text}]}});
        let mut left = change(2, 1, 2, "😀");
        assert!(merge(&mut left, &change(3, 3, 4, "!"), &docs));
        assert_eq!(left["params"]["contentChanges"][0]["text"], "a😀!");
        assert_eq!(left["params"]["textDocument"]["version"], 3);
        assert!(merge(&mut left, &change(2, 0, 0, "stale"), &docs));
        assert!(!merge(
            &mut left,
            &json!({"method":"shutdown","id":4}),
            &docs
        ));
    }

    #[test]
    fn ignores_changes_without_a_document_object() {
        let malformed = json!({"method":"textDocument/didChange","params":{"textDocument":"x","contentChanges":[{"text":"a"}]}});
        let mut left = malformed.clone();
        assert!(!merge(&mut left, &malformed, &HashMap::new()));
    }
}
