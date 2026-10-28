//! Formatting and explicit, behavior-preserving spacing migration.
use crate::{byte_offset, class_literals, macro_calls, walk_body};
use gpui_rsx_core::syntax::{RsxAttribute, RsxBody, RsxElement, RsxNode};
use quote::ToTokens;
use syn::spanned::Spanned;

/// Copies an embedded Rust expression from the source instead of re-printing its tokens,
/// which would space out every token (`cx . listener (| this |`). Continuation lines are
/// re-based onto `indent`, so formatting twice is a no-op.
fn expression(source: &str, value: &(impl ToTokens + Spanned), indent: &str) -> String {
    let span = value.span();
    let (start, end) = (
        byte_offset(source, span, false),
        byte_offset(source, span, true),
    );
    let Some(text) = source
        .get(start..end)
        .filter(|text| !text.trim().is_empty())
    else {
        return value.to_token_stream().to_string();
    };
    let indentation = |line: &str| line.len() - line.trim_start_matches([' ', '\t']).len();
    let continuation = text
        .split('\n')
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(indentation)
        .min()
        .unwrap_or(0);
    // On the body's first line (`rsx!{<div on_click={...`) the line's own indentation is
    // not the template's, so only the continuation lines can provide a base.
    let base = match source[..start].rfind('\n') {
        Some(i) => continuation.min(indentation(&source[i + 1..])),
        None => continuation,
    };
    let mut out = String::new();
    for (i, line) in text.split('\n').enumerate() {
        let line = line.trim_end_matches('\r');
        if i == 0 {
            out.push_str(line);
            continue;
        }
        out.push('\n');
        let rest = &line[indentation(line).min(base)..];
        if !rest.trim().is_empty() {
            out.push_str(indent);
            out.push_str(rest);
        }
    }
    out
}

pub fn body(source: &str) -> Result<String, String> {
    let body = syn::parse_str::<RsxBody>(source).map_err(|e| e.to_string())?;
    if let Some(reason) = unformattable(source)? {
        return Err(reason.into());
    }
    let mut out = String::new();
    fn element(
        source: &str,
        el: &RsxElement,
        depth: usize,
        out: &mut String,
    ) -> Result<(), String> {
        let indent = "    ".repeat(depth);
        let expression = |value: &dyn ExprLike| value.text(source, &indent);
        out.push_str(&format!("{indent}<{}", el.name));
        for attr in &el.attributes {
            let attr = match attr {
                RsxAttribute::Flag(name) => name.to_string(),
                // Literals need no braces (`class="flex"`), matching how they are usually written.
                RsxAttribute::Value {
                    name,
                    value: value @ syn::Expr::Lit(lit),
                } if lit.attrs.is_empty() => format!("{name}={}", expression(value)),
                RsxAttribute::Value { name, value } => format!("{name}={{{}}}", expression(value)),
                RsxAttribute::When { condition, closure } => format!(
                    "when={{({}, {})}}",
                    expression(condition),
                    expression(closure)
                ),
                RsxAttribute::WhenSome { option, closure } => format!(
                    "whenSome={{({}, {})}}",
                    expression(option),
                    expression(closure)
                ),
                RsxAttribute::WhenClass {
                    condition,
                    class_lit,
                } => format!(
                    "whenClass={{({}, {})}}",
                    expression(condition),
                    expression(class_lit)
                ),
                RsxAttribute::StateClass { method, class_lit } => {
                    let name = match method.to_string().as_str() {
                        "hover" => "hoverClass",
                        "focus" => "focusClass",
                        "active" => "activeClass",
                        _ => unreachable!("parser restricts state class methods"),
                    };
                    format!("{name}={}", expression(class_lit))
                }
                _ => return Err("formatter does not support this RSX attribute".into()),
            };
            out.push(' ');
            out.push_str(&attr);
        }
        if el.children.is_empty() {
            out.push_str(" />\n");
        } else {
            out.push_str(">\n");
            nodes(source, &el.children, depth + 1, out)?;
            out.push_str(&format!("{indent}</{}>\n", el.name));
        }
        Ok(())
    }
    fn nodes(
        source: &str,
        children: &[RsxNode],
        depth: usize,
        out: &mut String,
    ) -> Result<(), String> {
        let indent = "    ".repeat(depth);
        let expression = |value: &dyn ExprLike| value.text(source, &indent);
        for child in children {
            match child {
                RsxNode::Element(el) => element(source, el, depth, out)?,
                RsxNode::Fragment(children) => {
                    out.push_str(&format!("{indent}<>\n"));
                    nodes(source, children, depth + 1, out)?;
                    out.push_str(&format!("{indent}</>\n"));
                }
                RsxNode::Expr(expr) => out.push_str(&format!("{indent}{{{}}}\n", expression(expr))),
                RsxNode::Spread(expr) => {
                    out.push_str(&format!("{indent}{{...{}}}\n", expression(expr)))
                }
                RsxNode::For {
                    binding,
                    iter,
                    body,
                } => {
                    out.push_str(&format!(
                        "{indent}{{for {} in {} {{\n",
                        expression(binding),
                        expression(iter)
                    ));
                    nodes(source, body, depth + 1, out)?;
                    out.push_str(&format!("{indent}}}}}\n"));
                }
                _ => return Err("formatter does not support this RSX node".into()),
            }
        }
        Ok(())
    }
    match body {
        RsxBody::Single(el) => element(source, &el, 0, &mut out)?,
        RsxBody::Fragment(children) => {
            out.push_str("<>\n");
            nodes(source, &children, 1, &mut out)?;
            out.push_str("</>\n");
        }
        _ => return Err("formatter does not support this RSX body".into()),
    }
    syn::parse_str::<RsxBody>(&out).map_err(|e| format!("formatter produced invalid RSX: {e}"))?;
    Ok(out.trim_end().into())
}

