//! Resolves the class mode a file compiles with, in the same priority order as the macros:
//! an explicit `--tailwind`, then `rsx.toml`, then the `tailwind-scale` feature of `gpui-rsx`.
use gpui_rsx_core::{ClassMode, SpacingScale, Theme, resolve_spacing_scale};
use serde_json::Value;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};

/// Command-line overrides; the language server uses the defaults.
#[derive(Clone, Copy, Default)]
pub(crate) struct ModeOptions<'a> {
    pub strict: bool,
    pub tailwind: bool,
    pub manifest: Option<&'a Path>,
    pub offline: bool,
}

pub(crate) fn mode(options: ModeOptions, path: Option<&Path>) -> Result<ClassMode, String> {
    let package = package_dir(options.manifest, path)?;
    let theme = match &package {
        Some(dir) => Theme::load_from_dir(dir)?,
        None => None,
    };
    let forced = options.tailwind.then_some(SpacingScale::Tailwind);
    let feature_default = if forced.is_some() || theme.as_deref().and_then(Theme::scale).is_some() {
        // Explicit choices do not need dependency resolution.
        SpacingScale::Px
    } else {
        package.as_deref().map_or(Ok(SpacingScale::Px), |dir| {
            feature_scale(dir, options.offline)
        })?
    };
    Ok(ClassMode::builder()
        .strict(options.strict)
        .scale(resolve_spacing_scale(
            forced,
            theme.as_deref(),
            feature_default,
        ))
        .theme(theme)
        .build())
}

pub(crate) fn resolve_theme(
    manifest: Option<&Path>,
    path: Option<&Path>,
) -> Result<Option<Arc<Theme>>, String> {
    match package_dir(manifest, path)? {
        Some(dir) => Theme::load_from_dir(&dir),
        None => Ok(None),
    }
}

/// The nearest directory with a `Cargo.toml` above `path`, or above the manifest or current
/// directory when no path is given.
fn package_dir(manifest: Option<&Path>, path: Option<&Path>) -> Result<Option<PathBuf>, String> {
    let fallback = manifest
        .map(|manifest| Ok(manifest.to_owned()))
        .unwrap_or_else(std::env::current_dir)
        .map_err(|e| format!("cannot resolve current directory: {e}"))?;
    let anchor = path.unwrap_or(&fallback);
    let anchor = std::fs::canonicalize(anchor).unwrap_or_else(|_| anchor.to_owned());
    let start = if anchor.is_dir() {
        Some(anchor.as_path())
    } else {
        anchor.parent()
    };
    if let Some(dir) = start
        .into_iter()
        .flat_map(Path::ancestors)
        .find(|dir| dir.join("Cargo.toml").exists())
    {
        return Ok(Some(dir.to_owned()));
    }
    Ok(manifest.map(|manifest| manifest.parent().unwrap_or(Path::new(".")).to_owned()))
}

static FEATURE_CACHE: Mutex<Option<HashMap<PathBuf, SpacingScale>>> = Mutex::new(None);

