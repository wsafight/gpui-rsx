use std::io::Write;
use std::process::{Command, Stdio};
fn command(args: &[&str], source: Option<&str>) -> std::process::Output {
    let mut process = Command::new(env!("CARGO_BIN_EXE_cargo-rsx"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(source) = source {
        process
            .stdin
            .take()
            .unwrap()
            .write_all(source.as_bytes())
            .unwrap();
    }
    process.wait_with_output().unwrap()
}
#[test]
fn every_command_and_theme_operation_has_help() {
    for cmd in [
        "doctor", "lint", "fmt", "migrate", "convert", "expand", "preview", "theme", "lsp",
    ] {
        let output = command(&[cmd, "--help"], None);
        assert!(output.status.success(), "{cmd}");
        assert!(String::from_utf8_lossy(&output.stdout).contains(&format!("cargo rsx {cmd}")));
    }
    for op in ["check", "list", "explain"] {
        assert!(command(&["theme", op, "--help"], None).status.success());
    }
}
#[test]
fn bare_body_and_severity_filters_keep_exit_status() {
    let output = command(
        &["lint", "-", "--json", "--min-severity", "warning"],
        Some("<div class={classes}/>"),
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["diagnostics"].as_array().unwrap().is_empty());
    assert!(output.status.success());
    let output = command(
        &["lint", "-", "--quiet", "--deny-warnings", "--json"],
        Some("<div class=\"flex-colum\"/>"),
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["diagnostics"].as_array().unwrap().is_empty());
    assert_eq!(output.status.code(), Some(1));
    let output = command(&["lint", "-", "-q"], Some("<div>hello</div>"));
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Bare text"));
}
#[test]
fn tailwind_conversion_keeps_units_and_simple_expressions() {
    let output = command(
        &["convert", "-", "--tailwind"],
        Some("<div className=\"p-4\" title={user.name}>{title}</div>"),
    );
    assert!(output.status.success());
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let rsx = result["rsx"].as_str().unwrap();
    assert!(rsx.starts_with("rsx_tw!"));
    assert!(rsx.contains("p-4"));
    assert!(rsx.contains("user.name"));
    assert!(rsx.contains("{title}"));
}
