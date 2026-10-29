//! The released Kit and its matching GPUI snapshot form one compatibility target.
use serde_json::Value;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub struct Revisions {
    pub gpui: String,
    pub component: String,
    pub kit_version: String,
}

pub fn locked_revisions(workspace: &Path) -> Result<Revisions, String> {
    let lock = std::fs::read_to_string(workspace.join("demo/Cargo.lock"))
        .map_err(|e| format!("failed to read demo/Cargo.lock: {e}"))?;
    revisions_from_lock(&lock)
}

fn revisions_from_lock(lock: &str) -> Result<Revisions, String> {
    let version = |name: &str| -> Result<String, String> {
        let packages: Vec<_> = lock
            .split("[[package]]")
            .skip(1)
            .filter(|block| field(block, "name") == Some(name))
            .collect();
        if packages.len() != 1 {
            return Err(format!(
                "expected one `{name}` in demo/Cargo.lock, found {}",
                packages.len()
            ));
        }
        let block = packages[0];
        if !field(block, "source").is_some_and(|s| s.starts_with("registry+")) {
            return Err(format!("`{name}` must use a published registry release"));
        }
        field(block, "version")
            .map(str::to_owned)
            .ok_or_else(|| format!("missing version for `{name}`"))
    };
    Ok(Revisions {
        gpui: format!("gpui-pre@{}", version("gpui-pre")?),
        component: format!("gpui-component@{}", version("gpui-component")?),
        kit_version: version("gpui-kit")?,
    })
}

fn field<'a>(block: &'a str, key: &str) -> Option<&'a str> {
    block.lines().find_map(|line| {
        let (name, value) = line.split_once('=')?;
        (name.trim() == key)
            .then(|| value.trim().strip_prefix('"')?.strip_suffix('"'))
            .flatten()
    })
}

