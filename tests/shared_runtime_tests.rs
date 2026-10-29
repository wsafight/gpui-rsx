//! Exercise Cargo's actual target environment, including a renamed library and GPUI alias.
#[test]
fn shared_runtime_compiles_across_cargo_targets() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target"))
        .join("shared-runtime-fixtures");
    for fixture in ["shared-runtime", "binary-runtime"] {
        let output = std::process::Command::new(env!("CARGO"))
            .args(["test", "--offline", "--all-targets", "--manifest-path"])
            .arg(root.join(format!("tests/fixtures/{fixture}/Cargo.toml")))
            .env("CARGO_TARGET_DIR", &target)
            .output()
            .expect("run fixture cargo tests");
        assert!(
            output.status.success(),
            "{fixture}:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
