//! JSON-RPC framing and conversions between LSP positions and byte offsets.
use serde_json::{Value, json};
use std::{
    io::{BufRead, Write},
    path::PathBuf,
};

pub(super) fn send(out: &mut impl Write, message: Value) -> Result<(), String> {
    let body = message.to_string();
    write!(out, "Content-Length: {}\r\n\r\n{body}", body.len())
        .and_then(|()| out.flush())
        .map_err(|e| e.to_string())
}
/// Outer `Err`: the stream is unusable. Inner `Err`: one malformed body, which is skippable
/// because framing is still intact.
pub(super) fn read(input: &mut impl BufRead) -> Result<Option<Result<Value, String>>, String> {
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
/// Line start offsets, built once per document version so converting an offset to an LSP
/// position only scans the line it is on rather than the whole file.
pub(super) struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub(super) fn new(source: &str) -> Self {
        let starts = std::iter::once(0)
            .chain(source.match_indices('\n').map(|(i, _)| i + 1))
            .collect();
        Self { starts }
    }

    /// Same result as [`position`], for the source this index was built from.
    pub(super) fn position(&self, source: &str, offset: usize) -> Value {
        let mut offset = offset.min(source.len());
        while !source.is_char_boundary(offset) {
            offset -= 1;
        }
        let line = self.starts.partition_point(|&start| start <= offset) - 1;
        let character = source[self.starts[line]..offset].encode_utf16().count();
        json!({"line": line, "character": character})
    }

    pub(super) fn range(&self, source: &str, start: usize, end: usize) -> Value {
        json!({"start":self.position(source,start),"end":self.position(source,end)})
    }
}

pub(super) fn position(source: &str, offset: usize) -> Value {
    let mut offset = offset.min(source.len());
    while !source.is_char_boundary(offset) {
        offset -= 1;
    }
    let before = &source[..offset];
    json!({"line": before.bytes().filter(|&b| b == b'\n').count(), "character": before.rsplit('\n').next().unwrap_or("").encode_utf16().count()})
}
pub(super) fn offset(source: &str, pos: &Value) -> usize {
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
pub(super) fn range(source: &str, start: usize, end: usize) -> Value {
    json!({"start":position(source,start),"end":position(source,end)})
}
pub(super) fn uri_path(uri: &str) -> Option<PathBuf> {
    let raw = uri.strip_prefix("file://")?;
    let mut bytes = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        if raw.as_bytes()[i] == b'%' {
            bytes.push(u8::from_str_radix(raw.get(i + 1..i + 3)?, 16).ok()?);
            i += 3;
        } else {
            bytes.push(raw.as_bytes()[i]);
            i += 1;
        }
    }
    let decoded = String::from_utf8(bytes).ok()?;
    // Only a Windows drive URI has an extra slash (`file:///C:/...`). Rooted
    // paths such as `file:///tmp/...` must keep theirs on every platform.
    let path = if cfg!(windows)
        && decoded.as_bytes().get(..4).is_some_and(|prefix| {
            prefix[0] == b'/'
                && prefix[1].is_ascii_alphabetic()
                && prefix[2] == b':'
                && prefix[3] == b'/'
        }) {
        &decoded[1..]
    } else {
        &decoded
    };
    Some(path.into())
}

pub(super) fn apply_content_changes(source: &str, changes: &[Value]) -> Result<String, String> {
    let mut updated = source.to_owned();
    for change in changes {
        let text = change["text"]
            .as_str()
            .ok_or("content change is missing text")?;
        let Some(range) = change.get("range") else {
            updated.clear();
            updated.push_str(text);
            continue;
        };
        let start = edit_offset(&updated, &range["start"])?;
        let end = edit_offset(&updated, &range["end"])?;
        if start > end {
            return Err("content change range is invalid".into());
        }
        updated.replace_range(start..end, text);
    }
    Ok(updated)
}

fn edit_offset(source: &str, pos: &Value) -> Result<usize, String> {
    let line = pos["line"]
        .as_u64()
        .ok_or("edit line must be a nonnegative integer")?;
    let column = pos["character"]
        .as_u64()
        .ok_or("edit character must be a nonnegative integer")?;
    let mut start = 0;
    for _ in 0..line {
        start += source[start..]
            .find('\n')
            .ok_or("edit line is out of bounds")?
            + 1;
    }
    let mut units = 0;
    for (index, c) in source[start..].char_indices() {
        if units == column {
            return Ok(start + index);
        }
        if c == '\n' || (c == '\r' && source[start + index..].starts_with("\r\n")) {
            break;
        }
        units += c.len_utf16() as u64;
        if units > column {
            return Err("edit splits a UTF-16 surrogate pair".into());
        }
    }
    if units == column {
        Ok(source.len())
    } else {
        Err("edit character is out of bounds".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_index_matches_full_scans() {
        for source in ["", "a", "\n", "中文😀\r\nx\n\n😀y", "fn f() {}\n"] {
            let index = LineIndex::new(source);
            for offset in 0..=source.len() + 2 {
                assert_eq!(
                    index.position(source, offset),
                    position(source, offset),
                    "{source:?} @ {offset}"
                );
            }
        }
    }

    #[test]
    fn utf16_positions_roundtrip() {
        let source = "// 中文😀\nlet x = 1;";
        for (i, _) in source.char_indices() {
            assert_eq!(offset(source, &position(source, i)), i);
        }
    }

    #[test]
    fn incremental_edits_apply_in_order_and_reject_invalid_utf16_ranges() {
        let source = "// 😀\r\nfn f(){rsx!{<div class=\"flex-colum\"/>};}\r\n";
        let start = source.find("flex-colum").unwrap();
        let changes = vec![
            json!({"range":range(source, start, start + 10),"text":"flex-col"}),
            json!({"range":{"start":{"line":0,"character":3},"end":{"line":0,"character":5}},"text":"中文"}),
        ];
        let updated = apply_content_changes(source, &changes).unwrap();
        assert!(updated.starts_with("// 中文\r\n"));
        assert!(updated.contains("flex-col\""));
        for pos in [
            json!({"line":0,"character":4}),
            json!({"line":9,"character":0}),
            json!({"line":0,"character":99}),
        ] {
            assert!(
                apply_content_changes(
                    source,
                    &[json!({"range":{"start":pos,"end":pos},"text":"x"})]
                )
                .is_err()
            );
        }
        assert_eq!(
            apply_content_changes(source, &[json!({"text":"replacement"})]).unwrap(),
            "replacement"
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
        assert_eq!(uri_path("file:///tmp/%"), None);
        assert_eq!(uri_path("file:///tmp/%4"), None);
    }

    #[test]
    fn uri_path_strips_only_windows_drive_prefixes() {
        let expected = if cfg!(windows) {
            "C:/Users/a b/main.rs"
        } else {
            "/C:/Users/a b/main.rs"
        };
        for uri in [
            "file:///C:/Users/a%20b/main.rs",
            "file:///%43%3A/Users/a%20b/main.rs",
        ] {
            assert_eq!(uri_path(uri), Some(PathBuf::from(expected)), "{uri}");
        }
        assert_eq!(
            uri_path("file:///Code/main.rs"),
            Some(PathBuf::from("/Code/main.rs"))
        );
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
}
