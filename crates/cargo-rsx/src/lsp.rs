//! A synchronous stdio LSP server. Full document sync keeps editing deterministic.
use gpui_rsx_core::{ClassMode, ClassOutcome, analyze_class, known_classes};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
    path::PathBuf,
};

fn send(out: &mut impl Write, message: Value) -> Result<(), String> {
    let body = message.to_string();
    write!(out, "Content-Length: {}\r\n\r\n{body}", body.len())
        .and_then(|()| out.flush())
        .map_err(|e| e.to_string())
}
fn read(input: &mut impl BufRead) -> Result<Option<Value>, String> {
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
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|e| e.to_string())
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
fn mode_at(source: &str, cursor: usize, uri: &str) -> ClassMode {
    let base = mode(uri);
    gpui_rsx_tools::macro_calls(source)
        .ok()
        .and_then(|calls| {
            calls
                .into_iter()
                .find(|call| cursor >= call.start && cursor <= call.end)
        })
        .map(|call| gpui_rsx_tools::mode_for(&call.name, base))
        .unwrap_or(base)
}

fn diagnostics(source: &str, uri: &str) -> Vec<Value> {
    gpui_rsx_tools::analyze_rust(source, mode(uri)).into_iter().map(|d| json!({
        "range":range(source,d.start,d.end),"severity": match d.severity.as_str() {"error"=>1,"warning"=>2,_=>3},
        "source":"gpui-rsx","message":d.message,"data":{"suggestion":d.suggestion}
    })).collect()
}

