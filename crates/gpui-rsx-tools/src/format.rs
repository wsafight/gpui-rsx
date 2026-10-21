//! Formatting and explicit, behavior-preserving spacing migration.
use crate::{byte_offset, macro_calls, walk_body};
use gpui_rsx_core::codegen::attribute::static_class_literals;
use gpui_rsx_core::parser::{RsxAttribute, RsxBody, RsxElement, RsxNode};
use quote::ToTokens;

fn expression(value: &impl ToTokens) -> String {
    value.to_token_stream().to_string()
}

pub fn body(source: &str) -> Result<String, String> {
    let body = syn::parse_str::<RsxBody>(source).map_err(|e| e.to_string())?;
    // Comments are not represented by syn's AST. Refuse rather than discard them.
    if has_comments(source)? {
        return Err("RSX body contains comments; leave this macro unchanged or move comments outside it before formatting".into());
    }
    let mut out = String::new();
    fn element(el: &RsxElement, depth: usize, out: &mut String) {
        let indent = "    ".repeat(depth);
        out.push_str(&format!("{indent}<{}", el.name));
        for attr in &el.attributes {
            let attr = match attr {
                RsxAttribute::Flag(name) => name.to_string(),
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
            };
            out.push(' ');
            out.push_str(&attr);
        }
        if el.children.is_empty() {
            out.push_str(" />\n");
        } else {
            out.push_str(">\n");
            nodes(&el.children, depth + 1, out);
            out.push_str(&format!("{indent}</{}>\n", el.name));
        }
    }
    fn nodes(children: &[RsxNode], depth: usize, out: &mut String) {
        for child in children {
            let indent = "    ".repeat(depth);
            match child {
                RsxNode::Element(el) => element(el, depth, out),
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
                    nodes(body, depth + 1, out);
                    out.push_str(&format!("{indent}}}}}\n"));
                }
            }
        }
    }
    match body {
        RsxBody::Single(el) => element(&el, 0, &mut out),
        RsxBody::Fragment(children) => {
            out.push_str("<>\n");
            nodes(&children, 1, &mut out);
            out.push_str("</>\n");
        }
    }
    syn::parse_str::<RsxBody>(&out).map_err(|e| format!("formatter produced invalid RSX: {e}"))?;
    Ok(out.trim_end().into())
}

fn has_comments(source: &str) -> Result<bool, String> {
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
    Ok(source.as_bytes().windows(2).enumerate().any(|(i, pair)| {
        (pair == b"//" || pair == b"/*") && !ranges.iter().any(|&(s, e)| i >= s && i < e)
    }))
}

/// Only rewrites RSX macro bodies, preserving surrounding Rust byte for byte.
pub fn rust(source: &str) -> Result<String, String> {
    let mut out = source.to_owned();
    for call in macro_calls(source)?.into_iter().rev() {
        let raw = &source[call.start..call.end];
        if has_comments(raw)? {
            continue;
        }
        let formatted = body(raw)?;
        let line_start = source[..call.start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let indent: String = source[line_start..call.start]
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect();
        let formatted = formatted
            .lines()
            .map(|line| format!("{indent}    {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        out.replace_range(call.start..call.end, &format!("\n{formatted}\n{indent}"));
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
                let literals = match attr {
                    RsxAttribute::Value { name, value } if name == "class" => {
                        static_class_literals(value).unwrap_or_default()
                    }
                    RsxAttribute::WhenClass { class_lit, .. }
                    | RsxAttribute::StateClass { class_lit, .. } => vec![class_lit],
                    _ => vec![],
                };
                for lit in literals {
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
