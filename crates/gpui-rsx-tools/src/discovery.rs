use crate::{Diagnostic, DiagnosticSeverity, MACROS, MacroCall, byte_offset};
use proc_macro2::{TokenStream, TokenTree};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

type Aliases = HashMap<String, &'static str>;

#[derive(Clone, Default)]
struct Bindings {
    aliases: Aliases,
    shadowed: HashSet<String>,
    foreign_imports: HashSet<String>,
}

fn canonical(name: &str) -> Option<&'static str> {
    MACROS.iter().copied().find(|candidate| *candidate == name)
}

fn identifier(ident: &proc_macro2::Ident) -> String {
    let mut name = ident.to_string();
    if name.starts_with("r#") {
        name.replace_range(..2, "");
    }
    name
}

fn imports(tree: &syn::UseTree, prefix: &mut Vec<String>, bindings: &mut Bindings) {
    match tree {
        syn::UseTree::Path(path) => {
            prefix.push(identifier(&path.ident));
            imports(&path.tree, prefix, bindings);
            prefix.pop();
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                imports(item, prefix, bindings);
            }
        }
        syn::UseTree::Name(name) => bind(&name.ident, &name.ident, prefix, bindings),
        syn::UseTree::Rename(rename) => bind(&rename.ident, &rename.rename, prefix, bindings),
        syn::UseTree::Glob(_) => {}
    }
}

fn bind(
    original: &proc_macro2::Ident,
    local: &proc_macro2::Ident,
    prefix: &[String],
    bindings: &mut Bindings,
) {
    let local = identifier(local);
    bindings.aliases.remove(&local);
    bindings.foreign_imports.remove(&local);
    if local == "_" {
        return;
    }
    if prefix == ["gpui_rsx"]
        && let Some(name) = canonical(&identifier(original))
    {
        bindings.aliases.insert(local, name);
    } else {
        // An explicit foreign binding must not fall back to a same-named GPUI-RSX macro.
        bindings.foreign_imports.insert(local);
    }
}

fn double_colon_before(tokens: &[TokenTree], index: usize) -> bool {
    index >= 2
        && tokens[index - 2..index]
            .iter()
            .all(|token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == ':'))
}

/// Returns whether a macro path is absolute and its identifier segments, including the macro.
fn invocation_path(tokens: &[TokenTree], index: usize) -> (bool, Vec<String>) {
    let TokenTree::Ident(name) = &tokens[index] else {
        unreachable!("macro names are identifiers")
    };
    let mut segments = vec![identifier(name)];
    let mut start = index;
    let mut absolute = false;
    while double_colon_before(tokens, start) {
        if start >= 3
            && let TokenTree::Ident(segment) = &tokens[start - 3]
        {
            segments.push(identifier(segment));
            start -= 3;
        } else {
            absolute = true;
            break;
        }
    }
    segments.reverse();
    (absolute, segments)
}

fn scan_scope(
    tokens: TokenStream,
    source: &str,
    nested: bool,
    mut bindings: Bindings,
    spelled: bool,
    calls: &mut Vec<MacroCall>,
) {
    let tokens: Vec<_> = tokens.into_iter().collect();
    for (index, token) in tokens.iter().enumerate() {
        if matches!(token, TokenTree::Ident(name) if name == "use")
            && let Some(end) = tokens[index..].iter().position(
                |token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == ';'),
            )
        {
            let item: TokenStream = tokens[index..=index + end].iter().cloned().collect();
            if let Ok(item) = syn::parse2::<syn::ItemUse>(item) {
                imports(&item.tree, &mut Vec::new(), &mut bindings);
            }
        }
    }
    let mut index = 0;
    while index < tokens.len() {
        if let TokenTree::Ident(name) = &tokens[index] {
            if name == "macro_rules"
                && let Some(TokenTree::Punct(bang)) = tokens.get(index + 1)
                && bang.as_char() == '!'
                && let Some(TokenTree::Ident(local)) = tokens.get(index + 2)
                && let Some(TokenTree::Group(_)) = tokens.get(index + 3)
            {
                bindings.shadowed.insert(identifier(local));
            }
            if let Some(TokenTree::Punct(bang)) = tokens.get(index + 1)
                && bang.as_char() == '!'
                && let Some(TokenTree::Group(group)) = tokens.get(index + 2)
            {
                let local = identifier(name);
                let (absolute, segments) = invocation_path(&tokens, index);
                let canonical = if absolute || segments.len() > 1 {
                    // Qualified calls are GPUI-RSX only at exactly `gpui_rsx::<macro>` or
                    // `::gpui_rsx::<macro>`, never merely because their last segment matches.
                    if segments.len() == 2 && segments[0] == "gpui_rsx" {
                        canonical(&local)
                    } else {
                        None
                    }
                } else if bindings.shadowed.contains(&local)
                    || bindings.foreign_imports.contains(&local)
                {
                    None
                } else {
                    bindings
                        .aliases
                        .get(&local)
                        .copied()
                        .or_else(|| canonical(&local))
                };
                if let Some(canonical_name) = canonical {
                    calls.push(MacroCall {
                        name: if spelled {
                            name.to_string()
                        } else {
                            canonical_name.into()
                        },
                        start: byte_offset(source, group.span_open(), true),
                        end: byte_offset(source, group.span_close(), false),
                    });
                    if nested {
                        scan_scope(
                            group.stream(),
                            source,
                            nested,
                            bindings.clone(),
                            spelled,
                            calls,
                        );
                    }
                    index += 3;
                    continue;
                }
            }
        }
        if let TokenTree::Group(group) = &tokens[index] {
            let module =
                index >= 2 && matches!(&tokens[index - 2], TokenTree::Ident(name) if name == "mod");
            let inherited = if module {
                Bindings {
                    aliases: Aliases::new(),
                    shadowed: bindings.shadowed.clone(),
                    foreign_imports: HashSet::new(),
                }
            } else {
                bindings.clone()
            };
            scan_scope(group.stream(), source, nested, inherited, spelled, calls);
        }
        index += 1;
    }
}

