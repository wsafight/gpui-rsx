//! Shared, host-independent tooling. Positions are UTF-8 byte offsets into the input.
pub mod convert;
pub mod format;
pub mod preview;

use gpui_rsx_core::codegen::attribute::static_class_literals;
use gpui_rsx_core::parser::{RsxAttribute, RsxBody, RsxElement, RsxNode};
use gpui_rsx_core::{ClassMode, Severity, SpacingScale, class_diagnostics};
use proc_macro2::{Span, TokenStream, TokenTree};
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

#[derive(Clone, Debug)]
pub struct MacroCall {
    pub name: String,
    pub start: usize,
    pub end: usize,
}

pub const MACROS: &[&str] = &[
    "rsx",
    "rsx_strict",
    "rsx_permissive",
    "rsx_tw",
    "rsx_expand",
];

pub fn byte_offset(source: &str, span: Span, end: bool) -> usize {
    let loc = if end { span.end() } else { span.start() };
    let line = source
        .split_inclusive('\n')
        .take(loc.line.saturating_sub(1))
        .map(str::len)
        .sum::<usize>();
    let rest = &source[line.min(source.len())..];
    line + rest
        .char_indices()
        .nth(loc.column)
        .map(|(i, _)| i)
        .unwrap_or(rest.len())
}

pub fn macro_calls(source: &str) -> Result<Vec<MacroCall>, String> {
    fn scan(tokens: TokenStream, source: &str, calls: &mut Vec<MacroCall>) {
        let tokens: Vec<_> = tokens.into_iter().collect();
        let mut i = 0;
        while i < tokens.len() {
            if let TokenTree::Ident(name) = &tokens[i]
                && MACROS.contains(&name.to_string().as_str())
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
    ClassMode {
        strict: name == "rsx_strict" || (mode.strict && name != "rsx_permissive"),
        scale: if name == "rsx_tw" {
            SpacingScale::Tailwind
        } else {
            mode.scale
        },
        ..mode
    }
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

pub fn analyze_rust(source: &str, mode: ClassMode) -> Vec<Diagnostic> {
    let calls = match macro_calls(source) {
        Ok(calls) => calls,
        Err(message) => {
            return vec![Diagnostic {
                start: 0,
                end: source.len().min(1),
                severity: "error".into(),
                message,
                suggestion: None,
            }];
        }
    };
    let mut out = Vec::new();
    for call in calls {
        out.extend(
            analyze_body(&source[call.start..call.end], mode_for(&call.name, mode))
                .into_iter()
                .map(|mut d| {
                    d.start += call.start;
                    d.end += call.start;
                    d
                }),
        );
    }
    out
}

pub fn analyze_body(source: &str, mode: ClassMode) -> Vec<Diagnostic> {
    let body = match syn::parse_str::<RsxBody>(source) {
        Ok(body) => body,
        Err(e) => return vec![error(source, e)],
    };
    let mut out = Vec::new();
    walk_body(&body, &mut |el| {
        for attr in &el.attributes {
            match attr {
                RsxAttribute::Value { name, value } if name == "class" => {
                    if let Some(literals) = static_class_literals(value) {
                        for lit in literals {
                            lint_literal(source, lit, mode, &mut out);
                        }
                    } else {
                        out.push(Diagnostic { start: byte_offset(source, value.span(), false), end: byte_offset(source, value.span(), true),
                            severity: "information".into(), message: "Dynamic class expression: values are checked at runtime; variants and theme tokens require static strings.".into(), suggestion: None });
                    }
                }
                RsxAttribute::WhenClass { class_lit, .. }
                | RsxAttribute::StateClass { class_lit, .. } => {
                    lint_literal(source, class_lit, mode, &mut out)
                }
                _ => {}
            }
        }
    });
    // Include semantic errors (required constructors, loop keys, unsupported state classes).
    let expanded = gpui_rsx_core::generate_body_with_mode(&body, mode);
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
    semantic_errors(expanded, source, &mut out);
    out
}

fn lint_literal(source: &str, lit: &syn::LitStr, mode: ClassMode, out: &mut Vec<Diagnostic>) {
    let value = lit.value();
    let start = byte_offset(source, lit.span(), false);
    let end = byte_offset(source, lit.span(), true);
    let raw = &source[start..end];
    let prefix = raw.find('"').map(|i| i + 1).unwrap_or(0);
    // Escaped strings do not have a one-to-one mapping to source bytes; highlight the literal.
    let exact = raw.get(prefix..prefix + value.len()) == Some(&value);
    for d in class_diagnostics(&value, mode) {
        out.push(Diagnostic {
            start: if exact {
                start + prefix + d.offset
            } else {
                start
            },
            end: if exact {
                start + prefix + d.offset + d.class.len()
            } else {
                end
            },
            severity: if d.severity == Severity::Error {
                "error"
            } else {
                "warning"
            }
            .into(),
            message: d.message,
            suggestion: if exact { d.suggestion } else { None },
        });
    }
}

pub fn walk_body(body: &RsxBody, visitor: &mut impl FnMut(&RsxElement)) {
    fn nodes(nodes: &[RsxNode], visitor: &mut impl FnMut(&RsxElement)) {
        for node in nodes {
            match node {
                RsxNode::Element(el) => element(el, visitor),
                RsxNode::For { body, .. } => self_nodes(body, visitor),
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
            let literals = match attr {
                RsxAttribute::Value { name, value } if name == "class" => {
                    static_class_literals(value).unwrap_or_default()
                }
                RsxAttribute::WhenClass { class_lit, .. }
                | RsxAttribute::StateClass { class_lit, .. } => vec![class_lit],
                _ => vec![],
            };
            for lit in literals {
                let start = byte_offset(source, lit.span(), false);
                let end = byte_offset(source, lit.span(), true);
                let raw = &source[start..end];
                if let Some(prefix) = raw.find('"').map(|i| i + 1)
                    && raw.get(prefix..prefix + lit.value().len()) == Some(lit.value().as_str())
                {
                    ranges.push((start + prefix, start + prefix + lit.value().len()));
                }
            }
        }
    });
    Ok(ranges)
}
