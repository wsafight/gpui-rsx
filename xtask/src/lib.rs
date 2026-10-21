//! Library half of `xtask`, shared by the binary and the API snapshot tests.

pub mod bench;
pub mod compat;
pub mod gen_tables;
pub mod gpui_api;
pub mod spec;
pub mod upstream;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Writes generated files (rustfmt-formatting `.rs` outputs), or with `check` reports stale ones.
pub fn write_outputs(files: &[(PathBuf, String)], check: bool) -> Result<(), String> {
    let mut stale = Vec::new();
    for (path, content) in files {
        let content = if path.extension().is_some_and(|e| e == "rs") {
            rustfmt(content)?
        } else {
            content.clone()
        };
        let current = std::fs::read_to_string(path).unwrap_or_default();
        if current == content {
            continue;
        }
        if check {
            stale.push(path.display().to_string());
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
            }
            std::fs::write(path, content)
                .map_err(|e| format!("failed to write {}: {e}", path.display()))?;
            eprintln!("wrote {}", path.display());
        }
    }
    if stale.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "generated files are stale (run the xtask without --check):\n  {}",
            stale.join("\n  ")
        ))
    }
}

pub fn rustfmt(source: &str) -> Result<String, String> {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2024", "--emit", "stdout", "--quiet"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run rustfmt: {e}"))?;
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(source.as_bytes())
        .map_err(|e| format!("failed to feed rustfmt: {e}"))?;
    let output = child
        .wait_with_output()
        .map_err(|e| format!("rustfmt failed: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "rustfmt rejected generated code:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| e.to_string())
}
