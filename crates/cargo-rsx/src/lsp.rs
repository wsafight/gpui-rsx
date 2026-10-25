//! A synchronous stdio LSP server. Full document sync keeps editing deterministic.
use gpui_rsx_core::{ClassMode, ClassOutcome, analyze_class, known_classes};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
    path::PathBuf,
    sync::OnceLock,
};

struct Document {
    source: String,
    mode: ClassMode,
    calls: Vec<gpui_rsx_tools::MacroCall>,
    /// Absolute byte ranges for static class string contents.
    literals: Vec<(usize, usize)>,
    diagnostics: Vec<gpui_rsx_tools::Diagnostic>,
}

impl Document {
    fn new(uri: &str, source: &str) -> Self {
        Self::with_mode(source, mode(uri))
    }

    /// Reuses an already resolved mode; resolving it walks the filesystem.
    fn with_mode(source: &str, mode: ClassMode) -> Self {
        let analysis = gpui_rsx_tools::analyze_rust_document(source, mode);
        Self {
            source: source.to_owned(),
            mode,
            calls: analysis.calls,
            literals: analysis.literals,
            diagnostics: analysis.diagnostics,
        }
    }
}

fn completion_classes() -> &'static [String] {
    static CLASSES: OnceLock<Vec<String>> = OnceLock::new();
    CLASSES.get_or_init(|| {
        let mut classes = known_classes().to_vec();
        classes.extend(
            gpui_rsx_core::SUPPORTED_VARIANTS
                .iter()
                .map(|variant| format!("{variant}:")),
        );
        classes.sort();
        classes.dedup();
        classes
    })
}

fn append_prefixed_classes(classes: &[String], prefix: &str, variant: &str, out: &mut Vec<String>) {
    let start = classes.partition_point(|class| class.as_str() < prefix);
    for class in classes[start..]
        .iter()
        .take_while(|class| class.starts_with(prefix))
    {
        out.push(format!("{variant}{class}"));
    }
}

fn completion_candidates(prefix: &str, mode: ClassMode) -> Vec<String> {
    let (variant, class_prefix) = prefix
        .rsplit_once(':')
        .map(|(variant, class)| (format!("{variant}:"), class))
        .unwrap_or_else(|| (String::new(), prefix));
    let mut candidates = Vec::new();
    append_prefixed_classes(
        completion_classes(),
        class_prefix,
        &variant,
        &mut candidates,
    );
    if let Some(theme) = mode.theme {
        // Theme lists are small and the result is sorted below, so a linear filter suffices.
        candidates.extend(
            theme
                .class_names()
                .into_iter()
                .filter(|class| class.starts_with(class_prefix))
                .map(|class| format!("{variant}{class}")),
        );
    }
    candidates.sort();
    candidates.dedup();
    candidates
}

fn send(out: &mut impl Write, message: Value) -> Result<(), String> {
    let body = message.to_string();
    write!(out, "Content-Length: {}\r\n\r\n{body}", body.len())
        .and_then(|()| out.flush())
        .map_err(|e| e.to_string())
}
/// Outer `Err`: the stream is unusable. Inner `Err`: one malformed body, which is skippable
/// because framing is still intact.
fn read(input: &mut impl BufRead) -> Result<Option<Result<Value, String>>, String> {
    let mut length = None;
    let mut header_bytes = 0;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
            return Ok(None);
        }
        header_bytes += line.len();
        if header_bytes > 8192 {
            return Err("LSP header too large".into());
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((key, value)) = line.split_once(':')
            && key.eq_ignore_ascii_case("Content-Length")
        {
            length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "invalid Content-Length")?,
            );
        }
    }
    let length = length.ok_or("missing Content-Length")?;
    if length > 16 * 1024 * 1024 {
        return Err("LSP message exceeds 16 MiB".into());
    }
    let mut body = vec![0; length];
    input.read_exact(&mut body).map_err(|e| e.to_string())?;
    Ok(Some(
        serde_json::from_slice(&body).map_err(|e| e.to_string()),
    ))
}
fn position(source: &str, offset: usize) -> Value {
    let mut offset = offset.min(source.len());
    while !source.is_char_boundary(offset) {
        offset -= 1;
    }
    let before = &source[..offset];
    json!({"line": before.bytes().filter(|&b| b == b'\n').count(), "character": before.rsplit('\n').next().unwrap_or("").encode_utf16().count()})
}
fn offset(source: &str, pos: &Value) -> usize {
    let line = pos["line"].as_u64().unwrap_or(0) as usize;
    let column = pos["character"].as_u64().unwrap_or(0) as usize;
    let start: usize = source.split_inclusive('\n').take(line).map(str::len).sum();
    let rest = &source[start.min(source.len())..];
    let mut units = 0;
    for (i, c) in rest.char_indices() {
        if units >= column || c == '\n' {
            return start + i;
        }
        units += c.len_utf16();
    }
    source.len()
}
fn range(source: &str, start: usize, end: usize) -> Value {
    json!({"start":position(source,start),"end":position(source,end)})
}
fn uri_path(uri: &str) -> Option<PathBuf> {
    let raw = uri.strip_prefix("file://")?;
    let raw = if cfg!(windows) {
        raw.strip_prefix('/').unwrap_or(raw)
    } else {
        raw
    };
    let mut bytes = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        if raw.as_bytes()[i] == b'%' && i + 2 < raw.len() {
            bytes.push(u8::from_str_radix(raw.get(i + 1..i + 3)?, 16).ok()?);
            i += 3;
        } else {
            bytes.push(raw.as_bytes()[i]);
            i += 1;
        }
    }
    Some(String::from_utf8(bytes).ok()?.into())
}
fn mode(uri: &str) -> ClassMode {
    let path = uri_path(uri);
    super::mode(&super::Options::default(), path.as_deref()).unwrap_or(ClassMode::PERMISSIVE)
}
fn mode_at(document: Option<&Document>, cursor: usize, uri: &str) -> ClassMode {
    let base = document.map_or_else(|| mode(uri), |document| document.mode);
    document
        .and_then(|document| {
            document
                .calls
                .iter()
                .find(|call| cursor >= call.start && cursor <= call.end)
        })
        .map(|call| gpui_rsx_tools::mode_for(&call.name, base))
        .unwrap_or(base)
}

