//! Shared, host-independent tooling. Positions are UTF-8 byte offsets into the input.
#[cfg(feature = "convert")]
pub mod convert;
pub mod format;
pub mod preview;

use gpui_rsx_core::codegen::attribute::static_class_literals;
use gpui_rsx_core::syntax::{RsxAttribute, RsxBody, RsxElement, RsxNode};
use gpui_rsx_core::{ClassMode, Severity, SpacingScale, class_diagnostics};
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use serde::{Deserialize, Serialize};
use syn::{Expr, Lit, spanned::Spanned};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub start: usize,
    pub end: usize,
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MacroCall {
    pub name: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Default)]
pub struct RustAnalysis {
    pub calls: Vec<MacroCall>,
    pub literals: Vec<(usize, usize)>,
    pub diagnostics: Vec<Diagnostic>,
}

pub const MACROS: &[&str] = &[
    "rsx",
    "rsx_strict",
    "rsx_permissive",
    "rsx_tw",
    "rsx_expand",
];

fn is_macro_name(name: &Ident) -> bool {
    name == "rsx"
        || name == "rsx_strict"
        || name == "rsx_permissive"
        || name == "rsx_tw"
        || name == "rsx_expand"
}

/// Byte offset of `span` within `source`, which must be the string the span was parsed from.
///
/// `Span::byte_range` resolves through proc-macro2's cached char-to-byte table, so this
/// doesn't rescan `source` from the start for every literal and diagnostic.
pub fn byte_offset(source: &str, span: Span, end: bool) -> usize {
    let range = span.byte_range();
    let mut offset = if end { range.end } else { range.start }.min(source.len());
    while !source.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

pub fn macro_calls(source: &str) -> Result<Vec<MacroCall>, String> {
    fn scan(tokens: TokenStream, source: &str, calls: &mut Vec<MacroCall>) {
        let tokens: Vec<_> = tokens.into_iter().collect();
        let mut i = 0;
        while i < tokens.len() {
            if let TokenTree::Ident(name) = &tokens[i]
                && is_macro_name(name)
                && let Some(TokenTree::Punct(bang)) = tokens.get(i + 1)
                && bang.as_char() == '!'
                && let Some(TokenTree::Group(group)) = tokens.get(i + 2)
            {
                calls.push(MacroCall {
                    name: name.to_string(),
                    start: byte_offset(source, group.span_open(), true),
                    end: byte_offset(source, group.span_close(), false),
                });
                i += 3;
                continue;
            }
            if let TokenTree::Group(group) = &tokens[i] {
                scan(group.stream(), source, calls);
            }
            i += 1;
        }
    }
    let tokens = source.parse::<TokenStream>().map_err(|e| e.to_string())?;
    let mut calls = Vec::new();
    scan(tokens, source, &mut calls);
    Ok(calls)
}

pub fn mode_for(name: &str, mode: ClassMode) -> ClassMode {
    let strict = name == "rsx_strict" || (mode.is_strict() && name != "rsx_permissive");
    let scale = if name == "rsx_tw" {
        SpacingScale::Tailwind
    } else {
        mode.scale()
    };
    mode.with_strict(strict).with_scale(scale)
}

fn error(source: &str, e: syn::Error) -> Diagnostic {
    Diagnostic {
        start: byte_offset(source, e.span(), false),
        end: byte_offset(source, e.span(), true),
        severity: "error".into(),
        message: e.to_string(),
        suggestion: None,
    }
}

/// Frees the per-thread source map that `span-locations` parsing appends to on every parse.
///
/// Long-lived hosts (LSP, wasm) must call this between requests or memory grows without
/// bound until span offsets overflow `u32`. Any `Span` created before the call becomes invalid,
/// so only call it once results have been reduced to byte offsets.
pub fn release_span_memory() {
    proc_macro2::extra::invalidate_current_thread_spans();
}

pub fn analyze_rust(source: &str, mode: ClassMode) -> Vec<Diagnostic> {
    analyze_rust_document(source, mode).diagnostics
}

/// Scans a Rust document once, then parses each RSX macro body once to collect editor state.
pub fn analyze_rust_document(source: &str, mode: ClassMode) -> RustAnalysis {
    analyze_document(source, mode, None)
}

/// Reanalyzes changed macro bodies while reusing diagnostics and literal ranges for
/// unchanged bodies. The Rust token stream is rescanned to keep discovery correct
/// after edits to comments, strings, delimiters and macro names. `mode` must match
/// the mode used for `previous`; configuration changes require a full analysis.
pub fn update_rust_document(
    source: &str,
    mode: ClassMode,
    previous_source: &str,
    previous: &RustAnalysis,
) -> RustAnalysis {
    analyze_document(source, mode, Some((previous_source, previous)))
}

fn analyze_document(
    source: &str,
    mode: ClassMode,
    previous: Option<(&str, &RustAnalysis)>,
) -> RustAnalysis {
    let calls = match macro_calls(source) {
        Ok(calls) => calls,
        Err(message) => {
            return RustAnalysis {
                diagnostics: vec![Diagnostic {
                    start: 0,
                    end: source.len().min(1),
                    severity: "error".into(),
                    message,
                    suggestion: None,
                }],
                ..RustAnalysis::default()
            };
        }
    };
    let mut analysis = RustAnalysis {
        calls,
        ..RustAnalysis::default()
    };
    let mut reusable = std::collections::HashMap::new();
    if let Some((previous_source, previous)) = previous {
        for call in &previous.calls {
            if let Some(body) = previous_source.get(call.start..call.end) {
                reusable.entry((call.name.as_str(), body)).or_insert(call);
            }
        }
    }
    for call in &analysis.calls {
        let body = &source[call.start..call.end];
        if let Some(old_call) = reusable.get(&(call.name.as_str(), body))
            && let Some((_, previous)) = previous
        {
            analysis.literals.extend(
                previous
                    .literals
                    .iter()
                    .filter(|&&(start, end)| start >= old_call.start && end <= old_call.end)
                    .map(|&(start, end)| {
                        (
                            call.start + start - old_call.start,
                            call.start + end - old_call.start,
                        )
                    }),
            );
            analysis.diagnostics.extend(
                previous
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic.start >= old_call.start && diagnostic.end <= old_call.end
                    })
                    .map(|diagnostic| {
                        let mut diagnostic = diagnostic.clone();
                        diagnostic.start = call.start + diagnostic.start - old_call.start;
                        diagnostic.end = call.start + diagnostic.end - old_call.start;
                        diagnostic
                    }),
            );
            continue;
        }
        let body_analysis = analyze_body_with_ranges(body, mode_for(&call.name, mode.clone()));
        analysis.literals.extend(
            body_analysis
                .literals
                .into_iter()
                .map(|(start, end)| (call.start + start, call.start + end)),
        );
        analysis
            .diagnostics
            .extend(body_analysis.diagnostics.into_iter().map(|mut diagnostic| {
                diagnostic.start += call.start;
                diagnostic.end += call.start;
                diagnostic
            }));
    }
    analysis
}

