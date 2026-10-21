use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn frame(value: Value) -> String {
    let body = value.to_string();
    format!("Content-Length: {}\r\n\r\n{body}", body.len())
}
#[test]
fn language_server_serves_real_framed_requests() {
    let mut process = Command::new(env!("CARGO_BIN_EXE_cargo-rsx"))
        .arg("lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let source = "fn f(){let _=rsx!{<div class=\"flex-colum bg-[#f00]\">{\"中文😀\"}</div>};}";
    let start = source.find("flex-colum").unwrap();
    let messages = [
        json!({"id":1,"method":"initialize","params":{}}),
        json!({"method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/test.rs","version":1,"text":source}}}),
        json!({"id":2,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///tmp/test.rs"},"position":{"line":0,"character":start+5}}}),
        json!({"id":3,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///tmp/test.rs"},"position":{"line":0,"character":start+5}}}),
        json!({"id":4,"method":"textDocument/documentColor","params":{"textDocument":{"uri":"file:///tmp/test.rs"}}}),
        json!({"id":5,"method":"textDocument/codeAction","params":{"textDocument":{"uri":"file:///tmp/test.rs"}}}),
        json!({"id":6,"method":"shutdown","params":{}}),
        json!({"method":"exit"}),
    ];
    let mut stdin = process.stdin.take().unwrap();
    for m in messages {
        stdin.write_all(frame(m).as_bytes()).unwrap();
    }
    drop(stdin);
    let output = process.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut raw = std::str::from_utf8(&output.stdout).unwrap();
    let mut values = Vec::new();
    while !raw.is_empty() {
        let (header, body) = raw.split_once("\r\n\r\n").unwrap();
        let length = header
            .strip_prefix("Content-Length: ")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        values.push(serde_json::from_str::<Value>(&body[..length]).unwrap());
        raw = &body[length..];
    }
    assert_eq!(
        values[0]["result"]["capabilities"]["positionEncoding"],
        "utf-16"
    );
    assert!(
        values[1]["params"]["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("flex-col")
    );
    let completion = values.iter().find(|v| v["id"] == 2).unwrap();
    assert!(
        completion["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "flex-col")
    );
    let colors = values.iter().find(|v| v["id"] == 4).unwrap();
    assert_eq!(colors["result"][0]["color"]["red"], 1.0);
    let fixes = values.iter().find(|v| v["id"] == 5).unwrap();
    assert!(!fixes["result"].as_array().unwrap().is_empty());
}

#[test]
fn cli_lint_json_and_exit_status_match_findings() {
    let root = std::env::temp_dir().join(format!("rsx-cli-test-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let file = root.join("test.rs");
    std::fs::write(&file, r#"fn f(){rsx_strict!{<div class="dark:flex"/>};}"#).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-rsx"))
        .args(["rsx", "lint"])
        .arg(&file)
        .arg("--json")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], false);
    assert!(!report["diagnostics"].as_array().unwrap().is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn watch_preview_reflects_file_changes_without_recompiling() {
    use std::io::{BufRead, Read};
    let root = std::env::temp_dir().join(format!("rsx-preview-test-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let file = root.join("layout.rsx");
    std::fs::write(&file, "<div>{\"before\"}</div>").unwrap();
    let mut process = Command::new(env!("CARGO_BIN_EXE_cargo-rsx"))
        .arg("preview")
        .arg(&file)
        .args(["--watch", "--port", "0"])
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let mut line = String::new();
    std::io::BufReader::new(process.stderr.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    let address = line
        .split("http://")
        .nth(1)
        .unwrap()
        .split(' ')
        .next()
        .unwrap();
    let get = || {
        let mut stream = std::net::TcpStream::connect(address).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();
        stream
            .write_all(b"GET /content HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    };
    let before = get();
    std::fs::write(&file, "<div>{\"after\"}</div>").unwrap();
    let after = get();
    process.kill().unwrap();
    process.wait().unwrap();
    std::fs::remove_dir_all(root).unwrap();
    assert!(before.contains("before"));
    assert!(after.contains("after"));
    assert!(!after.contains("before"));
}
