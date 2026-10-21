//! Extracts the builder API surface of GPUI from its source tree.
//!
//! GPUI declares most `Styled` helpers through `gpui_macros::*_style_methods!()` invocations,
//! so plain trait parsing is not enough. Hand-written trait methods are read directly with
//! `syn`; macro-generated families are reconstructed from the data tables in
//! `gpui_macros/src/styles.rs` (prefix × suffix vectors) using the same rules the macro uses.

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use syn::visit::Visit;

/// Traits whose methods make up the RSX-facing API.
pub const TRAITS: &[(&str, &str)] = &[
    ("Styled", "src/styled.rs"),
    ("InteractiveElement", "src/elements/div.rs"),
    ("StatefulInteractiveElement", "src/elements/div.rs"),
    ("ParentElement", "src/element.rs"),
    ("FluentBuilder", "src/util.rs"),
];

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    /// Hand-written trait method or a literal `fn` inside a style macro.
    Explicit,
    /// `prefix(value)` setter generated for a family prefix.
    FamilySetter { family: String, prefix: String },
    /// `prefix[_neg]_suffix()` predefined setter generated for a family.
    FamilyPreset {
        family: String,
        prefix: String,
        suffix: String,
        negated: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Method {
    pub name: String,
    pub arity: usize,
    pub origin: Origin,
}

#[derive(Debug, Default)]
pub struct GpuiApi {
    /// Trait name → methods (sorted by name, deduplicated).
    pub traits: BTreeMap<String, Vec<Method>>,
    /// Complete explicit signatures plus the source templates for generated methods.
    pub signature_contract: String,
}

impl GpuiApi {
    pub fn load(gpui_root: &Path, macros_root: &Path) -> Result<Self, String> {
        let styles = StylesModel::load(&macros_root.join("src/styles.rs"))?;
        let mut api = GpuiApi::default();
        for (name, function) in &styles.fns {
            api.signature_contract.push_str(&format!(
                "style-template {name}: {}\n",
                function.to_token_stream()
            ));
        }

        for (trait_name, rel) in TRAITS {
            let path = gpui_root.join(rel);
            let file = parse_file(&path)?;
            let item = file
                .items
                .iter()
                .find_map(|item| match item {
                    syn::Item::Trait(t) if t.ident == trait_name => Some(t),
                    _ => None,
                })
                .ok_or_else(|| format!("trait `{trait_name}` not found in {}", path.display()))?;

            api.signature_contract.push_str(&trait_signatures(item));
            let mut methods = BTreeMap::new();
            for trait_item in &item.items {
                match trait_item {
                    syn::TraitItem::Fn(f) => {
                        let arity = f
                            .sig
                            .inputs
                            .iter()
                            .filter(|arg| matches!(arg, syn::FnArg::Typed(_)))
                            .count();
                        insert(
                            &mut methods,
                            Method {
                                name: f.sig.ident.to_string(),
                                arity,
                                origin: Origin::Explicit,
                            },
                        );
                    }
                    syn::TraitItem::Macro(m) => {
                        let macro_name = m
                            .mac
                            .path
                            .segments
                            .last()
                            .map(|s| s.ident.to_string())
                            .unwrap_or_default();
                        for method in styles.expand_macro(&macro_name)? {
                            insert(&mut methods, method);
                        }
                    }
                    _ => {}
                }
            }
            api.traits
                .insert((*trait_name).to_owned(), methods.into_values().collect());
        }

        Ok(api)
    }

    /// `name/arity` lines used by the API snapshot fixtures.
    pub fn fixture(&self, trait_name: &str) -> String {
        let mut out = String::new();
        for m in &self.traits[trait_name] {
            out.push_str(&format!("{}/{}\n", m.name, m.arity));
        }
        out
    }
}

fn trait_signatures(item: &syn::ItemTrait) -> String {
    let mut out = format!(
        "trait {}: {}\n",
        item.ident,
        item.supertraits.to_token_stream()
    );
    out = out.trim_end().to_owned();
    out.push('\n');
    for member in &item.items {
        match member {
            syn::TraitItem::Fn(method) => {
                out.push_str(&format!("{}\n", method.sig.to_token_stream()))
            }
            syn::TraitItem::Type(ty) => out.push_str(&format!("{}\n", ty.to_token_stream())),
            _ => {}
        }
    }
    out
}

fn insert(methods: &mut BTreeMap<String, Method>, method: Method) {
    // Keep the first definition; GPUI does not overload names inside one trait.
    methods.entry(method.name.clone()).or_insert(method);
}

fn parse_file(path: &Path) -> Result<syn::File, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    syn::parse_file(&source).map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

#[derive(Clone, Debug, Default)]
struct DataEntry {
    value: String,
    auto_allowed: bool,
}

/// Model of `gpui_macros/src/styles.rs`.
struct StylesModel {
    fns: BTreeMap<String, syn::ItemFn>,
    /// Data functions returning `vec![Struct { prefix|suffix: "..", .. }]`.
    data: BTreeMap<String, Vec<DataEntry>>,
}

impl StylesModel {
    fn load(path: &Path) -> Result<Self, String> {
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

    fn expand_macro(&self, macro_name: &str) -> Result<Vec<Method>, String> {
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
fn literal_fns(tokens: TokenStream, out: &mut Vec<Method>) {
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
fn count_params(params: TokenStream) -> usize {
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

/// Locates the GPUI and gpui_macros source roots.
///
/// Priority: explicit path → `GPUI_SOURCE_ROOT` → `cargo metadata` of the demo manifest.
pub fn locate_sources(
    explicit: Option<PathBuf>,
    workspace: &Path,
) -> Result<(PathBuf, PathBuf), String> {
    locate_sources_with(explicit, workspace, true)
}

/// Like [`locate_sources`]; with `allow_network = false` only cached sources are used.
pub fn locate_sources_with(
    explicit: Option<PathBuf>,
    workspace: &Path,
    allow_network: bool,
) -> Result<(PathBuf, PathBuf), String> {
    let override_root =
        explicit.or_else(|| std::env::var_os("GPUI_SOURCE_ROOT").map(PathBuf::from));
    let metadata = if override_root.is_none() {
        Some(demo_metadata(workspace, allow_network)?)
    } else {
        None
    };
    let gpui_root = match override_root {
        Some(root) => root,
        None => package_root(metadata.as_ref().unwrap(), "gpui")?,
    };
    // Git checkouts have sibling crates. Published snapshots are independent
    // registry directories, so resolve their macros from the same locked graph.
    let macros_root = std::env::var_os("GPUI_MACROS_SOURCE_ROOT")
        .map(PathBuf::from)
        .or_else(|| {
            gpui_root
                .parent()
                .map(|crates| crates.join("gpui_macros"))
                .filter(|p| p.join("src/styles.rs").exists())
        });
    let macros_root = match macros_root {
        Some(root) => root,
        None => {
            let metadata = match metadata {
                Some(data) => data,
                None => demo_metadata(workspace, allow_network)?,
            };
            package_root(&metadata, "gpui_macros")?
        }
    };
    if !macros_root.join("src/styles.rs").is_file() {
        return Err(format!(
            "GPUI style macros not found in {}; set GPUI_MACROS_SOURCE_ROOT",
            macros_root.display()
        ));
    }
    Ok((gpui_root, macros_root))
}

pub fn metadata_package_root(
    workspace: &Path,
    package: &str,
    allow_network: bool,
) -> Result<PathBuf, String> {
    package_root(&demo_metadata(workspace, allow_network)?, package)
}

fn package_root(metadata: &serde_json::Value, package: &str) -> Result<PathBuf, String> {
    let names: &[&str] = match package {
        "gpui" => &["gpui-pre", "gpui"],
        "gpui_macros" => &["gpui-pre-macros", "gpui_macros"],
        _ => &[package],
    };
    let packages: Vec<_> = metadata["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| names.iter().any(|name| p["name"] == *name))
        .collect();
    if packages.len() != 1 {
        return Err(format!(
            "expected one `{package}` package in demo metadata, found {}",
            packages.len()
        ));
    }
    let manifest = packages[0]["manifest_path"]
        .as_str()
        .ok_or("missing package manifest_path")?;
    Path::new(manifest)
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| format!("invalid manifest path: {manifest}"))
}

pub fn demo_metadata(workspace: &Path, allow_network: bool) -> Result<serde_json::Value, String> {
    let manifest = workspace.join("demo/Cargo.toml");
    let mut last_error = String::new();
    let modes: &[bool] = if allow_network {
        &[true, false]
    } else {
        &[true]
    };
    for &offline in modes {
        let mut cmd =
            std::process::Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
        cmd.args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--manifest-path",
        ])
        .arg(&manifest);
        if offline {
            cmd.arg("--offline");
        }
        let output = cmd
            .output()
            .map_err(|e| format!("failed to run cargo metadata: {e}"))?;
        if !output.status.success() {
            last_error = String::from_utf8_lossy(&output.stderr).into_owned();
            continue;
        }
        return serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("invalid cargo metadata output: {e}"));
    }
    Err(format!(
        "could not resolve the demo dependency graph via cargo metadata: {last_error}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_snapshot_sources_instead_of_the_kit_facade_and_rejects_ambiguity() {
        let mut metadata = serde_json::json!({"packages": [
            {"name": "gpui-kit", "manifest_path": "/registry/gpui-kit-0.8.0/Cargo.toml"},
            {"name": "gpui-pre", "manifest_path": "/registry/gpui-pre-0.3.9/Cargo.toml"},
            {"name": "gpui-pre-macros", "manifest_path": "/registry/gpui-pre-macros-0.3.9/Cargo.toml"},
        ]});
        assert_eq!(
            package_root(&metadata, "gpui").unwrap(),
            Path::new("/registry/gpui-pre-0.3.9")
        );
        assert_eq!(
            package_root(&metadata, "gpui_macros").unwrap(),
            Path::new("/registry/gpui-pre-macros-0.3.9")
        );
        metadata["packages"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "name": "gpui", "manifest_path": "/git/zed/crates/gpui/Cargo.toml",
            }));
        assert!(
            package_root(&metadata, "gpui")
                .unwrap_err()
                .contains("found 2")
        );
    }

    #[test]
    fn signatures_detect_type_and_generic_bound_changes_with_equal_arity() {
        let a: syn::ItemTrait =
            syn::parse_quote! { trait A { fn set<T: Into<Hsla>>(self, value:T)->Self; } };
        let b: syn::ItemTrait =
            syn::parse_quote! { trait A { fn set<T: Into<Pixels>>(self, value:T)->Self; } };
        assert_ne!(trait_signatures(&a), trait_signatures(&b));
    }

    #[test]
    fn counts_params_without_receiver() {
        let params: TokenStream = "mut self, a: impl Into<X<Y, Z>>, b: C".parse().unwrap();
        assert_eq!(count_params(params), 2);
        let params: TokenStream = "mut self".parse().unwrap();
        assert_eq!(count_params(params), 0);
    }

    #[test]
    fn finds_literal_quote_fns() {
        let tokens: TokenStream =
            "#[doc = \"x\"] #visibility fn relative(mut self) -> Self { self } #visibility fn #name(mut self) {} fn border_color<C>(mut self, c: C) -> Self where C: Into<Hsla> { self }"
                .parse()
                .unwrap();
        let mut out = Vec::new();
        literal_fns(tokens, &mut out);
        let names: Vec<_> = out.iter().map(|m| (m.name.as_str(), m.arity)).collect();
        assert_eq!(names, [("relative", 0), ("border_color", 1)]);
    }
}
