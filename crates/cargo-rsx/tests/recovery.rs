use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

struct Project(PathBuf);

impl Project {
    fn new() -> Self {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "rsx-recovery-{}-{unique}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname=\"recovery-fixture\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[workspace]\n").unwrap();
        Self(root)
    }

    fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_cargo-rsx"))
            .current_dir(&self.0)
            .args(arguments)
            .arg("--offline")
            .output()
            .unwrap()
    }

    fn lsp(&self, messages: Vec<Value>) -> Vec<Value> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_cargo-rsx"))
            .current_dir(&self.0)
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let input = messages
            .into_iter()
            .map(|message| {
                let body = message.to_string();
                format!("Content-Length: {}\r\n\r\n{body}", body.len())
            })
            .collect::<String>();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut raw = std::str::from_utf8(&output.stdout).unwrap();
        let mut decoded = Vec::new();
        while let Some((header, body)) = raw.split_once("\r\n\r\n") {
            let length: usize = header
                .strip_prefix("Content-Length: ")
                .unwrap()
                .parse()
                .unwrap();
            decoded.push(serde_json::from_str(&body[..length]).unwrap());
            raw = &body[length..];
        }
        assert!(raw.is_empty());
        decoded
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn position(source: &str, offset: usize) -> Value {
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let last = prefix.rsplit('\n').next().unwrap();
    json!({"line": line, "character": last.encode_utf16().count()})
}

#[test]
fn batch_format_checks_and_migrations_reject_recoverable_lexical_damage_atomically() {
    for operation in [
        ["fmt", "--write"],
        ["fmt", "--check"],
        ["migrate", "--write"],
    ] {
        for suffix in [
            "fn unfinished(",
            "let text = r#\"unfinished",
            "/* unfinished",
            ")",
        ] {
            let project = Project::new();
            let good = "use gpui_rsx::rsx as view; fn main(){view!{<div class=\"gap-8\"/>};}\n";
            let broken = format!("{good}// 中文😀\n{suffix}");
            fs::write(project.0.join("src/main.rs"), good).unwrap();
            fs::write(project.0.join("src/z_broken.rs"), &broken).unwrap();
            let output = project.run(&[operation[0], "src", operation[1]]);
            assert_eq!(
                output.status.code(),
                Some(2),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stderr).contains("z_broken.rs"));
            assert_eq!(
                fs::read_to_string(project.0.join("src/main.rs")).unwrap(),
                good
            );
            assert_eq!(
                fs::read_to_string(project.0.join("src/z_broken.rs")).unwrap(),
                broken
            );
        }
    }
}

#[test]
fn aliased_formatting_is_idempotent_and_preserves_surrounding_rust() {
    let project = Project::new();
    let prefix = "use gpui_rsx::rsx as view;\nconst LABEL: &str = \"中文😀\";\nfn main(){";
    let suffix = "; println!(\"unchanged\");}\n";
    let source = format!("{prefix}view!{{<div class=\"flex\"/>}}{suffix}");
    fs::write(project.0.join("src/main.rs"), &source).unwrap();
    assert_eq!(
        project.run(&["fmt", "src", "--check"]).status.code(),
        Some(1)
    );
    let output = project.run(&["fmt", "src", "--write"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let formatted = fs::read_to_string(project.0.join("src/main.rs")).unwrap();
    assert!(formatted.starts_with(prefix) && formatted.ends_with(suffix));
    assert!(formatted.contains("<div class=\"flex\" />"));
    assert!(project.run(&["fmt", "src", "--check"]).status.success());
    assert!(project.run(&["fmt", "src", "--write"]).status.success());
    assert_eq!(
        fs::read_to_string(project.0.join("src/main.rs")).unwrap(),
        formatted
    );
}

#[test]
fn incomplete_file_preview_formats_the_macro_without_writing_or_hiding_diagnostics() {
    let project = Project::new();
    let source = "use gpui_rsx::rsx_strict as view; fn main(){view!{<div class=\"flex-colum\"/>};}\nfn unfinished(";
    fs::write(project.0.join("src/main.rs"), source).unwrap();
    let formatted = project.run(&["fmt", "src/main.rs"]);
    assert!(formatted.status.success());
    let stdout = String::from_utf8(formatted.stdout).unwrap();
    assert!(stdout.contains("<div class=\"flex-colum\" />"));
    assert!(stdout.contains("fn unfinished("));
    assert_eq!(
        fs::read_to_string(project.0.join("src/main.rs")).unwrap(),
        source
    );
    let linted = project.run(&["lint", "src/main.rs", "--json"]);
    assert_eq!(linted.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&linted.stdout).unwrap();
    let diagnostics = report["diagnostics"].as_array().unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|entry| entry["diagnostic"]["code"] == "rsx/rust-syntax")
    );
    assert!(
        diagnostics
            .iter()
            .any(|entry| entry["diagnostic"]["suggestion"] == "flex-col")
    );
}