/// A class token under the cursor; refuses text outside an RSX macro or class literal.
fn class_at(source: &str, cursor: usize) -> Option<(usize, usize, String)> {
    let calls = gpui_rsx_tools::macro_calls(source).ok()?;
    let call = calls
        .iter()
        .find(|c| cursor >= c.start && cursor <= c.end)?;
    let raw = &source[call.start..call.end];
    let body = syn_body_literals(raw)?;
    for (start, end) in body {
        let start = call.start + start;
        let end = call.start + end;
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
fn syn_body_literals(source: &str) -> Option<Vec<(usize, usize)>> {
    gpui_rsx_tools::class_literal_ranges(source).ok()
}

fn request(method: &str, params: &Value, docs: &HashMap<String, String>) -> Result<Value, String> {
    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
    let source = docs.get(uri).map(String::as_str).unwrap_or("");
    let cursor = offset(source, &params["position"]);
    match method {
        "initialize" => Ok(json!({"capabilities":{
            "positionEncoding":"utf-16", "textDocumentSync":{"openClose":true,"change":1},
            "completionProvider":{"triggerCharacters":["\"",":","-"," "]},
            "hoverProvider":true,"documentFormattingProvider":true,"colorProvider":true,"codeActionProvider":true
        },"serverInfo":{"name":"gpui-rsx","version":env!("CARGO_PKG_VERSION")}})),
        "shutdown" => Ok(Value::Null),
        "textDocument/completion" => {
            let Some((start, _, word)) = class_at(source, cursor) else {
                return Ok(json!({"isIncomplete":false,"items":[]}));
            };
            let prefix = &source[start..cursor];
            let mut classes = known_classes().to_vec();
            if let Some(theme) = mode(uri).theme {
                classes.extend(theme.class_names());
            }
            classes.extend(
                gpui_rsx_core::SUPPORTED_VARIANTS
                    .iter()
                    .map(|v| format!("{v}:")),
            );
            let variant = prefix.rsplit_once(':').map(|(v, _)| format!("{v}:"));
            if let Some(variant) = variant {
                classes = classes
                    .into_iter()
                    .map(|c| format!("{variant}{c}"))
                    .collect();
            }
            classes.sort();
            classes.dedup();
            let matches: Vec<_> = classes
                .into_iter()
                .filter(|c| {
                    c.starts_with(prefix)
                        && (c.ends_with(':')
                            || matches!(
                                analyze_class(c, mode_at(source, cursor, uri)).outcome,
                                ClassOutcome::Supported(_)
                            ))
                })
                .collect();
            let incomplete = matches.len() > 200;
            Ok(
                json!({"isIncomplete":incomplete,"items":matches.into_iter().take(200).map(|c|json!({"label":c,"kind":12,"textEdit":{"range":range(source,start,start+word.len()),"newText":c}})).collect::<Vec<_>>()}),
            )
        }
        "textDocument/hover" => {
            if let Some((start, end, class)) = class_at(source, cursor) {
                let tokens = match analyze_class(&class, mode_at(source, cursor, uri)).outcome {
                    ClassOutcome::Supported(t) | ClassOutcome::PassThrough { tokens: t, .. } => {
                        t.to_string()
                    }
                    ClassOutcome::Ignored { message } | ClassOutcome::Error { message } => message,
                };
                Ok(
                    json!({"contents":{"kind":"markdown","value":format!("```rust\n{tokens}\n```")},"range":range(source,start,end)}),
                )
            } else if let Ok(calls) = gpui_rsx_tools::macro_calls(source)
                && let Some(call) = calls.iter().find(|c| cursor >= c.start && cursor <= c.end)
            {
                let expansion = gpui_rsx_tools::expand(
                    &source[call.start..call.end],
                    gpui_rsx_tools::mode_for(&call.name, mode(uri)),
                )?;
                Ok(
                    json!({"contents":{"kind":"markdown","value":format!("```rust\n{expansion}\n```")}}),
                )
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
            if let Ok(calls) = gpui_rsx_tools::macro_calls(source) {
                for call in calls {
                    let raw = &source[call.start..call.end];
                    for (start, end) in syn_body_literals(raw).unwrap_or_default() {
                        let value = &raw[start..end];
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
                                colors.push(json!({"range":range(source,call.start+start+begin,call.start+start+at),"color":{"red":red,"green":green,"blue":blue,"alpha":alpha}}));
                            }
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
            for d in gpui_rsx_tools::analyze_rust(source, mode(uri)) {
                if let Some(suggestion) = d.suggestion {
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
    let rgba = if let Some(c) = gpui_rsx_core::codegen::tables::lookup_color(value) {
        (c << 8) | 255
    } else {
        let hex = value.strip_prefix("[#")?.strip_suffix(']')?;
        match hex.len() {
            3 => {
                let chars: Vec<_> = hex.chars().collect();
                u32::from_str_radix(
                    &format!("{0}{0}{1}{1}{2}{2}ff", chars[0], chars[1], chars[2]),
                    16,
                )
                .ok()?
            }
            6 => (u32::from_str_radix(hex, 16).ok()? << 8) | 255,
            8 => u32::from_str_radix(hex, 16).ok()?,
            _ => return None,
        }
    };
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
    let mut docs = HashMap::new();
    let mut shutdown = false;
    while let Some(message) = read(&mut input)? {
        let method = message["method"].as_str().unwrap_or("");
        let params = &message["params"];
        if method == "exit" {
            return if shutdown {
                Ok(())
            } else {
                Err("exit before shutdown".into())
            };
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
                docs.insert(uri.clone(), text.to_owned());
                send(
                    &mut out,
                    json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"version":doc["version"],"diagnostics":diagnostics(text,&uri)}}),
                )?;
            }
        } else if method == "textDocument/didClose" {
            let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
            docs.remove(uri);
            send(
                &mut out,
                json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"diagnostics":[]}}),
            )?;
        } else if let Some(id) = message.get("id") {
            if method == "shutdown" {
                shutdown = true;
            }
            let response = match request(method, params, &docs) {
                Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":result}),
                Err(error) => {
                    json!({"jsonrpc":"2.0","id":id,"error":{"code":if error.starts_with("unsupported method") {-32601} else {-32603},"message":error}})
                }
            };
            send(&mut out, response)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn utf16_positions_roundtrip() {
        let source = "// 中文😀\nlet x = 1;";
        for (i, _) in source.char_indices() {
            assert_eq!(offset(source, &position(source, i)), i);
        }
    }
}
