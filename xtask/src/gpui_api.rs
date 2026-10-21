//! Extracts the builder API surface of GPUI from its source tree.
//!
//! GPUI declares most `Styled` helpers through `gpui_macros::*_style_methods!()` invocations,
//! so plain trait parsing is not enough. Hand-written trait methods are read directly with
//! `syn`; macro-generated families are reconstructed from the data tables in
//! `gpui_macros/src/styles.rs` (prefix × suffix vectors) using the same rules the macro uses.

use quote::ToTokens;
use std::collections::BTreeMap;
use std::path::Path;

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

mod sources;
mod styles;

pub use sources::{demo_metadata, locate_sources, locate_sources_with, metadata_package_root};
use styles::StylesModel;

#[cfg(test)]
mod tests;