/// Ensure that Kit, base and components export the same default GPUI types.
pub fn validate_graph(metadata: &Value) -> Result<Value, String> {
    let packages = metadata["packages"].as_array().ok_or("missing packages")?;
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or("missing resolved dependency graph")?;
    let one = |names: &[&str]| -> Result<&Value, String> {
        let matches: Vec<_> = packages
            .iter()
            .filter(|p| names.iter().any(|name| p["name"] == *name))
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "expected one package among {names:?}, found {}",
                matches.len()
            ));
        }
        Ok(matches[0])
    };
    let kit = one(&["gpui-kit"])?;
    let component = one(&["gpui-component"])?;
    let base = one(&["gpui-base"])?;
    let has_feature = |package: &Value, feature: &str| {
        nodes
            .iter()
            .find(|node| node["id"] == package["id"])
            .and_then(|node| node["features"].as_array())
            .is_some_and(|features| features.iter().any(|value| value == feature))
    };
    if [kit, component, base]
        .iter()
        .any(|package| has_feature(package, "gpui-fast"))
    {
        return Err("the demo must use Kit's default gpui-pre backend".into());
    }
    // Cargo metadata includes weak optional gpui-fast?/test-support edges even
    // when the Fast backend is disabled (cargo tree/build do not activate them).
    let names: &[&str] = if has_feature(kit, "test-support") {
        &["gpui-pre", "gpui"]
    } else {
        &["gpui-pre", "gpui", "gpui-fast"]
    };
    let engine = one(names)?;
    if engine["name"] != "gpui-pre" {
        return Err("the demo must use Kit's default gpui-pre backend".into());
    }
    let version = engine["version"].as_str().ok_or("missing GPUI version")?;
    let source = engine["source"].as_str().ok_or("missing GPUI source")?;
    if !source.starts_with("registry+") {
        return Err("gpui-pre must use a published registry release".into());
    }
    for names in [
        &["gpui-pre-platform", "gpui_platform"][..],
        &["gpui-pre-macros", "gpui_macros"][..],
    ] {
        let package = one(names)?;
        if package["version"] != version || package["source"] != source {
            return Err(format!("{} must match gpui-pre@{version}", package["name"]));
        }
    }
    for package in [kit, component, base] {
        let node = nodes
            .iter()
            .find(|node| node["id"] == package["id"])
            .ok_or("missing package in resolved graph")?;
        let dependency = node["deps"]
            .as_array()
            .and_then(|deps| deps.iter().find(|dep| dep["name"] == "gpui"))
            .ok_or_else(|| format!("{} has no resolved gpui dependency", package["name"]))?;
        if dependency["pkg"] != engine["id"] {
            return Err(format!(
                "{} resolves a different GPUI instance",
                package["name"]
            ));
        }
    }
    Ok(serde_json::json!({
        "gpui": format!("gpui-pre@{version}"),
        "component": format!("gpui-component@{}", component["version"].as_str().unwrap()),
        "kit": format!("gpui-kit@{}", kit["version"].as_str().unwrap()),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn metadata() -> Value {
        let packages = [
            ("gpui-pre", "0.3.9"),
            ("gpui-pre-platform", "0.3.9"),
            ("gpui-pre-macros", "0.3.9"),
            ("gpui-kit", "0.8.0"),
            ("gpui-component", "0.8.0"),
            ("gpui-base", "0.8.0"),
        ];
        let nodes: Vec<_> = ["gpui-kit", "gpui-component", "gpui-base"]
            .iter()
            .map(|id| {
                json!({
                    "id": id, "deps": [{"name": "gpui", "pkg": "gpui-pre"}],
                })
            })
            .collect();
        json!({
            "packages": packages.iter().map(|(name, version)| json!({
                "name": name, "id": name, "version": version,
                "source": "registry+https://github.com/rust-lang/crates.io-index",
            })).collect::<Vec<_>>(),
            "resolve": {"nodes": nodes},
        })
    }

    #[test]
    fn accepts_one_matching_snapshot_and_rejects_duplicate_engines() {
        let mut data = metadata();
        assert_eq!(validate_graph(&data).unwrap()["gpui"], "gpui-pre@0.3.9");
        data["packages"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "gpui"}));
        assert!(validate_graph(&data).unwrap_err().contains("found 2"));
    }

    #[test]
    fn allows_test_support_metadata_without_enabling_the_fast_backend() {
        let mut data = metadata();
        data["packages"].as_array_mut().unwrap().push(json!({
            "name": "gpui-fast", "id": "gpui-fast", "version": "0.1.12",
        }));
        assert!(validate_graph(&data).unwrap_err().contains("found 2"));
        data["resolve"]["nodes"][0]["features"] = json!(["test-support"]);
        assert_eq!(validate_graph(&data).unwrap()["gpui"], "gpui-pre@0.3.9");
        for index in 0..3 {
            let mut fast = data.clone();
            fast["resolve"]["nodes"][index]["features"] = json!(["test-support", "gpui-fast"]);
            assert!(
                validate_graph(&fast)
                    .unwrap_err()
                    .contains("default gpui-pre")
            );
        }
        data["resolve"]["nodes"][1]["deps"][0]["pkg"] = json!("gpui-fast");
        assert!(
            validate_graph(&data)
                .unwrap_err()
                .contains("different GPUI")
        );
    }

    #[test]
    fn rejects_mismatched_snapshot_families_and_component_types() {
        let mut data = metadata();
        data["packages"][1]["version"] = json!("0.3.8");
        assert!(validate_graph(&data).unwrap_err().contains("must match"));
        let mut data = metadata();
        data["resolve"]["nodes"][1]["deps"][0]["pkg"] = json!("another-gpui");
        assert!(
            validate_graph(&data)
                .unwrap_err()
                .contains("different GPUI")
        );
    }

    #[test]
    fn reads_registry_versions_without_borrowing_fields_from_the_next_package() {
        let mut lock = String::new();
        for (name, version) in [
            ("gpui-pre", "0.3.9"),
            ("gpui-component", "0.8.0"),
            ("gpui-kit", "0.8.0"),
        ] {
            lock.push_str(&format!("[[package]]\nname = \"{name}\"\nversion = \"{version}\"\nsource = \"registry+index\"\n"));
        }
        assert_eq!(revisions_from_lock(&lock).unwrap().gpui, "gpui-pre@0.3.9");
        let broken = lock.replacen("source = \"registry+index\"\n", "", 1);
        assert!(revisions_from_lock(&broken).is_err());
        lock.push_str("[[package]]\nname = \"gpui-pre\"\nversion = \"0.3.8\"\n");
        assert!(revisions_from_lock(&lock).unwrap_err().contains("found 2"));
    }
}
