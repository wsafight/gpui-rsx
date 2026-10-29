//! Request handlers: completion, hover, formatting, colors, semantic tokens and code actions.
use super::document::{Document, class_at, mode_at};
use super::protocol::{offset, range};
use gpui_rsx_core::{ClassMode, ClassOutcome, analyze_class, known_classes};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::OnceLock};

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
    if let Some(theme) = mode.theme() {
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

/// A request the server does not implement, answered with JSON-RPC's -32601.
#[derive(Debug)]
pub(super) struct Unsupported;

/// Answers a request. Every supported method degrades to an empty result instead of
/// failing, since editors surface request errors to users.
pub(super) fn request(
    method: &str,
    params: &Value,
    docs: &HashMap<String, Document>,
) -> Result<Value, Unsupported> {
    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
    let document = docs.get(uri);
    let source = document.map_or("", |document| document.source.as_str());
    let cursor = offset(source, &params["position"]);
    match method {
        // Formatting is opt-in: advertising it makes editors offer this server as the Rust
        // formatter, and it only formats RSX bodies, never the surrounding Rust.
        "initialize" => Ok(json!({"capabilities":{
            "positionEncoding":"utf-16", "textDocumentSync":{"openClose":true,"change":2},
            "completionProvider":{"triggerCharacters":["\"",":","-"," ","<"]},
            "hoverProvider":true,"colorProvider":true,"codeActionProvider":true,
            "documentFormattingProvider":params["initializationOptions"]["formatting"] == true,
            "semanticTokensProvider":{"legend":{"tokenTypes":["rsxClass","rsxThemeToken","rsxInvalidClass"],"tokenModifiers":[]},"full":true}
        },"serverInfo":{"name":"gpui-rsx","version":env!("CARGO_PKG_VERSION")}})),
        "shutdown" => Ok(Value::Null),
        "textDocument/completion" => {
            let Some(document) = document else {
                return Ok(json!({"isIncomplete":false,"items":[]}));
            };
            let Some((start, _, word)) = class_at(document, cursor) else {
                return Ok(super::markup::complete(document, cursor));
            };
            let prefix = &source[start..cursor];
            let class_mode = mode_at(Some(document), cursor, uri);
            let mut matches = Vec::with_capacity(201);
            for class in completion_candidates(prefix, class_mode.clone()) {
                if class.ends_with(':')
                    || matches!(
                        analyze_class(&class, class_mode.clone()).outcome,
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
                json!({"isIncomplete":incomplete,"items":matches.into_iter().map(|c|json!({"label":c,"kind":12,"textEdit":{"range":document.range(start,start+word.len()),"newText":c}})).collect::<Vec<_>>()}),
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
                    _ => "class analysis returned an unknown outcome".to_owned(),
                };
                Ok(
                    json!({"contents":{"kind":"markdown","value":format!("```rust\n{tokens}\n```")},"range":document.range(start,end)}),
                )
            } else if let Some(document) = document
                && let Some(call) = gpui_rsx_tools::call_at(&document.analysis.calls, cursor)
            {
                // Code being edited often fails to expand; diagnostics already report that.
                Ok(gpui_rsx_tools::expand_summary(
                    &source[call.start..call.end],
                    gpui_rsx_tools::mode_for(&call.name, document.mode.clone()),
                )
                .map_or(Value::Null, |expansion| {
                    json!({"contents":{"kind":"markdown","value":format!("```rust\n{expansion}\n```")}})
                }))
            } else {
                Ok(Value::Null)
            }
        }
        "textDocument/formatting" => {
            // Unparsable Rust is common mid-edit; answering `null` keeps format-on-save quiet.
            let Ok(result) = gpui_rsx_tools::format::rust(source) else {
                return Ok(Value::Null);
            };
            if result == source {
                return Ok(json!([]));
            }
            Ok(json!([{"range":range(source,0,source.len()),"newText":result}]))
        }
        "textDocument/documentColor" => {
            let mut colors = Vec::new();
            if let Some(document) = document {
                for token in &document.analysis.class_tokens {
                    if let Some(rgba) = token.rgba {
                        let (red, green, blue, alpha) = rgba_channels(rgba);
                        colors.push(json!({"range":document.range(token.start,token.end),"color":{"red":red,"green":green,"blue":blue,"alpha":alpha}}));
                    }
                }
            }
            Ok(json!(colors))
        }
        "textDocument/semanticTokens/full" => Ok(json!({
            "data": semantic_tokens(document, source),
        })),
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
            // Only fixes touching the requested range; a missing range means the whole file.
            let requested = params.get("range").map(|requested| {
                (
                    offset(source, &requested["start"]),
                    offset(source, &requested["end"]),
                )
            });
            let mut actions = Vec::new();
            let Some(document) = document else {
                return Ok(json!(actions));
            };
            for d in &document.analysis.diagnostics {
                if requested.is_some_and(|(start, end)| d.end < start || d.start > end) {
                    continue;
                }
                if let Some(suggestion) = d.suggestion.clone() {
                    let fix_range = document.range(d.start, d.end);
                    let mut changes = serde_json::Map::new();
                    changes.insert(
                        uri.into(),
                        json!([{"range":fix_range,"newText":suggestion}]),
                    );
                    actions.push(json!({
                        "title":format!("Replace with {suggestion}"),"kind":"quickfix","isPreferred":true,
                        "diagnostics":[{"range":fix_range,"source":"gpui-rsx","message":d.message}],
                        "edit":{"changes":changes}
                    }));
                }
            }
            Ok(json!(actions))
        }
        _ => Err(Unsupported),
    }
}
#[cfg(test)]
fn color(value: &str) -> Option<(f64, f64, f64, f64)> {
    gpui_rsx_core::codegen::class::color_rgba(value).map(rgba_channels)
}
fn rgba_channels(rgba: u32) -> (f64, f64, f64, f64) {
    (
        (rgba >> 24) as f64 / 255.0,
        ((rgba >> 16) & 255) as f64 / 255.0,
        ((rgba >> 8) & 255) as f64 / 255.0,
        (rgba & 255) as f64 / 255.0,
    )
}