fn nondefault_import(tree: &syn::UseTree, prefix: &mut Vec<String>) -> bool {
    match tree {
        syn::UseTree::Path(path) => {
            prefix.push(identifier(&path.ident));
            let nondefault = nondefault_import(&path.tree, prefix);
            prefix.pop();
            nondefault
        }
        syn::UseTree::Group(group) => group
            .items
            .iter()
            .any(|item| nondefault_import(item, prefix)),
        syn::UseTree::Name(name) => {
            canonical(&identifier(&name.ident)).is_some() && prefix.as_slice() != ["gpui_rsx"]
        }
        // A body-only rescan cannot reproduce either a positive or negative renamed binding.
        syn::UseTree::Rename(_) => true,
        syn::UseTree::Glob(_) => false,
    }
}

/// Body-only incremental rescans start with empty bindings. Imports that change bare macro
/// resolution therefore require a full scan; plain canonical GPUI-RSX imports do not.
pub(crate) fn imports_require_full_scan(source: &str) -> bool {
    if !source.contains("use") {
        return false;
    }
    fn scan(tokens: TokenStream) -> bool {
        let tokens: Vec<_> = tokens.into_iter().collect();
        for (index, token) in tokens.iter().enumerate() {
            if matches!(token, TokenTree::Ident(name) if name == "use")
                && let Some(end) = tokens[index..].iter().position(
                    |token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == ';'),
                )
            {
                let item: TokenStream = tokens[index..=index + end].iter().cloned().collect();
                if syn::parse2::<syn::ItemUse>(item)
                    .is_ok_and(|item| nondefault_import(&item.tree, &mut Vec::new()))
                {
                    return true;
                }
            }
            if let TokenTree::Group(group) = token
                && scan(group.stream())
            {
                return true;
            }
        }
        false
    }
    source.parse::<TokenStream>().map_or(true, scan)
}

pub(crate) fn scan(source: &str, nested: bool) -> (Vec<MacroCall>, Option<Diagnostic>) {
    scan_source(source, nested, false)
}

fn scan_source(source: &str, nested: bool, spelled: bool) -> (Vec<MacroCall>, Option<Diagnostic>) {
    let mut repaired = Cow::Borrowed(source);
    let mut diagnostic = None;
    for _attempt in 0..32 {
        match repaired.parse::<TokenStream>() {
            Ok(tokens) => {
                let mut calls = Vec::new();
                scan_scope(
                    tokens,
                    source,
                    nested,
                    Bindings::default(),
                    spelled,
                    &mut calls,
                );
                return (calls, diagnostic);
            }
            Err(error) => {
                let offset = byte_offset(&repaired, error.span(), false);
                diagnostic.get_or_insert_with(|| Diagnostic {
                    start: offset,
                    end: offset + source[offset..].chars().next().map_or(0, char::len_utf8),
                    severity: DiagnosticSeverity::Error,
                    code: Some("rsx/rust-syntax".into()),
                    message: error.to_string(),
                    suggestion: None,
                });
                match repaired.as_bytes().get(offset) {
                    Some(b'(' | b')' | b'[' | b']' | b'{' | b'}') => {
                        repaired.to_mut().replace_range(offset..offset + 1, " ");
                    }
                    Some(_) => repaired.to_mut().truncate(offset),
                    None => break,
                }
            }
        }
    }
    (Vec::new(), diagnostic)
}

pub(crate) fn calls(source: &str, nested: bool) -> Result<Vec<MacroCall>, String> {
    calls_with_spelling(source, nested, false)
}

pub(crate) fn calls_with_spelling(
    source: &str,
    nested: bool,
    spelled: bool,
) -> Result<Vec<MacroCall>, String> {
    let (calls, diagnostic) = scan_source(source, nested, spelled);
    if calls.is_empty()
        && let Some(diagnostic) = diagnostic
    {
        return Err(diagnostic.message);
    }
    Ok(calls)
}
