//! Supported state variants and their style refinement closures.

use super::suggestions::suggest;
use proc_macro2::{Span, TokenStream};
use quote::quote;

/// Tailwind 状态前缀对应的 GPUI 样式细化方法。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    /// `hover`、`focus`、`focus_visible`、`active`、`group_hover`、`group_active`。
    pub method: &'static str,
    /// `group-*` 前缀的组名（`group-hover:` 为 `""`，`group-hover/menu:` 为 `"menu"`）。
    pub group: Option<String>,
}

impl Variant {
    pub(super) fn call(&self, inner: &[TokenStream]) -> TokenStream {
        let method = syn::Ident::new(self.method, Span::call_site());
        match &self.group {
            Some(group) => quote! { .#method(#group, |__style| __style #(#inner)*) },
            None => quote! { .#method(|__style| __style #(#inner)*) },
        }
    }
}

pub const SUPPORTED_VARIANTS: &[&str] = &[
    "hover",
    "focus",
    "focus-visible",
    "active",
    "group-hover",
    "group-active",
];

const RESPONSIVE_VARIANTS: &[&str] = &["sm", "md", "lg", "xl", "2xl"];

pub(super) fn parse_variant(prefix: &str) -> Result<Variant, String> {
    let (name, group) = match prefix.split_once('/') {
        Some((name, group)) => (name, Some(group)),
        None => (prefix, None),
    };
    let variant = |method| Variant {
        method,
        group: None,
    };
    let grouped = |method: &'static str| Variant {
        method,
        group: Some(group.unwrap_or("").to_owned()),
    };
    match (name, group) {
        ("hover", None) => Ok(variant("hover")),
        ("focus", None) => Ok(variant("focus")),
        ("focus-visible", None) => Ok(variant("focus_visible")),
        ("active", None) => Ok(variant("active")),
        ("group-hover", _) => Ok(grouped("group_hover")),
        ("group-active", _) => Ok(grouped("group_active")),
        ("dark", _) => Err(format!(
            "`{prefix}:` variants are not supported: GPUI has no color-scheme media query.\n\
             \x20 help: define the color as a theme token in rsx.toml (e.g. `bg-surface`), or use `when={{(is_dark, |el| ...)}}`"
        )),
        (name, _) if RESPONSIVE_VARIANTS.contains(&name) => Err(format!(
            "Responsive variant `{prefix}:` is not supported: GPUI windows have no CSS breakpoints.\n\
             \x20 help: compute the condition from the window size and use `when={{(is_wide, |el| ...)}}`"
        )),
        _ => {
            let hint = suggest(name, SUPPORTED_VARIANTS.iter().copied())
                .map(|s| format!(" Did you mean `{s}:`?"))
                .unwrap_or_default();
            Err(format!(
                "Unsupported variant `{prefix}:`.{hint}\n\
                 \x20 note: supported variants are hover:, focus:, focus-visible:, active:, group-hover:, group-active:"
            ))
        }
    }
}
