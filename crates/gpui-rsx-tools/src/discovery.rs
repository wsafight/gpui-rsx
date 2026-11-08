use crate::{Diagnostic, DiagnosticSeverity, MACROS, MacroCall, byte_offset};
use proc_macro2::{TokenStream, TokenTree};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

type Aliases = HashMap<String, &'static str>;

#[derive(Clone, Default)]
struct Bindings {
    aliases: Aliases,
    shadowed: HashSet<String>,
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

fn imports(tree: &syn::UseTree, prefix: &mut Vec<String>, aliases: &mut Aliases) {
    match tree {
        syn::UseTree::Path(path) => {
            prefix.push(identifier(&path.ident));
            imports(&path.tree, prefix, aliases);
            prefix.pop();
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                imports(item, prefix, aliases);
            }
        }
        syn::UseTree::Name(name) => bind(&name.ident, &name.ident, prefix, aliases),
        syn::UseTree::Rename(rename) => bind(&rename.ident, &rename.rename, prefix, aliases),
        syn::UseTree::Glob(_) => {}
    }
}

fn bind(
    original: &proc_macro2::Ident,
    local: &proc_macro2::Ident,
    prefix: &[String],
    aliases: &mut Aliases,
) {
    let local = identifier(local);
    aliases.remove(&local);
    if local != "_"
        && prefix == ["gpui_rsx"]
        && let Some(name) = canonical(&identifier(original))
    {
        aliases.insert(local, name);
    }
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
                imports(&item.tree, &mut Vec::new(), &mut bindings.aliases);
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
                let qualified = index >= 2
                    && tokens[index - 2..index].iter().all(
                        |token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == ':'),
                    );
                let local = identifier(name);
                let canonical = if qualified {
                    canonical(&local)
                } else if bindings.shadowed.contains(&local) {
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
                }
            } else {
                bindings.clone()
            };
            scan_scope(group.stream(), source, nested, inherited, spelled, calls);
        }
        index += 1;
    }
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
