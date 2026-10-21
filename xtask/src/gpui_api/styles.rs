//! Reconstructs style method families from GPUI macro templates.

use super::{Method, Origin, parse_file};
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use syn::visit::Visit;

#[derive(Clone, Debug, Default)]
struct DataEntry {
    value: String,
    auto_allowed: bool,
}

/// Model of `gpui_macros/src/styles.rs`.
pub(super) struct StylesModel {
    pub(super) fns: BTreeMap<String, syn::ItemFn>,
    /// Data functions returning `vec![Struct { prefix|suffix: "..", .. }]`.
    data: BTreeMap<String, Vec<DataEntry>>,
}

impl StylesModel {
    pub(super) fn load(path: &Path) -> Result<Self, String> {
        let file = parse_file(path)?;
        let mut fns = BTreeMap::new();
        let mut data = BTreeMap::new();
        for item in file.items {
            if let syn::Item::Fn(f) = item {
                let name = f.sig.ident.to_string();
                if let Some(entries) = data_entries(&f) {
                    data.insert(name.clone(), entries);
                }
                fns.insert(name, f);
            }
        }
        Ok(Self { fns, data })
    }

    pub(super) fn expand_macro(&self, macro_name: &str) -> Result<Vec<Method>, String> {
        let f = self
            .fns
            .get(macro_name)
            .ok_or_else(|| format!("style macro `{macro_name}` not found in styles.rs"))?;
        let mut collector = Collector {
            model: self,
            methods: Vec::new(),
            visited: BTreeSet::new(),
            family: macro_name.trim_end_matches("_style_methods").to_owned(),
        };
        collector.visit_item_fn(f);
        if collector.methods.is_empty() {
            return Err(format!(
                "style macro `{macro_name}` produced no methods; styles.rs layout changed?"
            ));
        }
        Ok(collector.methods)
    }

    fn entries(&self, call: &syn::Expr) -> Option<&Vec<DataEntry>> {
        let name = call_name(call)?;
        self.data.get(&name)
    }
}

struct Collector<'a> {
    model: &'a StylesModel,
    methods: Vec<Method>,
    visited: BTreeSet<String>,
    family: String,
}

impl Collector<'_> {
    fn push_box_family(&mut self, prefixes: &[DataEntry], suffixes: &[DataEntry]) {
        // Mirrors `generate_box_style_methods`.
        for p in prefixes {
            self.push_setter(&p.value);
            for s in suffixes {
                if s.value != "auto" || p.auto_allowed {
                    self.push_preset(&p.value, &s.value, false);
                }
                if s.value != "auto" {
                    self.push_preset(&p.value, &s.value, true);
                }
            }
        }
    }

    fn push_setter(&mut self, prefix: &str) {
        self.methods.push(Method {
            name: prefix.to_owned(),
            arity: 1,
            origin: Origin::FamilySetter {
                family: self.family.clone(),
                prefix: prefix.to_owned(),
            },
        });
    }

    fn push_preset(&mut self, prefix: &str, suffix: &str, negated: bool) {
        // Mirrors `generate_predefined_setter`.
        let neg = if negated { "_neg" } else { "" };
        let name = if suffix.is_empty() {
            format!("{prefix}{neg}")
        } else {
            format!("{prefix}{neg}_{suffix}")
        };
        self.methods.push(Method {
            name,
            arity: 0,
            origin: Origin::FamilyPreset {
                family: self.family.clone(),
                prefix: prefix.to_owned(),
                suffix: suffix.to_owned(),
                negated,
            },
        });
    }

    fn for_loop(&mut self, outer: &syn::ExprForLoop) -> bool {
        let Some(prefixes) = self.model.entries(&outer.expr).cloned() else {
            return false;
        };
        let mut has_setter = false;
        let mut inner_loops = Vec::new();
        find_in_block(&outer.body, &mut |expr| match expr {
            syn::Expr::Call(call)
                if call_name_of(&call.func).as_deref() == Some("generate_custom_value_setter") =>
            {
                has_setter = true;
            }
            syn::Expr::ForLoop(inner) => inner_loops.push(inner.clone()),
            _ => {}
        });

        for p in &prefixes {
            if has_setter {
                self.push_setter(&p.value);
            }
            for inner in &inner_loops {
                let Some(suffixes) = self.model.entries(&inner.expr).cloned() else {
                    continue;
                };
                let mut negate_flags = Vec::new();
                find_in_block(&inner.body, &mut |expr| {
                    if let syn::Expr::Call(call) = expr
                        && call_name_of(&call.func).as_deref() == Some("generate_predefined_setter")
                        && let Some(syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Bool(b),
                            ..
                        })) = call.args.iter().nth(5)
                    {
                        negate_flags.push(b.value);
                    }
                });
                for s in &suffixes {
                    for negated in &negate_flags {
                        self.push_preset(&p.value, &s.value, *negated);
                    }
                }
            }
        }
        true
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        if !self.for_loop(node) {
            syn::visit::visit_expr_for_loop(self, node);
        }
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        let name = call_name_of(&node.func);
        match name.as_deref() {
            Some("generate_box_style_methods") => {
                let mut args = node.args.iter();
                let prefixes = args.next().and_then(|a| self.model.entries(a)).cloned();
                let suffixes = args.next().and_then(|a| self.model.entries(a)).cloned();
                if let (Some(p), Some(s)) = (prefixes, suffixes) {
                    self.push_box_family(&p, &s);
                }
                return;
            }
            Some(other) => {
                // Follow helper functions such as `generate_methods()`.
                if !self.model.data.contains_key(other)
                    && self.visited.insert(other.to_owned())
                    && let Some(f) = self.model.fns.get(other)
                {
                    let f = f.clone();
                    self.visit_item_fn(&f);
                }
            }
            None => {}
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if node.path.is_ident("quote") {
            literal_fns(node.tokens.clone(), &mut self.methods);
        }
        syn::visit::visit_macro(self, node);
    }
}