#[test]
fn migration_converts_pixel_aliases_without_converting_tailwind_aliases_again() {
    let project = Project::new();
    let source = "use gpui_rsx::{rsx as view, rsx_tw as tw}; fn main(){view!{<div class=\"gap-8\"/>};tw!{<div class=\"gap-8\"/>};}";
    fs::write(project.0.join("src/main.rs"), source).unwrap();
    let output = project.run(&["migrate", "src/main.rs", "--write"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let migrated = fs::read_to_string(project.0.join("src/main.rs")).unwrap();
    assert!(migrated.contains("view!{<div class=\"gap-2\"/>}"));
    assert!(migrated.contains("tw!{<div class=\"gap-8\"/>}"));
    fs::write(
        project.0.join("rsx.toml"),
        "[options]\nspacing-scale=\"tailwind\"\n",
    )
    .unwrap();
    assert_eq!(
        project
            .run(&["migrate", "src/main.rs", "--write"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        fs::read_to_string(project.0.join("src/main.rs")).unwrap(),
        migrated
    );
}

#[test]
fn lsp_recovers_incremental_edits_and_changes_alias_modes_with_utf16_ranges() {
    let project = Project::new();
    fs::write(project.0.join("src/main.rs"), "fn main(){}\n").unwrap();
    let uri = format!("file://{}", project.0.join("src/main.rs").display());
    let source = "use gpui_rsx::rsx_tw as view;\nfn render(){let name=\"中文😀\";view!{<div class=\"gap-4 flex-colum\"/>};}\nfn unfinished(";
    let class_start = source.find("flex-colum").unwrap();
    let gap_start = source.find("gap-4").unwrap();
    let import_start = source.find("rsx_tw").unwrap();
    let fixed_mode = source
        .replace("rsx_tw", "rsx_strict")
        .replace("fn unfinished(", "");
    let fixed_class = fixed_mode.find("flex-colum").unwrap();
    let messages = vec![
        json!({"id":1,"method":"initialize","params":{}}),
        json!({"method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"version":1,"text":source}}}),
        json!({"id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":uri},"position":position(source,gap_start+2)}}),
        json!({"method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[
            {"range":{"start":position(source,import_start),"end":position(source,import_start+6)},"text":"rsx_strict"},
            {"range":{"start":{"line":2,"character":0},"end":{"line":2,"character":14}},"text":""}
        ]}}),
        json!({"id":3,"method":"textDocument/hover","params":{"textDocument":{"uri":uri},"position":position(&fixed_mode,fixed_mode.find("gap-4").unwrap()+2)}}),
        json!({"method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":3},"contentChanges":[
            {"range":{"start":position(&fixed_mode,fixed_class),"end":position(&fixed_mode,fixed_class+10)},"text":"flex-col"}
        ]}}),
        json!({"id":4,"method":"textDocument/formatting","params":{"textDocument":{"uri":uri}}}),
        json!({"method":"textDocument/didClose","params":{"textDocument":{"uri":uri}}}),
        json!({"id":5,"method":"textDocument/hover","params":{"textDocument":{"uri":uri},"position":position(source,gap_start+2)}}),
        json!({"id":6,"method":"shutdown","params":{}}),
        json!({"method":"exit"}),
    ];
    let responses = project.lsp(messages);
    let response = |id| {
        responses
            .iter()
            .find(|message| message["id"] == id)
            .unwrap()
    };
    let before = response(2)["result"]["contents"]["value"]
        .as_str()
        .unwrap()
        .replace(' ', "");
    let after = response(3)["result"]["contents"]["value"]
        .as_str()
        .unwrap()
        .replace(' ', "");
    assert!(before.contains("px(16f32)"), "{before}");
    assert!(after.contains("px(4f32)"), "{after}");
    assert!(response(4)["result"].is_array());
    assert!(response(5)["result"].is_null());
    let published = responses
        .iter()
        .filter(|message| message["method"] == "textDocument/publishDiagnostics")
        .map(|message| &message["params"])
        .collect::<Vec<_>>();
    assert_eq!(published.len(), 4);
    let first = published[0]["diagnostics"].as_array().unwrap();
    assert!(
        first
            .iter()
            .any(|diagnostic| diagnostic["code"] == "rsx/rust-syntax")
    );
    let typo = first
        .iter()
        .find(|diagnostic| diagnostic["data"]["suggestion"] == "flex-col")
        .unwrap();
    assert_eq!(typo["severity"], 2);
    assert_eq!(typo["range"]["start"], position(source, class_start));
    assert_eq!(typo["range"]["end"], position(source, class_start + 10));
    let second = published[1]["diagnostics"].as_array().unwrap();
    assert!(
        !second
            .iter()
            .any(|diagnostic| diagnostic["code"] == "rsx/rust-syntax")
    );
    assert!(
        second.iter().any(|diagnostic| diagnostic["severity"] == 1
            && diagnostic["data"]["suggestion"] == "flex-col")
    );
    assert_eq!(published[2]["diagnostics"], json!([]));
    assert_eq!(published[3]["diagnostics"], json!([]));
}

