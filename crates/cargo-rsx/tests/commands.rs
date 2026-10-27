use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn frame(value: Value) -> String {
    let body = value.to_string();
    format!("Content-Length: {}\r\n\r\n{body}", body.len())
}
/// Feeds raw bytes to `cargo rsx lsp` and returns its exit status and decoded responses.
fn run_lsp(input: &str) -> (std::process::ExitStatus, Vec<Value>) {
    let mut process = Command::new(env!("CARGO_BIN_EXE_cargo-rsx"))
        .arg("lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    process
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = process.wait_with_output().unwrap();
    let mut raw = std::str::from_utf8(&output.stdout).unwrap();
    let mut values = Vec::new();
    while let Some((header, body)) = raw.split_once("\r\n\r\n") {
        let length: usize = header["Content-Length: ".len()..].parse().unwrap();
        values.push(serde_json::from_str(&body[..length]).unwrap());
        raw = &body[length..];
    }
    (output.status, values)
}

#[test]
fn language_server_survives_bad_input_and_follows_lifecycle() {
    let uri = "file:///tmp/broken.rs";
    let source = "fn f(){let _=rsx!{<div class=\"flex\">};}";
    let input = [
        frame(json!({"id":1,"method":"textDocument/hover","params":{}})),
        "Content-Length: 5\r\n\r\n{bad}".to_owned(),
        frame(json!({"id":2,"method":"initialize","params":{}})),
        frame(
            json!({"method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"version":1,"text":source}}}),
        ),
        frame(
            json!({"id":3,"method":"textDocument/hover","params":{"textDocument":{"uri":uri},"position":{"line":0,"character":20}}}),
        ),
        frame(json!({"id":4,"method":"shutdown"})),
        frame(json!({"id":5,"method":"textDocument/hover","params":{}})),
        frame(json!({"method":"exit"})),
    ]
    .concat();
    let (status, values) = run_lsp(&input);
    let by_id = |id: i64| values.iter().find(|v| v["id"] == id).unwrap();

    assert!(status.success());
    assert_eq!(by_id(1)["error"]["code"], -32002);
    assert!(
        values
            .iter()
            .any(|v| v["id"].is_null() && v["error"]["code"] == -32700)
    );
    assert!(by_id(2)["result"]["capabilities"].is_object());
    assert!(by_id(3)["result"].is_null());
    assert!(by_id(4)["error"].is_null());
    assert_eq!(by_id(5)["error"]["code"], -32600);
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
fn watch_preview_refreshes_same_size_edits_with_unchanged_timestamps() {
    use std::io::{BufRead, Read};
    let root = std::env::temp_dir().join(format!("rsx-preview-test-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let file = root.join("layout.rsx");
    std::fs::write(&file, "<div>{\"before\"}</div>").unwrap();
    let modified = std::fs::metadata(&file).unwrap().modified().unwrap();
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
    std::fs::write(&file, "<div>{\"after!\"}</div>").unwrap();
    // Editors can make same-size changes within the filesystem's timestamp
    // resolution. Keep both metadata fields unchanged to exercise that case.
    std::fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(modified)
        .unwrap();
    let after = get();
    process.kill().unwrap();
    process.wait().unwrap();
    std::fs::remove_dir_all(root).unwrap();
    assert!(before.contains("before"));
    assert!(after.contains("after!"));
    assert!(!after.contains("before"));
}

#[test]
fn preview_server_survives_aborted_connections_and_bad_content() {
    use std::io::{BufRead, Read};
    let root = std::env::temp_dir().join(format!("rsx-preview-robust-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let file = root.join("layout.rsx");
    std::fs::write(&file, "<div>{\"ok\"}</div>").unwrap();
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
    let request = |raw: &[u8]| {
        let mut stream = std::net::TcpStream::connect(address).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();
        stream.write_all(raw).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    };

    // A client that connects and closes without sending anything is ignored.
    drop(std::net::TcpStream::connect(address).unwrap());
    assert!(request(b"GET /content HTTP/1.1\r\n\r\n").contains("ok"));

    // Unknown routes are 404s and keep the server alive.
    assert!(request(b"GET /missing HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 404"));
    assert!(request(b"GET / HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 200"));

    // Breaking the previewed file shows the escaped error instead of exiting.
    std::fs::write(&file, "<div>").unwrap();
    let broken = request(b"GET /content HTTP/1.1\r\n\r\n");
    assert!(
        broken.contains("&lt;") || broken.contains("Unclosed tag"),
        "{broken}"
    );
    assert!(!broken.contains("<div>\n"), "{broken}");

    // Fixing it recovers.
    std::fs::write(&file, "<div>{\"fixed\"}</div>").unwrap();
    assert!(request(b"GET /content HTTP/1.1\r\n\r\n").contains("fixed"));

    process.kill().unwrap();
    process.wait().unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

/// Runs `cargo rsx` with `stdin` piped in and returns (exit code, stdout, stderr).
fn run(args: &[&str], stdin: &str) -> (Option<i32>, String, String) {
    let mut process = Command::new(env!("CARGO_BIN_EXE_cargo-rsx"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    process
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let output = process.wait_with_output().unwrap();
    (
        output.status.code(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

#[test]
fn stdin_is_a_filter_for_lint_fmt_and_migrate() {
    let source = "fn f(){rsx!{<div class=\"flex\"><span/></div>};}\n";

    // fmt prints the formatted document and succeeds.
    let (code, stdout, _) = run(&["rsx", "fmt", "-"], source);
    assert_eq!(code, Some(0));
    assert_eq!(
        stdout,
        "fn f(){rsx!{\n    <div class=\"flex\">\n        <span />\n    </div>\n};}\n"
    );

    // --check reports the file name as `-` and fails without printing a document.
    let (code, stdout, stderr) = run(&["rsx", "fmt", "-", "--check"], source);
    assert_eq!(code, Some(1));
    assert!(stdout.is_empty());
    assert_eq!(stderr.trim(), "would change -");

    // Already formatted input is echoed unchanged so the command stays a filter.
    let formatted = "fn f(){rsx!{\n    <div />\n};}\n";
    let (code, stdout, _) = run(&["rsx", "fmt", "-"], formatted);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, formatted);

    // --write has no file to write.
    let (code, _, stderr) = run(&["rsx", "fmt", "-", "--write"], source);
    assert_eq!(code, Some(2));
    assert!(
        stderr.contains("--write cannot be used with stdin"),
        "{stderr}"
    );

    // migrate is a filter too and explains the follow-up step on stderr.
    let (code, stdout, stderr) = run(
        &["rsx", "migrate", "-"],
        "fn f(){rsx!{<div class=\"gap-16\"/>};}\n",
    );
    assert_eq!(code, Some(0));
    assert!(stdout.contains("gap-4"), "{stdout}");
    assert!(stderr.contains("spacing-scale"), "{stderr}");

    // lint reads stdin and reports `-` as the file.
    let (code, stdout, _) = run(
        &["rsx", "lint", "-", "--json"],
        "fn f(){rsx!{<div class=\"flex-colum\"/>};}\n",
    );
    assert_eq!(code, Some(0));
    let report: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["diagnostics"][0]["file"], "-");

    // ... and --deny-warnings turns the same warning into a failure.
    let (code, stdout, _) = run(
        &["rsx", "lint", "-", "--deny-warnings", "--json"],
        "fn f(){rsx!{<div class=\"flex-colum\"/>};}\n",
    );
    assert_eq!(code, Some(1));
    let report: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["ok"], false);
}

#[test]
fn lint_columns_count_characters_not_bytes() {
    // `名` is three bytes but one column; the diagnostic starts on the class token.
    let source = "fn f(){let 名 = 1; rsx!{<div class=\"flex-colum\"/>};}\n";
    let (code, stdout, _) = run(&["rsx", "lint", "-", "--json"], source);
    assert_eq!(code, Some(0));
    let report: Value = serde_json::from_str(&stdout).unwrap();
    let diagnostic = &report["diagnostics"][0];
    assert_eq!(diagnostic["line"], 1);
    assert_eq!(diagnostic["column"], 36);

    // CRLF line endings still report the right line and column.
    let crlf = "fn f(){\r\n    rsx!{<div class=\"flex-colum\"/>};\r\n}";
    let (_, stdout, _) = run(&["rsx", "lint", "-", "--json"], crlf);
    let report: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["diagnostics"][0]["line"], 2);
    assert_eq!(report["diagnostics"][0]["column"], 22);
}

#[test]
fn lint_text_output_and_exit_codes_follow_severity() {
    let (code, stdout, _) = run(
        &["rsx", "lint", "-"],
        "fn f(){rsx!{<div class=\"flex-colum\"/>};}\n",
    );
    assert_eq!(code, Some(0));
    assert!(
        stdout.starts_with("-:1:25: warning: `flex-colum`"),
        "{stdout}"
    );

    let (code, stdout, _) = run(
        &["rsx", "lint", "-"],
        "fn f(){rsx_strict!{<div class=\"nonsense\"/>};}\n",
    );
    assert_eq!(code, Some(1));
    assert!(stdout.contains("-:1:32: error:"), "{stdout}");
}

#[test]
fn rewrite_commands_never_leave_a_partially_formatted_tree() {
    let root = std::env::temp_dir().join(format!("rsx-rewrite-test-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let good = root.join("a.rs");
    let broken = root.join("b.rs");
    std::fs::write(&good, "fn a(){rsx!{<div/>};}\n").unwrap();
    // `b.rs` is not a token stream at all, so it fails before anything is written.
    std::fs::write(&broken, "fn b( { rsx!{<div/>}; }\n").unwrap();

    let (code, _, stderr) = run(&["rsx", "fmt", root.to_str().unwrap(), "--write"], "");
    assert_eq!(code, Some(2));
    assert!(stderr.contains("b.rs"), "{stderr}");
    assert_eq!(
        std::fs::read_to_string(&good).unwrap(),
        "fn a(){rsx!{<div/>};}\n",
        "a.rs must not be rewritten when another file fails"
    );

    // Without the broken file, both files are rewritten.
    std::fs::remove_file(&broken).unwrap();
    let (code, _, stderr) = run(&["rsx", "fmt", root.to_str().unwrap(), "--write"], "");
    assert_eq!(code, Some(0));
    assert!(stderr.contains("updated"), "{stderr}");
    assert_eq!(
        std::fs::read_to_string(&good).unwrap(),
        "fn a(){rsx!{\n    <div />\n};}\n"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn migrate_refuses_projects_that_already_use_the_tailwind_scale() {
    let root = std::env::temp_dir().join(format!("rsx-migrate-test-{}", std::process::id()));
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"migrate-test\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::write(
        root.join("rsx.toml"),
        "[options]\nspacing-scale = \"tailwind\"\n",
    )
    .unwrap();
    let file = root.join("src/lib.rs");
    std::fs::write(&file, "fn f(){rsx!{<div class=\"gap-16\"/>};}\n").unwrap();

    let (code, _, stderr) = run(&["rsx", "migrate", file.to_str().unwrap()], "");
    assert_eq!(code, Some(2));
    assert!(stderr.contains("already uses Tailwind scale"), "{stderr}");
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "fn f(){rsx!{<div class=\"gap-16\"/>};}\n"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn expand_and_convert_read_stdin_and_honor_output() {
    let (code, stdout, _) = run(&["rsx", "expand", "-"], "<div class=\"flex\"/>");
    assert_eq!(code, Some(0));
    assert!(stdout.contains("div().flex();"), "{stdout}");

    // The tailwind flag changes the emitted scale.
    let (_, stdout, _) = run(
        &["rsx", "expand", "-", "--tailwind"],
        "<div class=\"gap-4\"/>",
    );
    assert!(stdout.contains("px(16f32)"), "{stdout}");

    let (code, stdout, _) = run(
        &["rsx", "convert", "-"],
        "const a = <div className=\"p-4\"/>;",
    );
    assert_eq!(code, Some(0));
    let report: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["rsx"], "<div class=\"p-16\" />");

    let root = std::env::temp_dir().join(format!("rsx-output-test-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let output = root.join("out.rs");
    let (code, stdout, _) = run(
        &["rsx", "expand", "-", "--output", output.to_str().unwrap()],
        "<div/>",
    );
    assert_eq!(code, Some(0));
    assert!(stdout.is_empty(), "results go to the file instead");
    assert!(std::fs::read_to_string(&output).unwrap().contains("div();"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn unknown_commands_and_options_exit_with_a_usage_error() {
    let (code, _, stderr) = run(&["rsx", "nope"], "");
    assert_eq!(code, Some(2));
    assert!(stderr.contains("unknown command nope"), "{stderr}");

    let (code, _, stderr) = run(&["rsx", "lint", "--bad"], "");
    assert_eq!(code, Some(2));
    assert!(stderr.contains("unknown option --bad"), "{stderr}");

    let (code, _, stderr) = run(&["rsx", "fmt", "--write", "--check"], "");
    assert_eq!(code, Some(2));
    assert!(stderr.contains("mutually exclusive"), "{stderr}");

    let (code, _, _) = run(&["rsx", "expand", "-"], "<div");
    assert_eq!(code, Some(2));

    let (code, _, stderr) = run(&["rsx", "lint", "-", "extra"], "");
    assert_eq!(code, Some(2));
    assert!(stderr.contains("unexpected argument extra"), "{stderr}");
}

#[test]
fn fmt_without_write_prints_each_changed_file_behind_its_path() {
    let root = std::env::temp_dir().join(format!("rsx-fmt-print-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let file = root.join("a.rs");
    std::fs::write(&file, "fn a(){rsx!{<div/>};}\n").unwrap();

    let (code, stdout, _) = run(&["rsx", "fmt", file.to_str().unwrap()], "");
    assert_eq!(code, Some(0));
    assert_eq!(
        stdout,
        format!(
            "// {}\nfn a(){{rsx!{{\n    <div />\n}};}}\n\n",
            file.display()
        )
    );
    // Printing does not touch the file.
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "fn a(){rsx!{<div/>};}\n"
    );

    // --offline is accepted by commands that shell out to cargo metadata.
    let (code, _, _) = run(&["rsx", "lint", file.to_str().unwrap(), "--offline"], "");
    assert_eq!(code, Some(0));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn preview_selects_a_macro_by_index_and_reports_bad_ones() {
    let root = std::env::temp_dir().join(format!("rsx-index-test-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let file = root.join("page.rs");
    std::fs::write(
        &file,
        "fn f(){rsx!{<div>{\"first\"}</div>};rsx_tw!{<div class=\"gap-4\">{\"second\"}</div>};}\n",
    )
    .unwrap();

    // `--index` picks the n-th macro; `rsx_tw!` is rendered with the Tailwind scale.
    let (code, stdout, _) = run(
        &["rsx", "preview", file.to_str().unwrap(), "--index", "1"],
        "",
    );
    assert_eq!(code, Some(0));
    assert!(stdout.contains("second"), "{stdout}");
    assert!(!stdout.contains("first"), "{stdout}");
    assert!(stdout.contains("gap:16px"), "{stdout}");

    let (code, _, stderr) = run(
        &["rsx", "preview", file.to_str().unwrap(), "--index", "9"],
        "",
    );
    assert_eq!(code, Some(2));
    assert!(stderr.contains("out of range"), "{stderr}");

    // A file without macros can't be previewed as RSX source.
    let plain = root.join("plain.txt");
    std::fs::write(&plain, "<div/>").unwrap();
    let (code, stdout, _) = run(&["rsx", "preview", plain.to_str().unwrap()], "");
    assert_eq!(code, Some(0));
    assert!(stdout.starts_with("<!doctype html>"), "{stdout}");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn help_and_version_describe_every_command() {
    let (code, stdout, _) = run(&["rsx", "--help"], "");
    assert_eq!(code, Some(0));
    for command in [
        "doctor", "lint", "fmt", "migrate", "convert", "expand", "preview", "lsp",
    ] {
        assert!(stdout.contains(command), "{command} missing from help");
    }
    assert!(stdout.contains("--manifest-path"), "{stdout}");

    // `cargo rsx` with no arguments prints help as well.
    let (code, stdout, _) = run(&["rsx"], "");
    assert_eq!(code, Some(0));
    assert!(stdout.starts_with("cargo rsx <"), "{stdout}");

    let (code, stdout, _) = run(&["rsx", "--version"], "");
    assert_eq!(code, Some(0));
    assert_eq!(
        stdout.trim(),
        format!("cargo-rsx {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn directory_lint_recurses_but_skips_generated_folders() {
    let root = std::env::temp_dir().join(format!("rsx-lint-dir-{}", std::process::id()));
    std::fs::create_dir_all(root.join("src/nested")).unwrap();
    std::fs::create_dir_all(root.join("target")).unwrap();
    std::fs::create_dir_all(root.join("node_modules")).unwrap();
    std::fs::write(
        root.join("src/a.rs"),
        "fn a(){rsx!{<div class=\"flex-colum\"/>};}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/nested/b.rs"),
        "fn b(){rsx!{<div class=\"flex-colun\"/>};}\n",
    )
    .unwrap();
    // Non-Rust files, generated folders and symlinks are ignored.
    std::fs::write(
        root.join("src/c.txt"),
        "fn c(){rsx!{<div class=\"nope\"/>};}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("target/generated.rs"),
        "fn g(){rsx!{<div class=\"nonsense\"/>};}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("node_modules/dep.rs"),
        "fn d(){rsx!{<div class=\"nonsense\"/>};}\n",
    )
    .unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("src"), root.join("link")).unwrap();

    let (code, stdout, _) = run(&["rsx", "lint", root.to_str().unwrap(), "--json"], "");
    assert_eq!(code, Some(0));
    let report: Value = serde_json::from_str(&stdout).unwrap();
    let files: Vec<_> = report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| std::path::PathBuf::from(d["file"].as_str().unwrap()))
        .collect();
    assert_eq!(files.len(), 2, "{files:?}");
    assert!(files.iter().any(|f| f.ends_with("src/a.rs")), "{files:?}");
    assert!(
        files.iter().any(|f| f.ends_with("src/nested/b.rs")),
        "{files:?}"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn workspace_lint_without_a_path_uses_cargo_metadata() {
    let root = std::env::temp_dir().join(format!("rsx-lint-ws-{}", std::process::id()));
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"lint-ws-test\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/lib.rs"),
        "fn f(){rsx!{<div class=\"flex-colum\"/>};}\n",
    )
    .unwrap();

    let manifest = root.join("Cargo.toml");
    let (code, stdout, _) = run(
        &[
            "rsx",
            "lint",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--json",
        ],
        "",
    );
    assert_eq!(code, Some(0));
    let report: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["diagnostics"].as_array().unwrap().len(), 1);

    // A manifest that does not exist surfaces cargo's error.
    let (code, _, stderr) = run(
        &[
            "rsx",
            "lint",
            "--manifest-path",
            root.join("missing/Cargo.toml").to_str().unwrap(),
        ],
        "",
    );
    assert_eq!(code, Some(2));
    assert!(!stderr.is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doctor_reports_a_healthy_project_without_gpui() {
    let root = std::env::temp_dir().join(format!("rsx-doctor-{}", std::process::id()));
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"doctor-test\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::write(root.join("src/lib.rs"), "pub fn f() {}\n").unwrap();

    let (code, stdout, _) = run(
        &[
            "rsx",
            "doctor",
            "--manifest-path",
            root.join("Cargo.toml").to_str().unwrap(),
            "--json",
        ],
        "",
    );
    assert_eq!(code, Some(0));
    let report: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert!(report["engines"].as_array().unwrap().is_empty());
    assert!(
        report["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().contains("No GPUI runtime")),
        "{report}"
    );

    // The text form names the outcome.
    let (code, stdout, _) = run(
        &[
            "rsx",
            "doctor",
            "--manifest-path",
            root.join("Cargo.toml").to_str().unwrap(),
        ],
        "",
    );
    assert_eq!(code, Some(0));
    assert!(stdout.contains("doctor: passed"), "{stdout}");
    std::fs::remove_dir_all(root).unwrap();
}
