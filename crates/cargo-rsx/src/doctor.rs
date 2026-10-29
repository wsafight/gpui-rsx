use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

const ENGINES: &[&str] = &["gpui", "gpui-pre", "gpui-fast"];

// Kit's backend feature controls which dependency it re-exports. Cargo metadata also
// lists weak optional test-support edges to the unselected backend and its platform.
fn selected_dependency(package: &Value, node: &Value, target: &Value) -> bool {
    if !matches!(
        package["name"].as_str(),
        Some("gpui-kit" | "gpui-base" | "gpui-component" | "gpui-kit-assets")
    ) {
        return true;
    }
    let fast = node["features"]
        .as_array()
        .is_some_and(|features| features.iter().any(|f| f == "gpui-fast"));
    match target["name"].as_str() {
        Some("gpui-fast" | "gpui-fast-platform" | "gpui-fast-web") => fast,
        Some(
            "gpui" | "gpui-pre" | "gpui_platform" | "gpui-pre-platform" | "gpui_web"
            | "gpui-pre-web",
        ) => !fast,
        _ => true,
    }
}

fn active_packages(
    metadata: &Value,
    packages: &[Value],
    nodes: &[Value],
) -> Option<HashSet<String>> {
    let roots = metadata["workspace_members"].as_array()?;
    let by_id: HashMap<_, _> = packages
        .iter()
        .filter_map(|p| Some((p["id"].as_str()?, p)))
        .collect();
    let by_node: HashMap<_, _> = nodes
        .iter()
        .filter_map(|n| Some((n["id"].as_str()?, n)))
        .collect();
    let mut pending: Vec<_> = roots.iter().filter_map(Value::as_str).collect();
    let mut active = HashSet::new();
    while let Some(id) = pending.pop() {
        if !active.insert(id.to_owned()) {
            continue;
        }
        let (Some(package), Some(node)) = (by_id.get(id), by_node.get(id)) else {
            continue;
        };
        for dep in node["deps"].as_array().into_iter().flatten() {
            if let Some(id) = dep["pkg"].as_str()
                && let Some(target) = by_id.get(id)
                && selected_dependency(package, node, target)
            {
                pending.push(id);
            }
        }
    }
    Some(active)
}

/// Analyze resolved identities, including dependency aliases. Valid for Git and registry GPUI.
pub fn inspect(metadata: &Value) -> Value {
    let packages = metadata["packages"].as_array().cloned().unwrap_or_default();
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let active = active_packages(metadata, &packages, &nodes);
    let is_active = |p: &&Value| {
        active
            .as_ref()
            .is_none_or(|active| p["id"].as_str().is_some_and(|id| active.contains(id)))
    };
    let engines: Vec<_> = packages
        .iter()
        .filter(is_active)
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
    for package in packages.iter().filter(is_active) {
        let name = package["name"].as_str().unwrap_or("");
        if !matches!(name, "gpui-kit" | "gpui-component" | "gpui-base") {
            continue;
        }
        if let Some(node) = nodes.iter().find(|n| n["id"] == package["id"]) {
            let deps = node["deps"].as_array().cloned().unwrap_or_default();
            let resolved: Vec<_> = deps
                .iter()
                .filter(|dep| engines.iter().any(|engine| engine["id"] == dep["pkg"]))
                .filter(|dep| {
                    packages
                        .iter()
                        .find(|p| p["id"] == dep["pkg"])
                        .is_some_and(|target| selected_dependency(package, node, target))
                })
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
            .filter(is_active)
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

    fn kit_metadata(fast: bool) -> Value {
        let features = if fast {
            json!(["test-support", "gpui-fast"])
        } else {
            json!(["test-support"])
        };
        let packages = [
            ("app", "app", "0.1.0"),
            ("kit", "gpui-kit", "0.8.1"),
            ("base", "gpui-base", "0.8.1"),
            ("component", "gpui-component", "0.8.1"),
            ("pre", "gpui-pre", "0.3.10"),
            ("fast", "gpui-fast", "0.1.12"),
            ("platform", "gpui-fast-platform", "0.1.12"),
        ];
        json!({
            "workspace_members": ["app"],
            "packages": packages.map(|(id, name, version)| json!({"id": id, "name": name, "version": version, "source": "registry+x"})),
            "resolve": {"nodes": [
                {"id": "app", "deps": [{"pkg": "kit"}]},
                {"id": "kit", "features": features, "deps": [{"pkg": "pre"}, {"pkg": "fast"}, {"pkg": "platform"}, {"pkg": "base"}, {"pkg": "component"}]},
                {"id": "base", "features": features, "deps": [{"pkg": "pre"}, {"pkg": "fast"}]},
                {"id": "component", "features": features, "deps": [{"pkg": "pre"}, {"pkg": "fast"}]},
                {"id": "pre", "deps": []},
                {"id": "fast", "deps": []},
                {"id": "platform", "deps": [{"pkg": "fast"}]},
            ]},
        })
    }

    #[test]
    fn ignores_unselected_weak_test_support_dependencies() {
        let mut graph = kit_metadata(false);
        // Even an unrelated inactive companion version must not affect the selected engine.
        graph["packages"][6]["version"] = json!("9.9.9");
        let report = inspect(&graph);
        assert_eq!(report["ok"], true, "{report}");
        assert_eq!(report["engines"].as_array().unwrap().len(), 1);
        assert_eq!(report["engines"][0]["name"], "gpui-pre");
        assert!(
            report["consumers"]
                .as_array()
                .unwrap()
                .iter()
                .all(|c| c["gpui"] == json!(["pre"]))
        );
    }

    #[test]
    fn accepts_the_selected_fast_backend_with_default_metadata_present() {
        let report = inspect(&kit_metadata(true));
        assert_eq!(report["ok"], true, "{report}");
        assert_eq!(report["engines"].as_array().unwrap().len(), 1);
        assert_eq!(report["engines"][0]["name"], "gpui-fast");
        assert!(
            report["consumers"]
                .as_array()
                .unwrap()
                .iter()
                .all(|c| c["gpui"] == json!(["fast"]))
        );
    }

    #[test]
    fn still_reports_a_direct_second_engine_with_default_kit() {
        let mut graph = kit_metadata(false);
        graph["resolve"]["nodes"][0]["deps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"pkg": "fast"}));
        let report = inspect(&graph);
        assert_eq!(report["ok"], false);
        assert_eq!(report["engines"].as_array().unwrap().len(), 2);
        // Kit itself continues to select only its default backend.
        assert!(
            report["consumers"]
                .as_array()
                .unwrap()
                .iter()
                .all(|c| c["gpui"] == json!(["pre"]))
        );
    }

    #[test]
    fn rejects_different_backend_features_between_consumers() {
        let mut graph = kit_metadata(true);
        graph["resolve"]["nodes"][3]["features"] = json!(["test-support"]);
        let report = inspect(&graph);
        assert_eq!(report["ok"], false);
        assert_eq!(report["engines"].as_array().unwrap().len(), 2);
    }

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