/// Forgets resolved features, for example after a manifest changed.
pub(crate) fn clear_feature_cache() {
    *FEATURE_CACHE.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

/// The scale implied by `gpui-rsx/tailwind-scale` for the package in `dir`.
///
/// A sibling member or transitive dependency can enable the feature without mentioning it
/// in this package's ancestor manifests. Resolve the actual Cargo graph once per workspace
/// root and share the result between members.
fn feature_scale(dir: &Path, offline: bool) -> Result<SpacingScale, String> {
    let root = workspace_root(dir);
    let mut cache = FEATURE_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let cache = cache.get_or_insert_with(HashMap::new);
    if let Some(scale) = cache.get(&root) {
        return Ok(*scale);
    }
    let scale = if metadata_enables_tailwind(&root, offline)? {
        SpacingScale::Tailwind
    } else {
        SpacingScale::Px
    };
    // Only cache successful resolutions; a missing dependency may become available later.
    cache.insert(root, scale);
    Ok(scale)
}

/// The nearest ancestor whose manifest declares `[workspace]`, or `dir` itself.
fn workspace_root(dir: &Path) -> PathBuf {
    dir.ancestors()
        .find(|ancestor| {
            std::fs::read_to_string(ancestor.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.lines().any(|line| line.trim() == "[workspace]"))
        })
        .unwrap_or(dir)
        .to_owned()
}

fn metadata_enables_tailwind(dir: &Path, offline: bool) -> Result<bool, String> {
    let mut command = Command::new("cargo");
    command
        .args(["metadata", "--format-version", "1", "--manifest-path"])
        .arg(dir.join("Cargo.toml"));
    if offline {
        command.arg("--offline");
    }
    let context = || format!("cannot determine spacing scale for {}", dir.display());
    let output = command
        .output()
        .map_err(|error| format!("{}: cannot run cargo metadata: {error}", context()))?;
    if !output.status.success() {
        return Err(format!(
            "{}: cargo metadata failed:\n{}",
            context(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("{}: invalid cargo metadata: {error}", context()))?;
    Ok(resolves_tailwind_feature(&metadata))
}

fn resolves_tailwind_feature(metadata: &Value) -> bool {
    let ids: Vec<&Value> = metadata["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|package| package["name"] == "gpui-rsx")
        .map(|package| &package["id"])
        .collect();
    metadata["resolve"]["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|node| ids.contains(&&node["id"]))
        .any(|node| {
            node["features"]
                .as_array()
                .is_some_and(|features| features.iter().any(|f| f == "tailwind-scale"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn workspace_members_share_one_root() {
        let root = std::env::temp_dir().join(format!("rsx-ws-root-{}", std::process::id()));
        std::fs::create_dir_all(root.join("crates/a")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/a\"]\n",
        )
        .unwrap();
        std::fs::write(
            root.join("crates/a/Cargo.toml"),
            "[package]\nname = \"a\"\n",
        )
        .unwrap();
        assert_eq!(workspace_root(&root.join("crates/a")), root);
        assert_eq!(workspace_root(&root), root);
        std::fs::remove_dir_all(&root).unwrap();
        // Without a workspace manifest the package is its own root.
        let lone = std::env::temp_dir().join(format!("rsx-ws-lone-{}", std::process::id()));
        std::fs::create_dir_all(&lone).unwrap();
        assert_eq!(workspace_root(&lone), lone);
        std::fs::remove_dir_all(&lone).unwrap();
    }

    #[test]
    fn tailwind_feature_is_read_from_the_resolved_gpui_rsx_node() {
        let metadata = |features: Value| {
            json!({
                "packages": [{"name": "gpui-rsx", "id": "rsx"}, {"name": "app", "id": "app"}],
                "resolve": {"nodes": [
                    {"id": "rsx", "features": features},
                    {"id": "app", "features": ["tailwind-scale"]},
                ]},
            })
        };
        assert!(resolves_tailwind_feature(&metadata(json!([
            "tailwind-scale"
        ]))));
        // Another package's feature of the same name does not count.
        assert!(!resolves_tailwind_feature(&metadata(json!([]))));
        assert!(!resolves_tailwind_feature(&json!({})));
    }

    #[test]
    fn failed_feature_resolution_is_retried_after_dependencies_become_available() {
        let root = std::env::temp_dir().join(format!("rsx-feature-retry-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), "").unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"retry\"\nversion = \"0.1.0\"\n\
             [dependencies]\ngpui-rsx = { path = \"runtime\", features = [\"tailwind-scale\"] }\n",
        )
        .unwrap();
        assert!(feature_scale(&root, true).is_err());
        std::fs::create_dir_all(root.join("runtime/src")).unwrap();
        std::fs::write(root.join("runtime/src/lib.rs"), "").unwrap();
        std::fs::write(
            root.join("runtime/Cargo.toml"),
            "[package]\nname = \"gpui-rsx\"\nversion = \"0.1.0\"\n\
             [features]\ntailwind-scale = []\n",
        )
        .unwrap();
        assert_eq!(feature_scale(&root, true).unwrap(), SpacingScale::Tailwind);
        std::fs::remove_dir_all(root).unwrap();
    }
}