pub fn analyze_body(source: &str, mode: ClassMode) -> Vec<Diagnostic> {
    analyze_body_with_ranges(source, mode).diagnostics
}

struct BodyAnalysis {
    literals: Vec<(usize, usize)>,
    diagnostics: Vec<Diagnostic>,
}

fn analyze_body_with_ranges(source: &str, mode: ClassMode) -> BodyAnalysis {
    let body = match syn::parse_str::<RsxBody>(source) {
        Ok(body) => body,
        Err(e) => {
            return BodyAnalysis {
                literals: Vec::new(),
                diagnostics: vec![error(source, e)],
            };
        }
    };
    let mut analysis = BodyAnalysis {
        literals: Vec::new(),
        diagnostics: Vec::new(),
    };
    walk_body(&body, &mut |el| {
        for attr in &el.attributes {
            match attr {
                RsxAttribute::Value { name, value } if name == "class" => {
                    if let Some(literals) = class_literals(attr) {
                        for lit in literals {
                            if let Some(range) =
                                lint_literal(source, lit, mode.clone(), &mut analysis.diagnostics)
                            {
                                analysis.literals.push(range);
                            }
                        }
                    } else {
                        analysis.diagnostics.push(Diagnostic { start: byte_offset(source, value.span(), false), end: byte_offset(source, value.span(), true),
                            severity: "information".into(), message: "Dynamic class expression: values are checked at runtime; variants and theme tokens require static strings.".into(), suggestion: None });
                    }
                }
                RsxAttribute::WhenClass { .. } | RsxAttribute::StateClass { .. } => {
                    for lit in class_literals(attr).into_iter().flatten() {
                        if let Some(range) =
                            lint_literal(source, lit, mode.clone(), &mut analysis.diagnostics)
                        {
                            analysis.literals.push(range);
                        }
                    }
                }
                _ => {}
            }
        }
    });
    // Include semantic errors (required constructors, loop keys, unsupported state classes).
    let expanded = gpui_rsx_core::generate_body_with_mode(&body, mode.clone());
    fn semantic_errors(tokens: TokenStream, source: &str, out: &mut Vec<Diagnostic>) {
        let tokens: Vec<_> = tokens.into_iter().collect();
        for (i, token) in tokens.iter().enumerate() {
            if let TokenTree::Ident(name) = token
                && name == "compile_error"
                && let Some(TokenTree::Group(group)) = tokens.get(i + 2)
                && let Ok(lit) = syn::parse2::<syn::LitStr>(group.stream())
            {
                let d = Diagnostic {
                    start: byte_offset(source, name.span(), false),
                    end: byte_offset(source, name.span(), true),
                    severity: "error".into(),
                    message: lit.value(),
                    suggestion: None,
                };
                if !out.iter().any(|prev| prev.message == d.message) {
                    out.push(d);
                }
            }
            if let TokenTree::Group(group) = token {
                semantic_errors(group.stream(), source, out);
            }
        }
    }
    semantic_errors(expanded, source, &mut analysis.diagnostics);
    analysis
}

