use serde_json::{Value, json};
use std::{path::Path, process::Command};

use crate::{Options, hot_reload, metadata, sources};

fn detect_hotreload_feature(meta: &Value) -> Option<String> {
    let members = meta["workspace_members"].as_array()?;
    for pkg in meta["packages"].as_array()? {
        if !members.iter().any(|id| id == &pkg["id"]) {
            continue;
        }
        let Some(features) = pkg["features"].as_object() else {
            continue;
        };
        for candidate in ["hotreload", "hot-reload"] {
            if features.contains_key(candidate) {
                return Some(candidate.to_owned());
            }
        }
    }
    None
}

pub(super) fn run(opt: &Options) -> Result<i32, String> {
    let mut resolved_opt = Options {
        manifest: opt.manifest.clone(),
        path: opt.path.clone(),
        offline: opt.offline,
        features: opt.features.clone(),
        ..Options::default()
    };
    if resolved_opt.features.is_none() {
        let base_meta = metadata(&resolved_opt, true)?;
        resolved_opt.features = detect_hotreload_feature(&base_meta);
    }

    let full_meta = metadata(&resolved_opt, false)?;
    let dx_binary = std::env::var_os("RSX_DX").unwrap_or_else(|| "dx".into());
    let dx = Command::new(&dx_binary)
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned());
    let hot_patch = Command::new(&dx_binary)
        .args(["serve", "--help"])
        .output()
        .ok()
        .is_some_and(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains("--hot-patch")
        });

    let scan_opt = Options {
        manifest: opt.manifest.clone(),
        offline: opt.offline,
        ..Options::default()
    };
    let scanned_sources = sources(&scan_opt)?;
    let readiness = hot_reload::inspect(
        &full_meta,
        &scanned_sources,
        dx.as_deref().map(|version| (version, hot_patch)),
    );

    let mut dx_args = vec![
        "serve".to_owned(),
        "--hot-patch".to_owned(),
        "--hot-reload".to_owned(),
        "true".to_owned(),
    ];
    if let Some(target) = opt.path.as_deref().and_then(Path::to_str) {
        dx_args.push("--bin".to_owned());
        dx_args.push(target.to_owned());
    }
    if let Some(features) = &resolved_opt.features {
        dx_args.push("--features".to_owned());
        dx_args.push(features.clone());
    }

    let payload = json!({
        "ok": readiness["ok"],
        "command": dx_binary.to_string_lossy(),
        "args": dx_args,
        "features": resolved_opt.features,
        "hotReload": readiness,
    });

    if opt.json {
        println!("{payload}");
    }

    if opt.check {
        if !opt.json {
            println!(
                "dev preflight: {} ({} {})",
                if payload["ok"] == true {
                    "ready"
                } else {
                    "failed"
                },
                payload["command"].as_str().unwrap_or("dx"),
                dx_args.join(" ")
            );
            if let Some(checks) = payload["hotReload"]["checks"].as_array() {
                for check in checks {
                    println!(
                        "  {} [{}]: {}",
                        check["code"].as_str().unwrap_or(""),
                        check["status"].as_str().unwrap_or(""),
                        check["message"].as_str().unwrap_or("")
                    );
                }
            }
        }
        return Ok(i32::from(payload["ok"] != true));
    }

    if payload["ok"] != true {
        if let Some(checks) = payload["hotReload"]["checks"].as_array() {
            for check in checks
                .iter()
                .filter(|c| c["status"] == "failed" || c["status"] == "warning")
            {
                eprintln!(
                    "{} [{}]: {}",
                    check["code"].as_str().unwrap_or(""),
                    check["status"].as_str().unwrap_or(""),
                    check["message"].as_str().unwrap_or("")
                );
            }
        }
        return Err("hot-reload preflight failed; run `cargo rsx dev --check` for details".into());
    }

    if let Some(collisions) = payload["hotReload"]["templateNameCollisions"].as_array()
        && !collisions.is_empty()
    {
        eprintln!(
            "warning: found {} `rsx!` macro invocation(s) that `dx` may intercept; alias `use gpui_rsx::rsx as view;`",
            collisions.len()
        );
    }

    let mut cmd = Command::new(&dx_binary);
    cmd.args(&dx_args);
    if let Some(manifest) = &opt.manifest
        && let Some(parent) = manifest.parent()
        && !parent.as_os_str().is_empty()
    {
        cmd.current_dir(parent);
    }
    let status = cmd
        .status()
        .map_err(|e| format!("failed to launch `{}`: {e}", dx_binary.to_string_lossy()))?;
    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_package_hotreload_feature_automatically() {
        let meta = json!({
            "workspace_members": ["app#0.1.0"],
            "packages": [
                {
                    "id": "app#0.1.0",
                    "name": "app",
                    "features": {
                        "default": [],
                        "hotreload": ["gpui/hot-reload"]
                    }
                }
            ]
        });
        assert_eq!(
            detect_hotreload_feature(&meta).as_deref(),
            Some("hotreload")
        );
    }
}
