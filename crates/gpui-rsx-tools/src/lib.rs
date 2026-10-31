//! Shared, host-independent tooling. Positions are UTF-8 byte offsets into the input.
mod class_tokens;
pub mod completion;
pub mod components;
#[cfg(feature = "convert")]
pub mod convert;
pub mod format;
mod incremental;
pub use class_tokens::{ClassToken, ClassTokenKind};
pub mod preview;

use gpui_rsx_core::codegen::attribute::static_class_literals;
use gpui_rsx_core::syntax::{RsxAttribute, RsxBody, RsxElement, RsxNode};
use gpui_rsx_core::{ClassMode, Severity, SpacingScale, class_diagnostics};
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use serde::{Deserialize, Serialize};
use syn::{Expr, Lit, spanned::Spanned};

/// Serialized in lowercase (`"error"`, `"warning"`, `"information"`).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
}

impl DiagnosticSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Information => "information",
        }
    }
}

impl std::fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub start: usize,
    pub end: usize,
    pub severity: DiagnosticSeverity,
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
    pub components: Vec<components::Component>,
    pub class_tokens: Vec<ClassToken>,
    pub calls: Vec<MacroCall>,
    pub literals: Vec<(usize, usize)>,
    pub diagnostics: Vec<Diagnostic>,
}

pub const MACROS: &[&str] = gpui_rsx_core::MACRO_NAMES;

fn is_macro_name(name: &Ident) -> bool {
    MACROS.iter().any(|macro_name| name == macro_name)
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

/// Top-level RSX macro calls, in source order. Their ranges never overlap, so callers may
/// rewrite each body independently.
pub fn macro_calls(source: &str) -> Result<Vec<MacroCall>, String> {
    scan_macro_calls(source, false)
}

/// Every RSX macro call, including calls nested inside another call's body (for example
/// `rsx!` inside an `if` child). Outer calls come before the calls they contain.
pub fn nested_macro_calls(source: &str) -> Result<Vec<MacroCall>, String> {
    scan_macro_calls(source, true)
}

fn scan_macro_calls(source: &str, nested: bool) -> Result<Vec<MacroCall>, String> {
    fn scan(tokens: TokenStream, source: &str, nested: bool, calls: &mut Vec<MacroCall>) {
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
                if nested {
                    scan(group.stream(), source, nested, calls);
                }
                i += 3;
                continue;
            }
            if let TokenTree::Group(group) = &tokens[i] {
                scan(group.stream(), source, nested, calls);
            }
            i += 1;
        }
    }
    let tokens = source.parse::<TokenStream>().map_err(|e| e.to_string())?;
    let mut calls = Vec::new();
    scan(tokens, source, nested, &mut calls);
    Ok(calls)
}

/// The innermost call whose body contains `offset`.
pub fn call_at(calls: &[MacroCall], offset: usize) -> Option<&MacroCall> {
    calls
        .iter()
        .filter(|call| offset >= call.start && offset <= call.end)
        .min_by_key(|call| call.end - call.start)
}

