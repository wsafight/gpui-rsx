//! Open-document state and the analysis queries shared by every feature.
use super::protocol::{LineIndex, uri_path};
use gpui_rsx_core::ClassMode;
use gpui_rsx_tools::DiagnosticSeverity;
use serde_json::{Value, json};

pub(super) struct Document {
    pub(super) source: String,
    pub(super) mode: ClassMode,
    /// Macro calls, absolute byte ranges of static class contents, and diagnostics.
    pub(super) analysis: gpui_rsx_tools::RustAnalysis,
    pub(super) lines: LineIndex,
    pub(super) config_error: Option<String>,
    pub(super) version: Option<i64>,
}

impl Document {
    pub(super) fn new(uri: &str, source: &str) -> Self {
        let path = uri_path(uri);
        match crate::config::mode(Default::default(), path.as_deref()) {
            Ok(mode) => Self::with_mode(source, mode),
            Err(error) => Self::with_mode_and_error(source, ClassMode::PERMISSIVE, Some(error)),
        }
    }

    /// Reuses an already resolved mode; resolving it walks the filesystem.
    pub(super) fn with_mode(source: &str, mode: ClassMode) -> Self {
        Self::with_mode_and_error(source, mode, None)
    }

    fn with_mode_and_error(source: &str, mode: ClassMode, config_error: Option<String>) -> Self {
        Self {
            analysis: gpui_rsx_tools::analyze_rust_document(source, mode.clone()),
            lines: LineIndex::new(source),
            source: source.to_owned(),
            mode,
            config_error,
            version: None,
        }
    }

    pub(super) fn range(&self, start: usize, end: usize) -> Value {
        self.lines.range(&self.source, start, end)
    }

    pub(super) fn position(&self, offset: usize) -> Value {
        self.lines.position(&self.source, offset)
    }

    pub(super) fn updated(&self, source: String) -> Self {
        let analysis = gpui_rsx_tools::update_rust_document(
            &source,
            self.mode.clone(),
            &self.source,
            &self.analysis,
        );
        Self {
            analysis,
            lines: LineIndex::new(&source),
            source,
            mode: self.mode.clone(),
            config_error: self.config_error.clone(),
            version: self.version,
        }
    }
}

fn mode(uri: &str) -> ClassMode {
    let path = uri_path(uri);
    crate::config::mode(Default::default(), path.as_deref()).unwrap_or(ClassMode::PERMISSIVE)
}
pub(super) fn mode_at(document: Option<&Document>, cursor: usize, uri: &str) -> ClassMode {
    let base = document.map_or_else(|| mode(uri), |document| document.mode.clone());
    document
        .and_then(|document| gpui_rsx_tools::call_at(&document.analysis.calls, cursor))
        .map(|call| gpui_rsx_tools::mode_for(&call.name, base.clone()))
        .unwrap_or(base)
}

pub(super) fn diagnostics(document: &Document) -> Vec<Value> {
    let mut diagnostics = document.analysis.diagnostics.iter().map(|d| json!({
        "range":document.range(d.start,d.end),"severity": match d.severity {DiagnosticSeverity::Error=>1,DiagnosticSeverity::Warning=>2,_=>3},
        "source":"gpui-rsx","message":d.message,"data":{"suggestion":d.suggestion}
    })).collect::<Vec<_>>();
    if let Some(error) = &document.config_error {
        diagnostics.push(json!({
            "range": document.range(0, 0),
            "severity": 1,
            "source": "gpui-rsx",
            "message": format!("Theme configuration error: {error}"),
        }));
    }
    diagnostics
}

/// A class token under the cursor; refuses text outside an RSX macro or class literal.
pub(super) fn class_at(document: &Document, cursor: usize) -> Option<(usize, usize, String)> {
    let source = &document.source;
    gpui_rsx_tools::call_at(&document.analysis.calls, cursor)?;
    for &(start, end) in &document.analysis.literals {
        if cursor < start || cursor > end {
            continue;
        }
        let before = &source[start..cursor];
        let after = &source[cursor..end];
        // The compiler splits classes on ASCII whitespace. A Unicode separator is part of
        // the token; advancing one byte past it would slice inside its UTF-8 encoding.
        let word_start = before
            .rfind(|c: char| c.is_ascii_whitespace())
            .map(|i| start + i + 1)
            .unwrap_or(start);
        let word_end = after
            .find(|c: char| c.is_ascii_whitespace())
            .map(|i| cursor + i)
            .unwrap_or(end);
        return Some((word_start, word_end, source[word_start..word_end].into()));
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_ascii_class_whitespace_keeps_valid_utf8_ranges() {
        for separator in ['\u{a0}', '\u{2003}', '\u{3000}'] {
            let classes = format!("flex{separator}bg-red-500");
            let source = format!("fn f(){{rsx!{{<div class=\"{classes}\"/>}}}}");
            let document = Document::with_mode(&source, ClassMode::PERMISSIVE);
            let start = source.find(&classes).unwrap();
            assert_eq!(
                class_at(&document, source.find("red").unwrap()),
                Some((start, start + classes.len(), classes)),
            );
        }
    }

    #[test]
    fn invalid_theme_config_is_reported_as_a_document_diagnostic() {
        let root = std::env::temp_dir().join(format!("rsx-lsp-theme-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname=\"lsp-theme-test\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
        )
        .unwrap();
        std::fs::write(root.join("rsx.toml"), "[opacity]\nmuted = \"120%\"\n").unwrap();
        let uri = format!("file://{}", root.join("src/main.rs").display());
        let document = Document::new(&uri, "fn f(){rsx!{<div/>}}\n");
        let values = diagnostics(&document);
        assert!(values.iter().any(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("Theme configuration error"))
        }));
        std::fs::remove_dir_all(root).unwrap();
    }
}
