//! Resolves GPUI and macro sources from overrides or the locked demo graph.

use std::path::{Path, PathBuf};

/// Locates the GPUI and gpui_macros source roots.
///
/// Priority: explicit path → `GPUI_SOURCE_ROOT` → `cargo metadata` of the demo manifest.
pub fn locate_sources(
    explicit: Option<PathBuf>,
    workspace: &Path,
) -> Result<(PathBuf, PathBuf), String> {
    locate_sources_with(explicit, workspace, true)
}

/// Like [`locate_sources`]; with `allow_network = false` only cached sources are used.
pub fn locate_sources_with(
    explicit: Option<PathBuf>,
    workspace: &Path,
    allow_network: bool,
) -> Result<(PathBuf, PathBuf), String> {
    let override_root =
        explicit.or_else(|| std::env::var_os("GPUI_SOURCE_ROOT").map(PathBuf::from));
    let metadata = if override_root.is_none() {
        Some(demo_metadata(workspace, allow_network)?)
    } else {
        None
    };
    let gpui_root = match override_root {
        Some(root) => root,
        None => package_root(metadata.as_ref().unwrap(), "gpui")?,
    };
    // Git checkouts have sibling crates. Published snapshots are independent
    // registry directories, so resolve their macros from the same locked graph.
    let macros_root = std::env::var_os("GPUI_MACROS_SOURCE_ROOT")
        .map(PathBuf::from)
        .or_else(|| {
            gpui_root
                .parent()
                .map(|crates| crates.join("gpui_macros"))
                .filter(|p| p.join("src/styles.rs").exists())
        });
    let macros_root = match macros_root {
        Some(root) => root,
        None => {
            let metadata = match metadata {
                Some(data) => data,
                None => demo_metadata(workspace, allow_network)?,
            };
            package_root(&metadata, "gpui_macros")?
        }
    };
    if !macros_root.join("src/styles.rs").is_file() {
        return Err(format!(
            "GPUI style macros not found in {}; set GPUI_MACROS_SOURCE_ROOT",
            macros_root.display()
        ));
    }
    Ok((gpui_root, macros_root))
}

pub fn metadata_package_root(
    workspace: &Path,
    package: &str,
    allow_network: bool,
) -> Result<PathBuf, String> {
    package_root(&demo_metadata(workspace, allow_network)?, package)
}

pub(super) fn package_root(metadata: &serde_json::Value, package: &str) -> Result<PathBuf, String> {
    let names: &[&str] = match package {
        "gpui" => &["gpui-pre", "gpui"],
        "gpui_macros" => &["gpui-pre-macros", "gpui_macros"],
        _ => &[package],
    };
    let packages: Vec<_> = metadata["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| names.iter().any(|name| p["name"] == *name))
        .collect();
    if packages.len() != 1 {
        return Err(format!(
            "expected one `{package}` package in demo metadata, found {}",
            packages.len()
        ));
    }
    let manifest = packages[0]["manifest_path"]
        .as_str()
        .ok_or("missing package manifest_path")?;
    Path::new(manifest)
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| format!("invalid manifest path: {manifest}"))
}

pub fn demo_metadata(workspace: &Path, allow_network: bool) -> Result<serde_json::Value, String> {
    let manifest = workspace.join("demo/Cargo.toml");
    let mut last_error = String::new();
    let modes: &[bool] = if allow_network {
        &[true, false]
    } else {
        &[true]
    };
    for &offline in modes {
        let mut cmd =
            std::process::Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
        cmd.args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--manifest-path",
        ])
        .arg(&manifest);
        if offline {
            cmd.arg("--offline");
        }
        let output = cmd
            .output()
            .map_err(|e| format!("failed to run cargo metadata: {e}"))?;
        if !output.status.success() {
            last_error = String::from_utf8_lossy(&output.stderr).into_owned();
            continue;
        }
        return serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("invalid cargo metadata output: {e}"));
    }
    Err(format!(
        "could not resolve the demo dependency graph via cargo metadata: {last_error}"
    ))
}