fn diagnostics(document: &Document) -> Vec<Value> {
    document.diagnostics.iter().map(|d| json!({
        "range":range(&document.source,d.start,d.end),"severity": match d.severity.as_str() {"error"=>1,"warning"=>2,_=>3},
        "source":"gpui-rsx","message":d.message,"data":{"suggestion":d.suggestion}
    })).collect()
}

/// A class token under the cursor; refuses text outside an RSX macro or class literal.
fn class_at(document: &Document, cursor: usize) -> Option<(usize, usize, String)> {
    let source = &document.source;
    if !document
        .calls
        .iter()
        .any(|call| cursor >= call.start && cursor <= call.end)
    {
        return None;
    }
    for &(start, end) in &document.literals {
        if cursor < start || cursor > end {
            continue;
        }
        let before = &source[start..cursor];
        let after = &source[cursor..end];
        let word_start = before
            .rfind(char::is_whitespace)
            .map(|i| start + i + 1)
            .unwrap_or(start);
        let word_end = after
            .find(char::is_whitespace)
            .map(|i| cursor + i)
            .unwrap_or(end);
        return Some((word_start, word_end, source[word_start..word_end].into()));
    }
    None
}

fn request(
    method: &str,
    params: &Value,
    docs: &HashMap<String, Document>,
) -> Result<Value, String> {
    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
    let document = docs.get(uri);
    let source = document.map_or("", |document| document.source.as_str());
    let cursor = offset(source, &params["position"]);
    match method {
        "initialize" => Ok(json!({"capabilities":{
            "positionEncoding":"utf-16", "textDocumentSync":{"openClose":true,"change":1},
            "completionProvider":{"triggerCharacters":["\"",":","-"," "]},
            "hoverProvider":true,"documentFormattingProvider":true,"colorProvider":true,"codeActionProvider":true
        },"serverInfo":{"name":"gpui-rsx","version":env!("CARGO_PKG_VERSION")}})),
        "shutdown" => Ok(Value::Null),
        "textDocument/completion" => {
            let Some(document) = document else {
                return Ok(json!({"isIncomplete":false,"items":[]}));
            };
            let Some((start, _, word)) = class_at(document, cursor) else {
                return Ok(json!({"isIncomplete":false,"items":[]}));
            };
            let prefix = &source[start..cursor];
            let class_mode = mode_at(Some(document), cursor, uri);
            let mut matches = Vec::with_capacity(201);
            for class in completion_candidates(prefix, class_mode) {
                if class.ends_with(':')
                    || matches!(
                        analyze_class(&class, class_mode).outcome,
                        ClassOutcome::Supported(_)
                    )
                {
                    matches.push(class);
                    if matches.len() == 201 {
                        break;
                    }
                }
            }
            let incomplete = matches.len() > 200;
            matches.truncate(200);
            Ok(
                json!({"isIncomplete":incomplete,"items":matches.into_iter().map(|c|json!({"label":c,"kind":12,"textEdit":{"range":range(source,start,start+word.len()),"newText":c}})).collect::<Vec<_>>()}),
            )
        }
        "textDocument/hover" => {
            if let Some(document) = document
                && let Some((start, end, class)) = class_at(document, cursor)
            {
                let tokens = match analyze_class(&class, mode_at(Some(document), cursor, uri))
                    .outcome
                {
                    ClassOutcome::Supported(t) | ClassOutcome::PassThrough { tokens: t, .. } => {
                        t.to_string()
                    }
                    ClassOutcome::Ignored { message } | ClassOutcome::Error { message } => message,
                };
                Ok(
                    json!({"contents":{"kind":"markdown","value":format!("```rust\n{tokens}\n```")},"range":range(source,start,end)}),
                )
            } else if let Some(document) = document
                && let Some(call) = document
                    .calls
                    .iter()
                    .find(|c| cursor >= c.start && cursor <= c.end)
            {
                // Code being edited often fails to expand; diagnostics already report that.
                Ok(gpui_rsx_tools::expand(
                    &source[call.start..call.end],
                    gpui_rsx_tools::mode_for(&call.name, document.mode),
                )
                .map_or(Value::Null, |expansion| {
                    json!({"contents":{"kind":"markdown","value":format!("```rust\n{expansion}\n```")}})
                }))
            } else {
                Ok(Value::Null)
            }
        }
        "textDocument/formatting" => {
            let result = gpui_rsx_tools::format::rust(source)?;
            Ok(json!([{"range":range(source,0,source.len()),"newText":result}]))
        }
        "textDocument/documentColor" => {
            let mut colors = Vec::new();
            if let Some(document) = document {
                for &(start, end) in &document.literals {
                    let value = &source[start..end];
                    let mut at = 0;
                    for class in value.split_ascii_whitespace() {
                        let begin = value[at..].find(class).unwrap_or(0) + at;
                        at = begin + class.len();
                        let base = class.rsplit(':').next().unwrap_or(class);
                        let color = ["bg-", "text-", "border-"]
                            .iter()
                            .find_map(|prefix| base.strip_prefix(prefix))
                            .and_then(color);
                        if let Some((red, green, blue, alpha)) = color {
                            colors.push(json!({"range":range(source,start+begin,start+at),"color":{"red":red,"green":green,"blue":blue,"alpha":alpha}}));
                        }
                    }
                }
            }
            Ok(json!(colors))
        }
        "textDocument/colorPresentation" => {
            let c = &params["color"];
            let channel =
                |name| (c[name].as_f64().unwrap_or(0.0).clamp(0.0, 1.0) * 255.0).round() as u8;
            let start = offset(source, &params["range"]["start"]);
            let end = offset(source, &params["range"]["end"]);
            let original = source.get(start..end).unwrap_or("bg-");
            let prefix = original
                .rfind("bg-")
                .or_else(|| original.rfind("text-"))
                .or_else(|| original.rfind("border-"));
            let prefix = prefix
                .map(|i| &original[..i + original[i..].find('-').unwrap_or(0) + 1])
                .unwrap_or("bg-");
            let label = format!(
                "{prefix}[#{:02x}{:02x}{:02x}{:02x}]",
                channel("red"),
                channel("green"),
                channel("blue"),
                channel("alpha")
            );
            Ok(json!([{"label":label,"textEdit":{"range":params["range"],"newText":label}}]))
        }
        "textDocument/codeAction" => {
            let mut actions = Vec::new();
            for d in document.map_or(&[][..], |document| document.diagnostics.as_slice()) {
                if let Some(suggestion) = d.suggestion.clone() {
                    let mut changes = serde_json::Map::new();
                    changes.insert(
                        uri.into(),
                        json!([{"range":range(source,d.start,d.end),"newText":suggestion}]),
                    );
                    actions.push(json!({"title":format!("Replace with {suggestion}"),"kind":"quickfix","edit":{"changes":changes}}));
                }
            }
            Ok(json!(actions))
        }
        _ => Err(format!("unsupported method {method}")),
    }
}
fn color(value: &str) -> Option<(f64, f64, f64, f64)> {
    let rgba = gpui_rsx_core::codegen::class::color_rgba(value)?;
    Some((
        (rgba >> 24) as f64 / 255.0,
        ((rgba >> 16) & 255) as f64 / 255.0,
        ((rgba >> 8) & 255) as f64 / 255.0,
        (rgba & 255) as f64 / 255.0,
    ))
}

