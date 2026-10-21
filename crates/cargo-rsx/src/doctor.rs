use serde_json::{Value, json};

const ENGINES: &[&str] = &["gpui", "gpui-pre", "gpui-fast"];

/// Analyze resolved identities, including dependency aliases. Valid for Git and registry GPUI.
pub fn inspect(metadata: &Value) -> Value {
    let packages = metadata["packages"].as_array().cloned().unwrap_or_default();
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let engines: Vec<_> = packages
        .iter()
        .filter(|p| ENGINES.contains(&p["name"].as_str().unwrap_or("")))
        .collect();
    let mut errors = Vec::new();
    let mut warnings = Vec::<String>::new();
    if engines.len() > 1 {
        errors.push(format!("Found {} GPUI identities; their Rust types are incompatible. Pin one source/version across dependencies.", engines.len()));
    }
    if engines.is_empty() {
        warnings.push("No GPUI runtime found. This is normal for macro/tooling crates.".into());
    }
    let mut consumers = Vec::new();
    for package in &packages {
        let name = package["name"].as_str().unwrap_or("");
        if !matches!(name, "gpui-kit" | "gpui-component" | "gpui-base") {
            continue;
        }
        if let Some(node) = nodes.iter().find(|n| n["id"] == package["id"]) {
            let deps = node["deps"].as_array().cloned().unwrap_or_default();
            let resolved: Vec<_> = deps
                .iter()
                .filter(|dep| engines.iter().any(|engine| engine["id"] == dep["pkg"]))
                .map(|dep| dep["pkg"].clone())
                .collect();
            if resolved.len() != 1 {
                errors.push(format!(
                    "{name} resolves {} GPUI backends; expected one.",
                    resolved.len()
                ));
            }
            consumers.push(json!({"package": package["id"], "gpui": resolved}));
        }
    }
    for engine in &engines {
        let engine_name = engine["name"].as_str().unwrap_or("");
        let companion_names: &[&str] = match engine_name {
            "gpui-pre" => &["gpui-pre-macros", "gpui-pre-platform"],
            "gpui-fast" => &["gpui-fast-macros", "gpui-fast-platform"],
            _ => &["gpui_macros", "gpui_platform"],
        };
        for companion in packages
            .iter()
            .filter(|p| companion_names.contains(&p["name"].as_str().unwrap_or("")))
        {
            if companion["source"] != engine["source"] || companion["version"] != engine["version"]
            {
                errors.push(format!(
                    "{} and {engine_name} must use matching source/version.",
                    companion["name"].as_str().unwrap_or("")
                ));
            }
        }
    }
    json!({"ok": errors.is_empty(), "engines": engines.iter().map(|p| json!({"id": p["id"], "name": p["name"], "version": p["version"], "source": p["source"]})).collect::<Vec<_>>(), "consumers": consumers, "errors": errors, "warnings": warnings})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_mixed_backends_even_when_aliased() {
        let graph = json!({"packages":[{"name":"gpui-pre","id":"pre","version":"0.3.9","source":"registry+x"},{"name":"gpui","id":"git","version":"0.2.2","source":"git+x"}]});
        assert_eq!(inspect(&graph)["ok"], false);
    }
    #[test]
    fn accepts_git_identity_and_rejects_mismatched_macros() {
        let mut graph = json!({"packages":[{"name":"gpui","id":"g","version":"0.2.2","source":"git+same#sha"},{"name":"gpui_macros","id":"m","version":"0.2.2","source":"git+same#sha"}]});
        assert_eq!(inspect(&graph)["ok"], true);
        graph["packages"][1]["source"] = json!("git+other#sha");
        assert_eq!(inspect(&graph)["ok"], false);
    }
}