/// Object-safe view of the syn nodes `expression` accepts.
trait ExprLike {
    fn text(&self, source: &str, indent: &str) -> String;
}

impl<T: ToTokens + Spanned> ExprLike for T {
    fn text(&self, source: &str, indent: &str) -> String {
        expression(source, self, indent)
    }
}

/// Why a body can't be reformatted losslessly, if it can't.
fn unformattable(source: &str) -> Result<Option<&'static str>, String> {
    let (comments, multiline_literal) = scan_literals(source)?;
    // Comments are not represented by syn's AST. Refuse rather than discard them.
    if comments {
        return Ok(Some(
            "RSX body contains comments; leave this macro unchanged or move comments outside it before formatting",
        ));
    }
    // Re-indenting would change the literal's contents.
    if multiline_literal {
        return Ok(Some(
            "RSX body contains a multi-line string literal; leave this macro unchanged",
        ));
    }
    Ok(None)
}

/// Returns (has comments, has a literal spanning several lines).
fn scan_literals(source: &str) -> Result<(bool, bool), String> {
    use proc_macro2::{TokenStream, TokenTree};
    fn literals(tokens: TokenStream, source: &str, ranges: &mut Vec<(usize, usize)>) {
        for token in tokens {
            match token {
                TokenTree::Literal(lit) => ranges.push((
                    byte_offset(source, lit.span(), false),
                    byte_offset(source, lit.span(), true),
                )),
                TokenTree::Group(group) => literals(group.stream(), source, ranges),
                _ => {}
            }
        }
    }
    let mut ranges = Vec::new();
    literals(
        source
            .parse()
            .map_err(|e: proc_macro2::LexError| e.to_string())?,
        source,
        &mut ranges,
    );
    let comments = source.as_bytes().windows(2).enumerate().any(|(i, pair)| {
        (pair == b"//" || pair == b"/*") && !ranges.iter().any(|&(s, e)| i >= s && i < e)
    });
    let multiline = ranges.iter().any(|&(s, e)| source[s..e].contains('\n'));
    Ok((comments, multiline))
}

/// Only rewrites RSX macro bodies, preserving surrounding Rust byte for byte.
///
/// Bodies that don't parse are left as they are, so one broken macro (reported by lint)
/// doesn't block formatting the rest of the file.
pub fn rust(source: &str) -> Result<String, String> {
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut out = source.to_owned();
    for call in macro_calls(source)?.into_iter().rev() {
        let raw = &source[call.start..call.end];
        if !matches!(unformattable(raw), Ok(None)) {
            continue;
        }
        let Ok(formatted) = body(raw) else {
            continue;
        };
        let line_start = source[..call.start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let indent: String = source[line_start..call.start]
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect();
        let formatted = formatted
            .lines()
            .map(|line| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("{indent}    {line}")
                }
            })
            .collect::<Vec<_>>()
            .join(newline);
        out.replace_range(
            call.start..call.end,
            &format!("{newline}{formatted}{newline}{indent}"),
        );
    }
    Ok(out)
}

/// Convert numeric px classes to Tailwind units while retaining the same pixel values.
/// Explicit px/rem values, fractions, theme tokens and dynamic expressions are untouched.
pub fn migrate_rust(source: &str) -> Result<String, String> {
    let mut edits = Vec::<(usize, usize, String)>::new();
    for call in macro_calls(source)? {
        if call.name == "rsx_tw" {
            continue;
        }
        let raw = &source[call.start..call.end];
        let body = syn::parse_str::<RsxBody>(raw).map_err(|e| e.to_string())?;
        walk_body(&body, &mut |el| {
            for attr in &el.attributes {
                for lit in class_literals(attr).into_iter().flatten() {
                    let value = lit.value();
                    let converted = value
                        .split_ascii_whitespace()
                        .map(migrate_class)
                        .collect::<Vec<_>>()
                        .join(" ");
                    if converted != value {
                        let start = byte_offset(raw, lit.span(), false) + call.start;
                        let end = byte_offset(raw, lit.span(), true) + call.start;
                        edits.push((
                            start,
                            end,
                            syn::LitStr::new(&converted, lit.span())
                                .to_token_stream()
                                .to_string(),
                        ));
                    }
                }
            }
        });
    }
    edits.sort_by_key(|e| e.0);
    let mut out = source.to_owned();
    for (start, end, replacement) in edits.into_iter().rev() {
        out.replace_range(start..end, &replacement);
    }
    Ok(out)
}

fn migrate_class(class: &str) -> String {
    let (variant, base) = class
        .rsplit_once(':')
        .map(|(v, b)| (format!("{v}:"), b))
        .unwrap_or_else(|| (String::new(), class));
    if let Some((_, value, _)) = gpui_rsx_core::codegen::tables::split_length_class(base)
        && !value.contains(['[', '/', '%'])
        && let Ok(n) = value.parse::<f32>()
        && n.is_finite()
    {
        let prefix = &base[..base.len() - value.len()];
        return format!("{variant}{prefix}{}", n / 4.0);
    }
    class.into()
}