#[test]
fn shadowed_macros_are_never_linted_formatted_or_migrated() {
    for local in ["view", "rsx", "视图", "r#type"] {
        let project = Project::new();
        let source = format!(
            "use gpui_rsx::rsx_strict as {local};\nmacro_rules! {local} {{ ($($tokens:tt)*) => {{ stringify!($($tokens)*) }}; }}\nfn main(){{let text = {local}!{{<div class=\"gap-8 flex-colum\"/>}};}}\n"
        );
        fs::write(project.0.join("src/main.rs"), &source).unwrap();
        let linted = project.run(&["lint", "src/main.rs", "--json"]);
        assert!(
            linted.status.success(),
            "{}",
            String::from_utf8_lossy(&linted.stdout)
        );
        let report: Value = serde_json::from_slice(&linted.stdout).unwrap();
        assert_eq!(report["diagnostics"], json!([]));
        for operation in [
            ["fmt", "--check"],
            ["fmt", "--write"],
            ["migrate", "--write"],
        ] {
            let output = project.run(&[operation[0], "src/main.rs", operation[1]]);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                fs::read_to_string(project.0.join("src/main.rs")).unwrap(),
                source
            );
        }
    }
}

#[test]
fn lsp_clears_shadowed_diagnostics_and_keeps_formatting_out_of_custom_macros() {
    let project = Project::new();
    fs::write(project.0.join("src/main.rs"), "fn main(){}\n").unwrap();
    let uri = format!("file://{}", project.0.join("src/main.rs").display());
    let source =
        "use gpui_rsx::rsx_strict as view;\nfn render(){view!{<div class=\"flex-colum gap-8\"/>};}";
    let definition = "macro_rules! r#view { ($($tokens:tt)*) => { stringify!($($tokens)*) }; }\n";
    let changed = source.replacen("fn render()", &format!("{definition}fn render()"), 1);
    let responses = project.lsp(vec![
        json!({"id":1,"method":"initialize","params":{}}),
        json!({"method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"version":1,"text":source}}}),
        json!({"method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[
            {"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":0}},"text":definition}
        ]}}),
        json!({"id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":uri},"position":position(&changed,changed.find("gap-8").unwrap()+2)}}),
        json!({"id":3,"method":"textDocument/formatting","params":{"textDocument":{"uri":uri}}}),
        json!({"method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":3},"contentChanges":[
            {"range":{"start":{"line":1,"character":0},"end":{"line":2,"character":0}},"text":""}
        ]}}),
        json!({"id":4,"method":"textDocument/hover","params":{"textDocument":{"uri":uri},"position":position(source,source.find("gap-8").unwrap()+2)}}),
        json!({"id":5,"method":"shutdown","params":{}}),
        json!({"method":"exit"}),
    ]);
    let response = |id| {
        responses
            .iter()
            .find(|message| message["id"] == id)
            .unwrap()
    };
    assert!(response(2)["result"].is_null());
    assert_eq!(response(3)["result"], json!([]));
    assert!(
        response(4)["result"]["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("8")
    );
    let published = responses
        .iter()
        .filter(|message| message["method"] == "textDocument/publishDiagnostics")
        .map(|message| &message["params"]["diagnostics"])
        .collect::<Vec<_>>();
    assert_eq!(published.len(), 3);
    assert!(!published[0].as_array().unwrap().is_empty());
    assert_eq!(*published[1], json!([]));
    assert_eq!(published[0], published[2]);
}