pub fn run() -> Result<(), String> {
    let mut input = io::BufReader::new(io::stdin().lock());
    let mut out = io::stdout().lock();
    let mut docs: HashMap<String, Document> = HashMap::new();
    let mut initialized = false;
    let mut shutdown = false;
    while let Some(message) = read(&mut input)? {
        let result = handle(
            message,
            &mut out,
            &mut docs,
            &mut initialized,
            &mut shutdown,
        );
        // Nothing kept across messages holds a span, so the per-thread source map that
        // `span-locations` parsing fills can be dropped; otherwise it grows on every edit.
        gpui_rsx_tools::release_span_memory();
        match result? {
            Flow::Continue => {}
            Flow::Exit(result) => return result,
        }
    }
    Ok(())
}

enum Flow {
    Continue,
    Exit(Result<(), String>),
}

fn handle(
    message: Result<Value, String>,
    out: &mut impl Write,
    docs: &mut HashMap<String, Document>,
    initialized: &mut bool,
    shutdown: &mut bool,
) -> Result<Flow, String> {
    let message = match message {
        Ok(message) => message,
        Err(error) => {
            send(
                out,
                json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":error}}),
            )?;
            return Ok(Flow::Continue);
        }
    };
    let method = message["method"].as_str().unwrap_or("");
    let params = &message["params"];
    if method == "exit" {
        return Ok(Flow::Exit(if *shutdown {
            Ok(())
        } else {
            Err("exit before shutdown".into())
        }));
    }
    if method == "textDocument/didOpen" || method == "textDocument/didChange" {
        let doc = &params["textDocument"];
        let uri = doc["uri"].as_str().unwrap_or("").to_owned();
        let text = if method.ends_with("didOpen") {
            doc["text"].as_str()
        } else {
            params["contentChanges"]
                .as_array()
                .and_then(|c| c.last())
                .and_then(|c| c["text"].as_str())
        };
        if let Some(text) = text {
            let document = match docs.get(&uri) {
                Some(previous) if method.ends_with("didChange") => {
                    Document::with_mode(text, previous.mode)
                }
                _ => Document::new(&uri, text),
            };
            send(
                out,
                json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"version":doc["version"],"diagnostics":diagnostics(&document)}}),
            )?;
            docs.insert(uri, document);
        }
    } else if method == "workspace/didChangeWatchedFiles" {
        let uris: Vec<_> = docs.keys().cloned().collect();
        for uri in uris {
            if let Some(document) = docs.get(&uri) {
                let refreshed = Document::new(&uri, &document.source);
                let diagnostics = diagnostics(&refreshed);
                docs.insert(uri.clone(), refreshed);
                send(
                    out,
                    json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"diagnostics":diagnostics}}),
                )?;
            }
        }
    } else if method == "textDocument/didClose" {
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        docs.remove(uri);
        send(
            out,
            json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"diagnostics":[]}}),
        )?;
    } else if let Some(id) = message.get("id") {
        let response = if *shutdown {
            json!({"jsonrpc":"2.0","id":id,"error":{"code":-32600,"message":"server is shutting down"}})
        } else if !*initialized && method != "initialize" {
            json!({"jsonrpc":"2.0","id":id,"error":{"code":-32002,"message":"server not initialized"}})
        } else {
            match method {
                "initialize" => *initialized = true,
                "shutdown" => *shutdown = true,
                _ => {}
            }
            match request(method, params, docs) {
                Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":result}),
                Err(error) => {
                    json!({"jsonrpc":"2.0","id":id,"error":{"code":if error.starts_with("unsupported method") {-32601} else {-32603},"message":error}})
                }
            }
        };
        send(out, response)?;
    }
    Ok(Flow::Continue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_rsx_core::{Theme, theme::TokenValue};
    use std::collections::BTreeMap;

    #[test]
    fn utf16_positions_roundtrip() {
        let source = "// 中文😀\nlet x = 1;";
        for (i, _) in source.char_indices() {
            assert_eq!(offset(source, &position(source, i)), i);
        }
    }

    fn completion_response(source: &str, cursor: usize, theme: Option<&'static Theme>) -> Value {
        let uri = "file:///tmp/rsx-lsp-test.rs";
        let mut document = Document::new(uri, source);
        if let Some(theme) = theme {
            document.mode = document.mode.with_theme(Some(theme));
        }
        let docs = HashMap::from([(uri.to_owned(), document)]);
        let params = json!({
            "textDocument":{"uri":uri},
            "position":position(source,cursor)
        });
        request("textDocument/completion", &params, &docs).unwrap()
    }

    fn labels(response: &Value) -> Vec<&str> {
        response["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["label"].as_str().unwrap())
            .collect()
    }

    #[test]
    fn completion_resolves_variant_prefixes_and_keeps_results_sorted() {
        let source = "fn f(){rsx!{<div class=\"hover:f\"/>}}";
        let prefix = "hover:f";
        let cursor = source.find(prefix).unwrap() + prefix.len();
        let response = completion_response(source, cursor, None);
        let labels = labels(&response);

        assert!(labels.contains(&"hover:focus:"));
        assert!(labels.contains(&"hover:flex"));
        assert!(!labels.contains(&"hover:hover:"));
        assert!(labels.windows(2).all(|pair| pair[0] <= pair[1]));
        assert!(!response["isIncomplete"].as_bool().unwrap());
    }

    #[test]
    fn completion_includes_theme_tokens() {
        static THEME: OnceLock<Theme> = OnceLock::new();
        let theme = THEME.get_or_init(|| Theme {
            colors: BTreeMap::from([(
                "accent".into(),
                TokenValue::Color {
                    hex: 0x123456,
                    alpha: false,
                },
            )]),
            ..Theme::default()
        });
        let source = "fn f(){rsx!{<div class=\"bg-acc\"/>}}";
        let prefix = "bg-acc";
        let cursor = source.find(prefix).unwrap() + prefix.len();
        let response = completion_response(source, cursor, Some(theme));

        assert!(labels(&response).contains(&"bg-accent"));
    }

    #[test]
    fn completion_caps_results_and_marks_truncated_lists_incomplete() {
        let source = "fn f(){rsx!{<div class=\"\"/>}}";
        let cursor = source.find("class=\"").unwrap() + "class=\"".len();
        let response = completion_response(source, cursor, None);
        let labels = labels(&response);

        assert_eq!(labels.len(), 200);
        assert!(response["isIncomplete"].as_bool().unwrap());
        assert!(labels.windows(2).all(|pair| pair[0] <= pair[1]));
    }

    fn request_at(
        method: &str,
        source: &str,
        cursor: usize,
        extra: Value,
    ) -> (Value, HashMap<String, Document>) {
        let uri = "file:///tmp/rsx-lsp-test.rs";
        let document = Document::new(uri, source);
        let docs = HashMap::from([(uri.to_owned(), document)]);
        let mut params = json!({
            "textDocument":{"uri":uri},
            "position":position(source,cursor)
        });
        for (key, value) in extra.as_object().unwrap() {
            params[key] = value.clone();
        }
        (request(method, &params, &docs).unwrap(), docs)
    }

    #[test]
    fn completion_is_empty_outside_macros_and_class_literals() {
        let source = "fn f(){let x = 1; rsx!{<div id=\"abc\"/>}}";
        // In ordinary Rust code.
        let (response, _) = request_at("textDocument/completion", source, 10, json!({}));
        assert_eq!(response["items"].as_array().unwrap().len(), 0);
        assert!(!response["isIncomplete"].as_bool().unwrap());
        // Inside the macro but outside any class literal (`id="abc"`).
        let id = source.find("abc").unwrap();
        let (response, _) = request_at("textDocument/completion", source, id + 1, json!({}));
        assert_eq!(response["items"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn completion_filters_unsupported_entries_and_supports_theme_variants() {
        static THEME: OnceLock<Theme> = OnceLock::new();
        let theme = THEME.get_or_init(|| Theme {
            colors: BTreeMap::from([(
                "accent".into(),
                TokenValue::Color {
                    hex: 0x123456,
                    alpha: false,
                },
            )]),
            ..Theme::default()
        });
        let source = "fn f(){rsx!{<div class=\"hover:bg-acc\"/>}}";
        let prefix = "hover:bg-acc";
        let cursor = source.find(prefix).unwrap() + prefix.len();
        let response = completion_response(source, cursor, Some(theme));
        let labels = labels(&response);

        assert_eq!(labels, ["hover:bg-accent"]);
        // The text edit replaces the whole word, not just the typed prefix.
        let edit = &response["items"][0]["textEdit"];
        assert_eq!(edit["newText"], "hover:bg-accent");
        assert_eq!(
            edit["range"]["start"],
            position(source, source.find(prefix).unwrap())
        );
        assert_eq!(
            edit["range"]["end"],
            position(source, source.find(prefix).unwrap() + prefix.len())
        );
    }

    #[test]
    fn completion_replaces_the_whole_word_under_the_cursor() {
        let source = "fn f(){rsx!{<div class=\"flex-col\"/>}}";
        let start = source.find("flex-col").unwrap();
        let (response, _) = request_at("textDocument/completion", source, start + 4, json!({}));
        let item = &response["items"][0];
        assert_eq!(item["textEdit"]["range"]["start"], position(source, start));
        assert_eq!(
            item["textEdit"]["range"]["end"],
            position(source, start + "flex-col".len())
        );
        assert!(labels(&response).contains(&"flex-col"));
    }

    #[test]
    fn hover_shows_class_expansions_and_reports_unsupported_classes() {
        let source = "fn f(){rsx!{<div class=\"bg-red-500 dark:flex\"/>}}";

        let class = source.find("bg-red-500").unwrap();
        let (response, _) = request_at("textDocument/hover", source, class + 2, json!({}));
        let value = response["contents"]["value"].as_str().unwrap();
        // The token stream is rendered as generated Rust, which `quote` spaces out.
        assert!(value.replace(' ', "").contains(".bg(rgb("), "{value}");
        assert_eq!(
            response["range"]["start"],
            position(source, source.find("bg-red-500").unwrap())
        );

        // Unsupported classes explain themselves instead of showing an expansion.
        let dark = source.find("dark:flex").unwrap();
        let (response, _) = request_at("textDocument/hover", source, dark + 2, json!({}));
        let value = response["contents"]["value"].as_str().unwrap();
        assert!(value.contains("color-scheme"), "{value}");
    }

    #[test]
    fn hover_falls_back_to_the_macro_expansion_and_never_errors() {
        let source = "fn f(){rsx!{<div class=\"flex\"/>}}";
        // Between attributes, inside the macro: show the whole expansion.
        let cursor = source.find("<div").unwrap();
        let (response, _) = request_at("textDocument/hover", source, cursor + 1, json!({}));
        let value = response["contents"]["value"].as_str().unwrap();
        assert!(value.contains(".flex()"), "{value}");

        // A body that cannot be expanded yields null rather than an error response.
        let broken = "fn f(){rsx!{<div>};}";
        let (response, _) = request_at("textDocument/hover", broken, 13, json!({}));
        assert!(response.is_null());

        // Outside any macro there is nothing to show.
        let (response, _) = request_at("textDocument/hover", source, 2, json!({}));
        assert!(response.is_null());
    }

    #[test]
    fn document_colors_cover_palette_hex_rgba_and_variants() {
        let source = "fn f(){rsx!{<div class=\"bg-red-500 text-[#abc] border-[#11223344] hover:bg-blue-600\"/>}}";
        let (response, _) = request_at("textDocument/documentColor", source, 0, json!({}));
        let colors = response.as_array().unwrap();
        assert_eq!(colors.len(), 4, "{colors:?}");

        let rgba = |index: usize| {
            let c = &colors[index]["color"];
            let channel = |name: &str| c[name].as_f64().unwrap();
            (
                (channel("red") * 255.0).round() as u8,
                (channel("green") * 255.0).round() as u8,
                (channel("blue") * 255.0).round() as u8,
                (channel("alpha") * 255.0).round() as u8,
            )
        };
        assert_eq!(rgba(0), (0xef, 0x44, 0x44, 0xff));
        assert_eq!(rgba(1), (0xaa, 0xbb, 0xcc, 0xff));
        assert_eq!(rgba(2), (0x11, 0x22, 0x33, 0x44));
        assert_eq!(rgba(3), (0x25, 0x63, 0xeb, 0xff));

        // The swatch covers the whole token, variant prefix included.
        let variant = source.find("hover:bg-blue-600").unwrap();
        assert_eq!(colors[3]["range"]["start"], position(source, variant));
        assert_eq!(
            colors[3]["range"]["end"],
            position(source, variant + "hover:bg-blue-600".len())
        );
    }

    #[test]
    fn color_presentation_keeps_the_class_prefix() {
        let source = "fn f(){rsx!{<div class=\"bg-red-500 text-white\"/>}}";
        for (class, expected) in [
            ("bg-red-500", "bg-[#336699ff]"),
            ("text-white", "text-[#336699ff]"),
        ] {
            let start = source.find(class).unwrap();
            let range =
                json!({"start":position(source,start),"end":position(source,start+class.len())});
            let (response, _) = request_at(
                "textDocument/colorPresentation",
                source,
                start,
                json!({"color":{"red":0.2,"green":0.4,"blue":0.6,"alpha":1.0},"range":range}),
            );
            assert_eq!(response[0]["label"], expected);
            assert_eq!(response[0]["textEdit"]["newText"], expected);
        }
    }

    #[test]
    fn formatting_returns_a_full_document_replacement() {
        let source = "fn f(){rsx!{<div class=\"flex\"><span/></div>};}";
        let (response, _) = request_at("textDocument/formatting", source, 0, json!({}));
        let edit = &response[0];
        assert_eq!(edit["range"]["start"], position(source, 0));
        assert_eq!(edit["range"]["end"], position(source, source.len()));
        assert_eq!(
            edit["newText"],
            "fn f(){rsx!{\n    <div class=\"flex\">\n        <span />\n    </div>\n};}"
        );
    }

    #[test]
    fn code_actions_offer_quick_fixes_for_suggestions_only() {
        let source = "fn f(){rsx!{<div class=\"flex-colum\"/>};}";
        let (response, _) = request_at("textDocument/codeAction", source, 0, json!({}));
        let actions = response.as_array().unwrap();
        assert_eq!(actions.len(), 1, "{actions:?}");
        assert_eq!(actions[0]["title"], "Replace with flex-col");
        assert_eq!(actions[0]["kind"], "quickfix");
        let edit = &actions[0]["edit"]["changes"]["file:///tmp/rsx-lsp-test.rs"][0];
        assert_eq!(edit["newText"], "flex-col");
        assert_eq!(
            edit["range"]["start"],
            position(source, source.find("flex-colum").unwrap())
        );

        // Diagnostics without a suggestion produce no action.
        let clean = "fn f(){rsx!{<div class=\"flex\"/>};}";
        let (response, _) = request_at("textDocument/codeAction", clean, 0, json!({}));
        assert_eq!(response.as_array().unwrap().len(), 0);
    }

    #[test]
    fn diagnostics_map_severities_and_carry_suggestions() {
        let source =
            "fn f(){rsx!{<div class=\"flex-colum\"/>};rsx_strict!{<p class=\"nonsense\"/>};}";
        let (response, _) = request_at("textDocument/codeAction", source, 0, json!({}));
        assert!(!response.as_array().unwrap().is_empty());

        let document = Document::new("file:///tmp/rsx-lsp-test.rs", source);
        let published = diagnostics(&document);
        let severities: Vec<_> = published.iter().map(|d| d["severity"].as_u64()).collect();
        assert!(severities.contains(&Some(2)), "{published:?}");
        assert!(severities.contains(&Some(1)), "{published:?}");
        assert!(
            published
                .iter()
                .any(|d| d["data"]["suggestion"] == "flex-col")
        );
        assert!(published.iter().all(|d| d["source"] == "gpui-rsx"));
    }

    #[test]
    fn class_mode_follows_the_macro_under_the_cursor() {
        let source =
            "fn f(){rsx_strict!{<div class=\"nonsense\"/>};rsx_tw!{<div class=\"gap-4\"/>};}";

        // Strict macro: unknown classes are errors.
        let strict = source.find("nonsense").unwrap();
        let (response, _) = request_at("textDocument/hover", source, strict + 1, json!({}));
        assert!(
            response["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("strict mode")
        );

        // `rsx_tw!` applies the Tailwind scale (4px per unit).
        let tw = source.find("gap-4").unwrap();
        let (response, _) = request_at("textDocument/hover", source, tw + 1, json!({}));
        assert!(
            response["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("16f32")
        );
    }

    #[test]
    fn uri_path_decodes_percent_escapes_and_windows_prefixes() {
        let path = uri_path("file:///tmp/a%20b/c%2Bd.rs").unwrap();
        assert_eq!(path, PathBuf::from("/tmp/a b/c+d.rs"));
        assert_eq!(uri_path("untitled:Untitled-1"), None);
        // Malformed escapes are rejected rather than silently dropped.
        assert_eq!(uri_path("file:///tmp/%zz.rs"), None);
        assert_eq!(uri_path("file:///tmp/%4.rs"), None);
    }

    #[test]
    fn position_clamps_out_of_range_offsets_to_char_boundaries() {
        let source = "中文😀";
        assert_eq!(position(source, 0)["character"], 0);
        // A byte offset inside a character snaps back to its start.
        assert_eq!(position(source, 1)["character"], 0);
        assert_eq!(position(source, 3)["character"], 1);
        assert_eq!(position(source, 6)["character"], 2);
        // Past the end, the offset is clamped; the emoji counts as two UTF-16 units.
        assert_eq!(position(source, 999)["character"], 4);
        assert_eq!(
            offset(source, &json!({"line":0,"character":999})),
            source.len()
        );
        assert_eq!(
            offset(source, &json!({"line":9,"character":0})),
            source.len()
        );
        // A column in the middle of a surrogate pair has no char boundary, so it rounds
        // forward to the end of the emoji.
        assert_eq!(offset(source, &json!({"line":0,"character":2})), 6);
        assert_eq!(offset(source, &json!({"line":0,"character":3})), 10);
        // Columns count UTF-16 units, so the emoji advances the column by two.
        assert_eq!(position(source, 6)["character"], 2);
        assert_eq!(position(source, 10)["character"], 4);
    }

    #[test]
    fn read_skips_malformed_bodies_but_stops_on_broken_framing() {
        use std::io::BufReader;

        let mut input =
            BufReader::new("Content-Length: 5\r\n\r\n{bad}Content-Length: 2\r\n\r\n{}".as_bytes());
        let first = read(&mut input).unwrap().unwrap();
        assert!(first.is_err(), "malformed JSON is reported per message");
        let second = read(&mut input).unwrap().unwrap();
        assert_eq!(second.unwrap(), json!({}), "framing survives a bad body");

        let mut input = BufReader::new("X-Test: 1\r\n\r\n".as_bytes());
        assert_eq!(read(&mut input).unwrap_err(), "missing Content-Length");

        let mut input = BufReader::new("Content-Length: nope\r\n\r\n".as_bytes());
        assert_eq!(read(&mut input).unwrap_err(), "invalid Content-Length");

        let mut input = BufReader::new("Content-Length: 99999999\r\n\r\n".as_bytes());
        assert!(read(&mut input).unwrap_err().contains("16 MiB"));

        // A header that never terminates is rejected without buffering the whole stream.
        let mut header = String::new();
        for _ in 0..2000 {
            header.push_str("X-Pad: 0123456789\r\n");
        }
        let mut input = BufReader::new(header.as_bytes());
        assert_eq!(read(&mut input).unwrap_err(), "LSP header too large");

        let mut input = BufReader::new("".as_bytes());
        assert!(read(&mut input).unwrap().is_none());
    }

    /// The mutable server state `handle` threads between messages.
    struct Server {
        out: Vec<u8>,
        docs: HashMap<String, Document>,
        initialized: bool,
        shutdown: bool,
    }

    impl Server {
        fn new() -> Self {
            Self {
                out: Vec::new(),
                docs: HashMap::new(),
                initialized: false,
                shutdown: false,
            }
        }

        fn run(&mut self, message: Value) -> Flow {
            self.handle(Ok(message))
        }

        fn handle(&mut self, message: Result<Value, String>) -> Flow {
            handle(
                message,
                &mut self.out,
                &mut self.docs,
                &mut self.initialized,
                &mut self.shutdown,
            )
            .unwrap()
        }

        fn output(&self) -> String {
            String::from_utf8_lossy(&self.out).into_owned()
        }
    }

    #[test]
    fn handle_tracks_lifecycle_and_reports_framing_errors() {
        let mut server = Server::new();

        // A request before `initialize` is rejected with the spec's code.
        assert!(matches!(
            server.run(json!({"id":1,"method":"textDocument/hover","params":{}})),
            Flow::Continue
        ));
        assert!(server.output().contains("-32002"));

        server.run(json!({"id":2,"method":"initialize","params":{}}));
        assert!(server.initialized);

        // Opening a document publishes diagnostics and stores it.
        let uri = "file:///tmp/handle-test.rs";
        server.run(
            json!({"method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"version":1,"text":"fn f(){rsx!{<div class=\"flex-colum\"/>};}"}}}),
        );
        assert!(server.docs.contains_key(uri));
        assert!(server.output().contains("flex-colum"));

        // didChange keeps the resolved mode and uses the last content change.
        let before = server.docs[uri].mode.scale;
        server.run(
            json!({"method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":"ignored"},{"text":"fn f(){rsx!{<div class=\"flex\"/>};}"}]}}),
        );
        assert!(server.docs[uri].source.contains("class=\"flex\""));
        assert_eq!(server.docs[uri].mode.scale, before);

        // didClose clears diagnostics and forgets the document.
        server.run(json!({"method":"textDocument/didClose","params":{"textDocument":{"uri":uri}}}));
        assert!(!server.docs.contains_key(uri));

        // A malformed body is answered with -32700 instead of stopping the loop.
        server.handle(Err("bad json".into()));
        assert!(server.output().contains("-32700"));

        // `shutdown` is answered, then any further request is refused.
        server.run(json!({"id":3,"method":"shutdown","params":{}}));
        assert!(server.shutdown);
        server.run(json!({"id":4,"method":"textDocument/hover","params":{}}));
        assert!(server.output().contains("-32600"));

        // `exit` after `shutdown` succeeds; before it, it is an error.
        assert!(matches!(
            server.run(json!({"method":"exit"})),
            Flow::Exit(Ok(()))
        ));
        let mut fresh = Server::new();
        assert!(matches!(
            fresh.run(json!({"method":"exit"})),
            Flow::Exit(Err(_))
        ));
    }

    #[test]
    fn handle_refreshes_watched_files_and_ignores_unknown_notifications() {
        let mut server = Server::new();
        server.run(json!({"id":1,"method":"initialize","params":{}}));
        let uri = "file:///tmp/watched.rs";
        server.run(
            json!({"method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"version":1,"text":"fn f(){rsx!{<div class=\"flex-colum\"/>};}"}}}),
        );
        let before = server.output().len();

        // Re-resolving watched files republishes diagnostics for every open document.
        server.run(json!({"method":"workspace/didChangeWatchedFiles","params":{"changes":[]}}));
        let published = &server.output()[before..];
        assert!(
            published.contains("textDocument/publishDiagnostics"),
            "{published}"
        );
        assert!(published.contains("flex-colum"), "{published}");
        assert!(server.docs.contains_key(uri));

        // Unknown notifications (no id) are silently dropped.
        let before = server.output().len();
        server.run(json!({"method":"$/cancelRequest","params":{"id":1}}));
        assert_eq!(server.output().len(), before);
    }

    #[test]
    fn handle_maps_request_failures_to_jsonrpc_codes() {
        let mut server = Server::new();
        server.run(json!({"id":1,"method":"initialize","params":{}}));
        let uri = "file:///tmp/codes.rs";
        // Not a token stream, so formatting cannot run.
        server.run(
            json!({"method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"version":1,"text":"fn f( { rsx!{<div/>}; }"}}}),
        );
        server.run(json!({"id":2,"method":"textDocument/formatting","params":{"textDocument":{"uri":uri}}}));
        let response = server.output();
        assert!(response.contains("-32603"), "{response}");

        server.run(
            json!({"id":3,"method":"textDocument/rename","params":{"textDocument":{"uri":uri}}}),
        );
        assert!(server.output().contains("-32601"), "{}", server.output());
    }

    #[test]
    fn requests_without_a_document_are_answered_with_empty_results() {
        let docs = HashMap::new();
        let params = json!({"textDocument":{"uri":"file:///tmp/missing.rs"},"position":{"line":0,"character":0}});

        let completion = request("textDocument/completion", &params, &docs).unwrap();
        assert_eq!(completion["items"].as_array().unwrap().len(), 0);

        assert!(
            request("textDocument/hover", &params, &docs)
                .unwrap()
                .is_null()
        );
        assert!(
            request("textDocument/documentColor", &params, &docs)
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(
            request("textDocument/codeAction", &params, &docs)
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );

        // Unsupported methods are reported so `handle` can map them to -32601.
        let error = request("textDocument/rename", &params, &docs).unwrap_err();
        assert!(error.starts_with("unsupported method"), "{error}");
    }

    #[test]
    fn color_presentation_defaults_to_a_background_class() {
        // With no recognizable prefix in the selection, `bg-` is used.
        let docs = HashMap::new();
        let params = json!({
            "textDocument":{"uri":"file:///tmp/color.rs"},
            "color":{"red":1.0,"green":0.5,"blue":0.0,"alpha":0.25},
            "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}}
        });
        let response = request("textDocument/colorPresentation", &params, &docs).unwrap();
        assert_eq!(response[0]["label"], "bg-[#ff800040]");

        // Out-of-range channels are clamped rather than wrapped.
        let params = json!({
            "textDocument":{"uri":"file:///tmp/color.rs"},
            "color":{"red":2.0,"green":-1.0,"blue":0.5,"alpha":1.0},
            "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}}
        });
        let response = request("textDocument/colorPresentation", &params, &docs).unwrap();
        assert_eq!(response[0]["label"], "bg-[#ff0080ff]");
    }

    #[test]
    fn color_helper_reports_every_channel() {
        let (r, g, b, a) = color("red-500").unwrap();
        assert_eq!(
            (r * 255.0).round() as u8,
            0xef,
            "palette color must match the macro"
        );
        assert_eq!((g * 255.0).round() as u8, 0x44);
        assert_eq!((b * 255.0).round() as u8, 0x44);
        assert_eq!(a, 1.0);

        let (_, _, _, alpha) = color("[#11223344]").unwrap();
        assert!((alpha * 255.0).round() as u8 == 0x44);
        assert_eq!(color("not-a-color"), None);
    }

    #[test]
    fn did_change_without_an_open_document_still_loads_it() {
        let mut server = Server::new();
        server.run(json!({"id":1,"method":"initialize","params":{}}));
        let uri = "file:///tmp/unopened.rs";

        // An editor may send changes for a file it never announced; the server must cope.
        server.run(
            json!({"method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":3},"contentChanges":[{"text":"fn f(){rsx!{<div class=\"flex-colum\"/>};}"}]}}),
        );
        assert!(server.docs.contains_key(uri));
        assert!(server.output().contains("flex-colum"));

        // didOpen without a `text` field is ignored rather than clearing the document.
        server.run(json!({"method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"version":4}}}));
        assert!(server.docs[uri].source.contains("flex-colum"));
    }
}
