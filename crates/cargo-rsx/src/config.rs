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
    let feature_default = package
        .as_deref()
        .map_or(SpacingScale::Px, |dir| feature_scale(dir, options.offline));
    let forced = options.tailwind.then_some(SpacingScale::Tailwind);
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
/// Manifests that never name the feature skip `cargo metadata`, which keeps the common case
/// free. Features unify across a workspace, so `cargo metadata` runs once per workspace root
/// and every member shares the result.
fn feature_scale(dir: &Path, offline: bool) -> SpacingScale {
    let mentioned = dir.ancestors().any(|ancestor| {
        std::fs::read_to_string(ancestor.join("Cargo.toml"))
            .is_ok_and(|manifest| manifest.contains("tailwind-scale"))
    });
    if !mentioned {
        return SpacingScale::Px;
    }
    let root = workspace_root(dir);
    let mut cache = FEATURE_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let cache = cache.get_or_insert_with(HashMap::new);
    *cache.entry(root.clone()).or_insert_with(|| {
        if metadata_enables_tailwind(&root, offline).unwrap_or(false) {
            SpacingScale::Tailwind
        } else {
            SpacingScale::Px
        }
    })
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

fn metadata_enables_tailwind(dir: &Path, offline: bool) -> Option<bool> {
    let mut command = Command::new("cargo");
    command
        .args(["metadata", "--format-version", "1", "--manifest-path"])
        .arg(dir.join("Cargo.toml"));
    if offline {
        command.arg("--offline");
    }
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let metadata: Value = serde_json::from_slice(&output.stdout).ok()?;
    Some(resolves_tailwind_feature(&metadata))
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
}