/// Finds `fn name(...)` with a literal identifier inside `quote!` bodies.
pub(super) fn literal_fns(tokens: TokenStream, out: &mut Vec<Method>) {
    let tokens: Vec<TokenTree> = tokens.into_iter().collect();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Group(g) => literal_fns(g.stream(), out),
            TokenTree::Ident(kw) if kw == "fn" => {
                // `#method_name` interpolations are handled by the family model.
                if let Some(TokenTree::Ident(name)) = tokens.get(i + 1) {
                    {
                        let mut j = i + 2;
                        // Skip generics `<...>`.
                        if matches!(tokens.get(j), Some(TokenTree::Punct(p)) if p.as_char() == '<')
                        {
                            let mut depth = 0i32;
                            while j < tokens.len() {
                                if let TokenTree::Punct(p) = &tokens[j] {
                                    match p.as_char() {
                                        '<' => depth += 1,
                                        '>' => {
                                            depth -= 1;
                                            if depth == 0 {
                                                j += 1;
                                                break;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                j += 1;
                            }
                        }
                        if let Some(TokenTree::Group(params)) = tokens.get(j)
                            && params.delimiter() == Delimiter::Parenthesis
                        {
                            out.push(Method {
                                name: name.to_string(),
                                arity: count_params(params.stream()),
                                origin: Origin::Explicit,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
}

/// Counts non-receiver parameters in a `(...)` token stream.
pub(super) fn count_params(params: TokenStream) -> usize {
    let mut segments = vec![Vec::new()];
    let mut depth = 0i32;
    for tt in params {
        if let TokenTree::Punct(p) = &tt {
            match p.as_char() {
                '<' => depth += 1,
                '>' if depth > 0 => depth -= 1,
                ',' if depth == 0 => {
                    segments.push(Vec::new());
                    continue;
                }
                _ => {}
            }
        }
        segments.last_mut().expect("non-empty").push(tt);
    }
    segments
        .iter()
        .filter(|seg| !seg.is_empty())
        .filter(|seg| {
            !seg.iter()
                .any(|tt| matches!(tt, TokenTree::Ident(i) if i == "self"))
        })
        .count()
}

fn data_entries(f: &syn::ItemFn) -> Option<Vec<DataEntry>> {
    let tail = match f.block.stmts.last()? {
        syn::Stmt::Expr(expr, None) => expr,
        syn::Stmt::Macro(m) => {
            return parse_vec_macro(&m.mac);
        }
        _ => return None,
    };
    match tail {
        syn::Expr::Macro(m) => parse_vec_macro(&m.mac),
        _ => None,
    }
}

fn parse_vec_macro(mac: &syn::Macro) -> Option<Vec<DataEntry>> {
    if !mac.path.is_ident("vec") {
        return None;
    }
    let items = mac
        .parse_body_with(syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
        .ok()?;
    let mut entries = Vec::new();
    for item in items {
        let syn::Expr::Struct(s) = item else {
            return None;
        };
        let mut entry = DataEntry::default();
        let mut has_key = false;
        for field in &s.fields {
            let syn::Member::Named(name) = &field.member else {
                continue;
            };
            match (name.to_string().as_str(), &field.expr) {
                (
                    "prefix" | "suffix",
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(v),
                        ..
                    }),
                ) => {
                    entry.value = v.value();
                    has_key = true;
                }
                (
                    "auto_allowed",
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Bool(b),
                        ..
                    }),
                ) => entry.auto_allowed = b.value,
                _ => {}
            }
        }
        if !has_key {
            return None;
        }
        entries.push(entry);
    }
    Some(entries)
}

fn call_name(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Call(call) => call_name_of(&call.func),
        _ => None,
    }
}

fn call_name_of(func: &syn::Expr) -> Option<String> {
    match func {
        syn::Expr::Path(p) => p.path.get_ident().map(ToString::to_string),
        _ => None,
    }
}

fn find_in_block(block: &syn::Block, f: &mut dyn FnMut(&syn::Expr)) {
    struct Finder<'f>(&'f mut dyn FnMut(&syn::Expr));
    impl<'ast> Visit<'ast> for Finder<'_> {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            (self.0)(node);
            // Do not descend into nested loops; callers handle them explicitly.
            if !matches!(node, syn::Expr::ForLoop(_)) {
                syn::visit::visit_expr(self, node);
            }
        }
    }
    Finder(f).visit_block(block);
}