/// Whether `start..end` belongs to `call` itself rather than to a call nested inside it.
/// Each body analysis treats nested macros as opaque, so every range has one owner.
fn owned_by(calls: &[MacroCall], call: &MacroCall, start: usize, end: usize) -> bool {
    start >= call.start
        && end <= call.end
        && !calls.iter().any(|inner| {
            inner != call
                && inner.start >= call.start
                && inner.end <= call.end
                && start >= inner.start
                && end <= inner.end
        })
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
    let start = byte_offset(source, e.span(), false);
    let end = byte_offset(source, e.span(), true);
    let message = e.to_string();
    let suggestion = if message.starts_with("Bare text is not supported") {
        Some(format!("{:?}", &source[start..end]))
    } else {
        message
            .split("help: Change the closing tag to `</")
            .nth(1)
            .and_then(|rest| rest.split(">`").next())
            .map(str::to_owned)
    };
    Diagnostic {
        start: byte_offset(source, e.span(), false),
        end: byte_offset(source, e.span(), true),
        severity: DiagnosticSeverity::Error,
        message: e.to_string(),
        suggestion,
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
/// unchanged bodies. Edits confined to a balanced macro rescan only that body;
/// structural edits fall back to scanning the Rust token stream. `mode` must match
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
    if !source.contains("rsx") {
        return RustAnalysis::default();
    }
    let cached = previous.and_then(|(old, analysis)| incremental::calls(source, old, analysis));
    let components = if cached.is_some() {
        previous.unwrap().1.components.clone()
    } else {
        components::collect(source)
    };
    let calls = match cached.map_or_else(|| nested_macro_calls(source), Ok) {
        Ok(calls) => calls,
        Err(message) => {
            return RustAnalysis {
                diagnostics: vec![Diagnostic {
                    start: 0,
                    end: source.len().min(1),
                    severity: DiagnosticSeverity::Error,
                    message,
                    suggestion: None,
                }],
                ..RustAnalysis::default()
            };
        }
    };
    let mut analysis = RustAnalysis {
        calls,
        components,
        ..RustAnalysis::default()
    };
    let mut reusable = std::collections::HashMap::new();
    if let Some((previous_source, previous)) = previous {
        for call in &previous.calls {
            if previous.components != analysis.components {
                break;
            }
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
                    .filter(|&&(start, end)| owned_by(&previous.calls, old_call, start, end))
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
                        owned_by(&previous.calls, old_call, diagnostic.start, diagnostic.end)
                    })
                    .map(|diagnostic| {
                        let mut diagnostic = diagnostic.clone();
                        diagnostic.start = call.start + diagnostic.start - old_call.start;
                        diagnostic.end = call.start + diagnostic.end - old_call.start;
                        diagnostic
                    }),
            );
            analysis.class_tokens.extend(
                previous
                    .class_tokens
                    .iter()
                    .filter(|t| owned_by(&previous.calls, old_call, t.start, t.end))
                    .map(|t| {
                        let mut t = t.clone();
                        t.start = call.start + t.start - old_call.start;
                        t.end = call.start + t.end - old_call.start;
                        t
                    }),
            );
            continue;
        }
        let body_analysis = analyze_body_with_ranges(
            body,
            mode_for(&call.name, mode.clone()),
            &analysis.components,
        );
        analysis
            .class_tokens
            .extend(body_analysis.class_tokens.into_iter().map(|mut t| {
                t.start += call.start;
                t.end += call.start;
                t
            }));
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
    analyze_body_with_ranges(source, mode, &[]).diagnostics
}

struct BodyAnalysis {
    class_tokens: Vec<ClassToken>,
    literals: Vec<(usize, usize)>,
    diagnostics: Vec<Diagnostic>,
}

/// Diagnostics, pretty-printed expansion and static preview of one RSX body.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct BodyReport {
    pub diagnostics: Vec<Diagnostic>,
    pub expansion: Result<String, String>,
    pub preview: Result<String, String>,
}

/// Same results as [`analyze_body`], [`expand`] and [`preview::render`], but parses and
/// expands the body once; the playground calls this on every keystroke.
pub fn report_body(source: &str, mode: ClassMode) -> BodyReport {
    match syn::parse_str::<RsxBody>(source) {
        Ok(body) => {
            let (analysis, (expanded, dynamic)) = analyze_parsed_body(source, &body, mode.clone());
            BodyReport {
                diagnostics: analysis.diagnostics,
                expansion: pretty_expansion(expanded).map(|code| with_helper_note(code, dynamic)),
                preview: preview::render_body(&body, mode),
            }
        }
        Err(e) => BodyReport {
            expansion: Err(e.to_string()),
            preview: Err(e.to_string()),
            diagnostics: vec![error(source, e)],
        },
    }
}

