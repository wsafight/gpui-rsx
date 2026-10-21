//! API snapshot: the resolved GPUI sources must match `tests/fixtures/gpui_api/*.txt`.
//!
//! Covers `Styled` (including macro-generated families), `InteractiveElement`,
//! `StatefulInteractiveElement`, `ParentElement` and `FluentBuilder`.
//!
//! Locally the test runs whenever GPUI sources are cached (`GPUI_SOURCE_ROOT`, or the
//! demo's locked registry packages) and is skipped otherwise. CI's `gpui-api` job sets
//! `GPUI_API_SNAPSHOT_REQUIRED=1`, which turns missing sources into a failure.

use std::path::Path;
use xtask::gpui_api::{GpuiApi, TRAITS, locate_sources_with};

fn strip_comments(text: &str) -> String {
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| format!("{l}\n"))
        .collect()
}

#[test]
fn resolved_gpui_api_matches_snapshot() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let required = std::env::var_os("GPUI_API_SNAPSHOT_REQUIRED").is_some_and(|v| v != "0");

    let (gpui_root, macros_root) = match locate_sources_with(None, workspace, false) {
        Ok(roots) => roots,
        Err(error) if !required => {
            eprintln!("skipping GPUI API snapshot: {error}");
            return;
        }
        Err(error) => {
            panic!("GPUI_API_SNAPSHOT_REQUIRED is set but GPUI sources are unavailable: {error}")
        }
    };

    let api = GpuiApi::load(&gpui_root, &macros_root).expect("GPUI API extraction failed");
    let mut drift = Vec::new();
    let contract =
        std::fs::read_to_string(workspace.join("tests/fixtures/gpui_api/TypeContracts.txt"))
            .expect("type contract fixture missing");
    if strip_comments(&contract) != strip_comments(&api.signature_contract) {
        drift.push("Explicit method signatures or generated style templates changed".into());
    }
    for (trait_name, _) in TRAITS {
        let fixture_path = workspace.join(format!("tests/fixtures/gpui_api/{trait_name}.txt"));
        let fixture = std::fs::read_to_string(&fixture_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", fixture_path.display()));
        let expected = strip_comments(&fixture);
        let actual = api.fixture(trait_name);
        if expected != actual {
            let expected: Vec<_> = expected.lines().collect();
            let actual: Vec<_> = actual.lines().collect();
            let added: Vec<_> = actual.iter().filter(|l| !expected.contains(l)).collect();
            let removed: Vec<_> = expected.iter().filter(|l| !actual.contains(l)).collect();
            drift.push(format!("{trait_name}: +{added:?} -{removed:?}"));
        }
    }

    assert!(
        drift.is_empty(),
        "GPUI API drifted from the snapshot; run `cargo xtask gen-tables`:\n{}",
        drift.join("\n")
    );
}