fn semantic_tokens(document: Option<&Document>, _source: &str) -> Vec<usize> {
    let Some(document) = document else {
        return Vec::new();
    };
    let mut tokens = Vec::new();
    for token in &document.analysis.class_tokens {
        let token_type = match token.kind {
            gpui_rsx_tools::ClassTokenKind::Supported => 0,
            gpui_rsx_tools::ClassTokenKind::Theme => 1,
            gpui_rsx_tools::ClassTokenKind::Invalid => 2,
        };
        let token_start = document.position(token.start);
        let token_end = document.position(token.end);
        tokens.push((
            token_start["line"].as_u64().unwrap_or(0) as usize,
            token_start["character"].as_u64().unwrap_or(0) as usize,
            token_end["character"].as_u64().unwrap_or(0) as usize
                - token_start["character"].as_u64().unwrap_or(0) as usize,
            token_type,
        ));
    }
    tokens.sort_unstable_by_key(|(line, character, _, _)| (*line, *character));
    let mut previous_line = 0;
    let mut previous_character = 0;
    let mut data = Vec::with_capacity(tokens.len() * 5);
    for (line, character, length, token_type) in tokens {
        data.extend([
            line - previous_line,
            if line == previous_line {
                character - previous_character
            } else {
                character
            },
            length,
            token_type,
            0,
        ]);
        previous_line = line;
        previous_character = character;
    }
    data
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::document::diagnostics;
    use crate::lsp::protocol::position;
    use gpui_rsx_core::{Theme, theme::TokenValue};

    #[test]
    fn semantic_tokens_use_macro_modes_theme_values_and_utf16_positions() {
        let source = "fn f(){let s=\"😀\";rsx_strict!{<div class=\"flex bg-brand nonsense\"/>};}";
        let theme = Theme::parse("[colors]\nbrand=\"#123456\"\n").unwrap();
        let document =
            Document::with_mode(source, ClassMode::PERMISSIVE.with_theme(Some(theme.into())));
        let data = semantic_tokens(Some(&document), source);
        assert_eq!(data.len(), 15);
        assert_eq!(
            data[1],
            position(source, source.find("flex").unwrap())["character"]
                .as_u64()
                .unwrap() as usize
        );
        assert_eq!(data[3], 0);
        assert_eq!(data[8], 1);
        assert_eq!(data[13], 2);
        assert_eq!(data[7], "bg-brand".len());
    }

    fn completion_response(
        source: &str,
        cursor: usize,
        theme: Option<std::sync::Arc<Theme>>,
    ) -> Value {
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
        static THEME: OnceLock<std::sync::Arc<Theme>> = OnceLock::new();
        let theme = THEME.get_or_init(|| {
            Theme::builder()
                .color(
                    "accent",
                    TokenValue::Color {
                        hex: 0x123456,
                        alpha: false,
                    },
                )
                .build()
                .into()
        });
        let source = "fn f(){rsx!{<div class=\"bg-acc\"/>}}";
        let prefix = "bg-acc";
        let cursor = source.find(prefix).unwrap() + prefix.len();
        let response = completion_response(source, cursor, Some(std::sync::Arc::clone(theme)));

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
        static THEME: OnceLock<std::sync::Arc<Theme>> = OnceLock::new();
        let theme = THEME.get_or_init(|| {
            Theme::builder()
                .color(
                    "accent",
                    TokenValue::Color {
                        hex: 0x123456,
                        alpha: false,
                    },
                )
                .build()
                .into()
        });
        let source = "fn f(){rsx!{<div class=\"hover:bg-acc\"/>}}";
        let prefix = "hover:bg-acc";
        let cursor = source.find(prefix).unwrap() + prefix.len();
        let response = completion_response(source, cursor, Some(std::sync::Arc::clone(theme)));
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
    fn formatting_is_only_advertised_when_enabled() {
        let docs = HashMap::new();
        let capability = |params: Value| {
            request("initialize", &params, &docs).unwrap()["capabilities"]
                ["documentFormattingProvider"]
                .clone()
        };
        assert_eq!(capability(json!({})), json!(false));
        assert_eq!(
            capability(json!({"initializationOptions":{"formatting":true}})),
            json!(true)
        );
    }

    #[test]
    fn formatting_returns_a_full_document_replacement() {
        let source = "fn f(){rsx!{<div class=\"flex\"><span/></div>};}";
        let (response, _) = request_at("textDocument/formatting", source, 0, json!({}));
        let edit = &response[0];
        let formatted = edit["newText"].as_str().unwrap().to_owned();
        // Already formatted input yields no edits, and broken input yields null, not an error.
        let (unchanged, _) = request_at("textDocument/formatting", &formatted, 0, json!({}));
        assert_eq!(unchanged, json!([]));
        let (broken, _) = request_at("textDocument/formatting", "fn f( {", 0, json!({}));
        assert!(broken.is_null());
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
    fn code_actions_only_cover_the_requested_range() {
        let source = "fn f(){rsx!{<div class=\"flex-colum\"/>};\nrsx!{<p class=\"flex-colun\"/>};}";
        let second = source.find("flex-colun\"").unwrap();
        for (cursor, expected) in [(second + 2, "Replace with flex-col"), (2, "")] {
            let at = position(source, cursor);
            let (response, _) = request_at(
                "textDocument/codeAction",
                source,
                cursor,
                json!({"range":{"start":at,"end":at}}),
            );
            let titles: Vec<_> = response
                .as_array()
                .unwrap()
                .iter()
                .map(|action| action["title"].as_str().unwrap())
                .collect();
            if expected.is_empty() {
                assert!(titles.is_empty(), "{titles:?}");
            } else {
                assert_eq!(titles, [expected]);
                assert_eq!(response[0]["diagnostics"][0]["source"], "gpui-rsx");
                assert_eq!(
                    response[0]["edit"]["changes"]["file:///tmp/rsx-lsp-test.rs"][0]["range"]["start"],
                    position(source, second)
                );
            }
        }
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
    fn nested_macros_use_their_own_mode_for_hover_and_completion() {
        let source = "fn f(){rsx!{<div>{if a { rsx_strict!{<span class=\"nonsense fle\"/>} } else { rsx!{<b/>} }}</div>};}";
        let strict = source.find("nonsense").unwrap();
        let (response, _) = request_at("textDocument/hover", source, strict + 1, json!({}));
        assert!(
            response["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("strict mode")
        );
        let cursor = source.find("fle").unwrap() + 3;
        let (response, _) = request_at("textDocument/completion", source, cursor, json!({}));
        assert!(labels(&response).contains(&"flex"));
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
        assert!(request("textDocument/rename", &params, &docs).is_err());
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
}