pub(crate) fn class_literals(attr: &RsxAttribute) -> Option<Vec<&syn::LitStr>> {
    match attr {
        RsxAttribute::Value { name, value } if name == "class" => static_class_literals(value),
        RsxAttribute::WhenClass { class_lit, .. } | RsxAttribute::StateClass { class_lit, .. } => {
            Some(vec![class_lit])
        }
        _ => None,
    }
}

/// Returns the literal's source span and, when the content maps byte-for-byte (no escapes),
/// the span of its content. `value` must be `lit.value()`, passed in to avoid re-allocating it.
fn literal_source_range(
    source: &str,
    lit: &syn::LitStr,
    value: &str,
) -> (usize, usize, Option<(usize, usize)>) {
    let start = byte_offset(source, lit.span(), false);
    let end = byte_offset(source, lit.span(), true);
    let raw = &source[start..end];
    let range = raw
        .find('"')
        .map(|i| i + 1)
        .filter(|&prefix| raw.get(prefix..prefix + value.len()) == Some(value))
        .map(|prefix| (start + prefix, start + prefix + value.len()));
    (start, end, range)
}

fn lint_literal(
    source: &str,
    lit: &syn::LitStr,
    mode: ClassMode,
    out: &mut Vec<Diagnostic>,
) -> Option<(usize, usize)> {
    let value = lit.value();
    let (start, end, exact_range) = literal_source_range(source, lit, &value);
    for d in class_diagnostics(&value, mode) {
        out.push(Diagnostic {
            start: exact_range.map_or(start, |(content_start, _)| content_start + d.offset),
            end: exact_range.map_or(end, |(content_start, _)| {
                content_start + d.offset + d.class.len()
            }),
            severity: if d.severity == Severity::Error {
                "error"
            } else {
                "warning"
            }
            .into(),
            message: d.message,
            suggestion: if exact_range.is_some() {
                d.suggestion
            } else {
                None
            },
        });
    }
    exact_range
}

pub fn walk_body(body: &RsxBody, visitor: &mut impl FnMut(&RsxElement)) {
    fn nodes(nodes: &[RsxNode], visitor: &mut impl FnMut(&RsxElement)) {
        for node in nodes {
            match node {
                RsxNode::Element(el) => element(el, visitor),
                RsxNode::For { body, .. } | RsxNode::Fragment(body) => self_nodes(body, visitor),
                _ => {}
            }
        }
    }
    fn self_nodes(body: &[RsxNode], visitor: &mut impl FnMut(&RsxElement)) {
        nodes(body, visitor);
    }
    fn element(el: &RsxElement, visitor: &mut impl FnMut(&RsxElement)) {
        visitor(el);
        nodes(&el.children, visitor);
    }
    match body {
        RsxBody::Single(el) => element(el, visitor),
        RsxBody::Fragment(children) => nodes(children, visitor),
        _ => {}
    }
}

pub fn literal_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Str(s) => Some(s.value()),
            _ => None,
        },
        _ => None,
    }
}

/// Pretty-print an expansion as a Rust expression (no GPUI runtime required).
pub fn expand(source: &str, mode: ClassMode) -> Result<String, String> {
    let tokens = gpui_rsx_core::expand_str(source, mode).map_err(|e| e.to_string())?;
    let expr = syn::parse2::<Expr>(tokens).map_err(|e| e.to_string())?;
    let file: syn::File = syn::parse_quote! { fn expanded() { #expr; } };
    Ok(prettyplease::unparse(&file))
}

/// Unescaped class literal content ranges, for completions and color previews.
pub fn class_literal_ranges(source: &str) -> Result<Vec<(usize, usize)>, String> {
    let body = syn::parse_str::<RsxBody>(source).map_err(|e| e.to_string())?;
    let mut ranges = Vec::new();
    walk_body(&body, &mut |el| {
        for attr in &el.attributes {
            if let Some(literals) = class_literals(attr) {
                for lit in literals {
                    if let Some(range) = literal_source_range(source, lit, &lit.value()).2 {
                        ranges.push(range);
                    }
                }
            }
        }
    });
    Ok(ranges)
}
