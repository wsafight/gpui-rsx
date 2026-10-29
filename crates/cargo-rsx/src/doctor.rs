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

    #[test]
    fn reports_a_missing_engine_as_a_warning_not_an_error() {
        let report = inspect(&json!({"packages":[]}));
        assert_eq!(report["ok"], true);
        assert_eq!(report["engines"].as_array().unwrap().len(), 0);
        assert!(
            report["warnings"][0]
                .as_str()
                .unwrap()
                .contains("No GPUI runtime")
        );
    }

    #[test]
    fn checks_companion_crates_for_each_engine() {
        for (engine, companion) in [
            ("gpui", "gpui_macros"),
            ("gpui-pre", "gpui-pre-macros"),
            ("gpui-fast", "gpui-fast-platform"),
        ] {
            let matching = json!({"packages":[
                {"name": engine, "id": "e", "version": "1.0.0", "source": "registry+x"},
                {"name": companion, "id": "c", "version": "1.0.0", "source": "registry+x"}
            ]});
            assert_eq!(inspect(&matching)["ok"], true, "{engine}");

            let version_mismatch = json!({"packages":[
                {"name": engine, "id": "e", "version": "1.0.0", "source": "registry+x"},
                {"name": companion, "id": "c", "version": "2.0.0", "source": "registry+x"}
            ]});
            assert_eq!(inspect(&version_mismatch)["ok"], false, "{engine}");

            let source_mismatch = json!({"packages":[
                {"name": engine, "id": "e", "version": "1.0.0", "source": "registry+x"},
                {"name": companion, "id": "c", "version": "1.0.0", "source": "git+y"}
            ]});
            assert_eq!(inspect(&source_mismatch)["ok"], false, "{engine}");
        }
    }

    #[test]
    fn resolves_the_gpui_backend_of_each_consumer_package() {
        let graph = json!({
            "packages": [
                {"name": "gpui", "id": "g", "version": "1.0.0", "source": "registry+x"},
                {"name": "gpui-kit", "id": "kit", "version": "1.0.0", "source": "registry+x"},
                {"name": "gpui-component", "id": "component", "version": "1.0.0", "source": "registry+x"},
                {"name": "gpui-base", "id": "base", "version": "1.0.0", "source": "registry+x"},
                {"name": "unrelated", "id": "u", "version": "1.0.0", "source": "registry+x"}
            ],
            "resolve": {
                "nodes": [
                    {"id": "g", "deps": []},
                    {"id": "kit", "deps": [{"pkg": "g"}]},
                    {"id": "component", "deps": [{"pkg": "g"}]},
                    {"id": "base", "deps": [{"pkg": "g"}]}
                ]
            }
        });
        let report = inspect(&graph);
        assert_eq!(report["ok"], true);
        let consumers = report["consumers"].as_array().unwrap();
        assert_eq!(consumers.len(), 3);
        assert!(consumers.iter().all(|c| c["gpui"] == json!(["g"])));
    }

    #[test]
    fn consumers_with_ambiguous_or_missing_backends_are_reported() {
        let two = json!({"packages":[
            {"name":"gpui","id":"a","version":"1.0.0","source":"registry+x"},
            {"name":"gpui-pre","id":"b","version":"1.0.0","source":"registry+x"},
            {"name":"gpui-kit","id":"kit","version":"1.0.0","source":"registry+x"}
        ],"resolve":{"nodes":[
            {"id":"a","deps":[]},
            {"id":"b","deps":[]},
            {"id":"kit","deps":[{"pkg":"a"},{"pkg":"b"}]}
        ]}});
        let report = inspect(&two);
        assert_eq!(report["ok"], false);
        assert!(
            report["errors"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e.as_str().unwrap().contains("resolves 2 GPUI backends"))
        );

        // A consumer without a resolve node is skipped, not treated as broken.
        let missing = json!({"packages":[
            {"name":"gpui","id":"a","version":"1.0.0","source":"registry+x"},
            {"name":"gpui-kit","id":"kit","version":"1.0.0","source":"registry+x"}
        ],"resolve":{"nodes":[]}});
        let report = inspect(&missing);
        assert_eq!(report["ok"], true);
        assert!(report["consumers"].as_array().unwrap().is_empty());
    }

    #[test]
    fn malformed_metadata_is_treated_as_empty() {
        let report = inspect(&json!({"packages": "not an array"}));
        assert_eq!(report["ok"], true);
        assert_eq!(report["engines"].as_array().unwrap().len(), 0);
    }
}