fn analyze_body_with_ranges(
    source: &str,
    mode: ClassMode,
    components: &[components::Component],
) -> BodyAnalysis {
    match syn::parse_str::<RsxBody>(source) {
        Ok(body) => {
            let mut result = analyze_parsed_body(source, &body, mode).0;
            components::diagnostics(source, &body, components, &mut result.diagnostics);
            result
        }
        Err(e) => BodyAnalysis {
            class_tokens: Vec::new(),
            literals: Vec::new(),
            diagnostics: vec![error(source, e)],
        },
    }
}

/// Lints a parsed body and returns its helper-free expansion, which semantic checks already
/// need, with whether it references the dynamic class helper.
fn analyze_parsed_body(
    source: &str,
    body: &RsxBody,
    mode: ClassMode,
) -> (BodyAnalysis, (TokenStream, bool)) {
    let mut analysis = BodyAnalysis {
        class_tokens: Vec::new(),
        literals: Vec::new(),
        diagnostics: Vec::new(),
    };
    walk_body(body, &mut |el| {
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
                        let mut message = "Dynamic class expression: values are checked at runtime; variants and theme tokens require static strings.".to_owned();
                        if mode.dynamic_runtime_path().is_none() {
                            message.push_str(" For many call sites, generate gpui_rsx::rsx_runtime!() once and set [options] dynamic-runtime = \"rsx_runtime\" in rsx.toml to reduce compilation time (50 call sites saved about 12s in a local release build).");
                        }
                        analysis.diagnostics.push(Diagnostic {
                            start: byte_offset(source, value.span(), false),
                            end: byte_offset(source, value.span(), true),
                            severity: DiagnosticSeverity::Information,
                            message,
                            suggestion: None,
                        });
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
    // The dynamic class helper holds no diagnostics and would dominate the walk below.
    analysis.class_tokens = class_tokens::collect(source, &analysis.literals, mode.clone());
    let (expanded, dynamic) = gpui_rsx_core::generate_body_without_runtime(body, mode);
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
                    severity: DiagnosticSeverity::Error,
                    message: lit.value(),
                    suggestion: lit.value().contains("has no `id` or `key`").then(|| {
                        format!(
                            "{} key={{todo!(\"choose a stable unique key\") as usize}}",
                            &source[byte_offset(source, name.span(), false)
                                ..byte_offset(source, name.span(), true)]
                        )
                    }),
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
    semantic_errors(expanded.clone(), source, &mut analysis.diagnostics);
    (analysis, (expanded, dynamic))
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
                DiagnosticSeverity::Error
            } else {
                DiagnosticSeverity::Warning
            },
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
/// Like [`expand`], but omits the dynamic class helper and notes where it is used; for hovers
/// and previews where its ~1,400 lines would bury the call site.
pub fn expand_summary(source: &str, mode: ClassMode) -> Result<String, String> {
    let body = syn::parse_str::<RsxBody>(source).map_err(|e| e.to_string())?;
    let (code, dynamic) = gpui_rsx_core::generate_body_without_runtime(&body, mode);
    pretty_expansion(code).map(|code| with_helper_note(code, dynamic))
}

pub fn expand_summary_for_macro(
    source: &str,
    name: &str,
    mode: ClassMode,
) -> Result<String, String> {
    let mode = mode_for(name, mode);
    if name != "rsx_fragment" {
        return expand_summary(source, mode);
    }
    let body = syn::parse_str::<RsxBody>(source).map_err(|e| e.to_string())?;
    let (code, dynamic) = gpui_rsx_core::generate_fragment_without_runtime(&body, mode);
    pretty_expansion(code).map(|code| with_helper_note(code, dynamic))
}

fn with_helper_note(code: String, dynamic: bool) -> String {
    if dynamic {
        format!(
            "// `__rsx_apply_class` is the dynamic class helper; `cargo rsx expand` shows it.\n{code}"
        )
    } else {
        code
    }
}

pub fn expand(source: &str, mode: ClassMode) -> Result<String, String> {
    pretty_expansion(gpui_rsx_core::expand_str(source, mode).map_err(|e| e.to_string())?)
}

fn pretty_expansion(tokens: TokenStream) -> Result<String, String> {
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
