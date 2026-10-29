//! Design tokens and themes loaded from `rsx.toml`.
//!
//! `rsx.toml` lives next to the crate's `Cargo.toml` and is read at macro-expansion time:
//!
//! ```toml
//! [options]
//! spacing-scale = "tailwind"      # or "px" (default); see `SpacingScale`
//! preset = "gpui-component"       # semantic colors from gpui-component's Theme
//! context = "cx"                  # expression used by `preset` (default `cx`)
//!
//! [colors]
//! primary = "#2563eb"                         # static: bg-primary → .bg(rgb(0x2563eb))
//! surface = { expr = "cx.theme().background" } # runtime: re-read on every render
//!
//! [radius]
//! card = "12px"                    # rounded-card → .rounded(px(12.0))
//!
//! [spacing]
//! gutter = "18px"                  # p-gutter, gap-gutter, w-gutter, ...
//!
//! [text-size]
//! body = "14px"                    # text-size-body → .text_size(px(14.0))
//!
//! [line-height]
//! body = "20px"                    # leading-body → .line_height(px(20.0))
//!
//! [opacity]
//! muted = "60%"                    # opacity-muted → .opacity(0.6)
//!
//! [shadow]
//! card = { expr = "cx.theme().shadow" } # shadow-card → .shadow(...)
//! ```
//!
//! Only a small TOML subset is accepted (sections, `key = "string"`, `key = { expr = "..." }`
//! and `#` comments) so the macro does not need a TOML dependency.

mod builder;
mod cache;
mod parse;
#[cfg(test)]
mod tests;

pub use builder::ThemeBuilder;

use crate::codegen::class::SpacingScale;
use crate::codegen::tables::{GPUI_COMPONENT_THEME_COLORS, LENGTH_CLASS_SPECS, split_length_class};
use crate::{StyleOperation, StyleValue};
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// File name looked up in `CARGO_MANIFEST_DIR`.
pub const CONFIG_FILE: &str = "rsx.toml";

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum TokenValue {
    /// `0xRRGGBB` (`alpha == false`) or `0xRRGGBBAA`.
    Color { hex: u32, alpha: bool },
    /// A GPUI length expression such as `px(12.0)` or `rems(0.75)`.
    Length(String),
    /// An arbitrary Rust expression evaluated at render time.
    Expr(String),
    /// A normalized opacity in the inclusive range `0.0..=1.0`.
    Number(f32),
}

/// The token section that owns a theme class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ThemeTokenKind {
    Color,
    Radius,
    Spacing,
    TextSize,
    LineHeight,
    Opacity,
    Shadow,
}

impl ThemeTokenKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Color => "colors",
            Self::Radius => "radius",
            Self::Spacing => "spacing",
            Self::TextSize => "text-size",
            Self::LineHeight => "line-height",
            Self::Opacity => "opacity",
            Self::Shadow => "shadow",
        }
    }
}

/// A configured token and its source value, useful to tooling integrations.
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeToken {
    pub name: String,
    pub kind: ThemeTokenKind,
    pub value: TokenValue,
}

/// Explanation of a theme class expansion.
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeExplanation {
    pub class: String,
    pub token: ThemeToken,
    pub expansion: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Theme {
    /// Source file, used for rebuild tracking. `None` for in-memory themes.
    path: Option<PathBuf>,
    scale: Option<SpacingScale>,
    dynamic_runtime: Option<String>,
    colors: BTreeMap<String, TokenValue>,
    radius: BTreeMap<String, TokenValue>,
    spacing: BTreeMap<String, TokenValue>,
    text_size: BTreeMap<String, TokenValue>,
    line_height: BTreeMap<String, TokenValue>,
    opacity: BTreeMap<String, TokenValue>,
    shadow: BTreeMap<String, TokenValue>,
}

impl Theme {
    /// Creates an empty in-memory theme.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a theme builder.
    pub fn builder() -> ThemeBuilder {
        ThemeBuilder::default()
    }

    /// Returns the optional spacing scale override.
    pub const fn scale(&self) -> Option<SpacingScale> {
        self.scale
    }

    /// Module used for shared dynamic class helpers, when explicitly configured.
    pub fn dynamic_runtime(&self) -> Option<&str> {
        self.dynamic_runtime.as_deref()
    }

    /// Returns the source path when this theme was loaded from `rsx.toml`.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Returns configured color tokens.
    pub fn colors(&self) -> &BTreeMap<String, TokenValue> {
        &self.colors
    }

    /// Returns configured radius tokens.
    pub fn radius(&self) -> &BTreeMap<String, TokenValue> {
        &self.radius
    }

    /// Returns configured spacing tokens.
    pub fn spacing(&self) -> &BTreeMap<String, TokenValue> {
        &self.spacing
    }

    /// Returns configured text-size tokens.
    pub fn text_size(&self) -> &BTreeMap<String, TokenValue> {
        &self.text_size
    }

    /// Returns configured line-height tokens.
    pub fn line_height(&self) -> &BTreeMap<String, TokenValue> {
        &self.line_height
    }

    /// Returns configured opacity tokens.
    pub fn opacity(&self) -> &BTreeMap<String, TokenValue> {
        &self.opacity
    }

    /// Returns configured shadow tokens.
    pub fn shadow(&self) -> &BTreeMap<String, TokenValue> {
        &self.shadow
    }

    /// Expands a theme-token class (`bg-primary`, `rounded-card`, `p-gutter`).
    pub fn expand_class(&self, class: &str) -> Option<TokenStream> {
        self.style_class(class)
            .map(|operation| operation.to_tokens())
    }

    /// Resolves a theme class without executing runtime token expressions.
    pub fn style_class(&self, class: &str) -> Option<StyleOperation> {
        let (_, _, value, method) = self.class_token(class)?;
        Some(StyleOperation::new(
            method,
            Some(match value {
                TokenValue::Color { hex, alpha } => StyleValue::Color {
                    hex: *hex,
                    alpha: *alpha,
                },
                TokenValue::Number(number) => StyleValue::Number(*number),
                TokenValue::Length(expr) | TokenValue::Expr(expr) => {
                    StyleValue::Expression(expr.clone())
                }
            }),
        ))
    }

    /// All classes this theme adds, for completion, suggestions and the spec.
    pub fn class_names(&self) -> Vec<String> {
        let mut out = Vec::new();
        for name in self.colors.keys() {
            for prefix in ["bg-", "text-", "border-"] {
                out.push(format!("{prefix}{name}"));
            }
        }
        for name in self.radius.keys() {
            out.push(format!("rounded-{name}"));
        }
        // Every length prefix `expand_class` accepts (`pt-`, `min-w-`, ...), not a subset.
        for name in self.spacing.keys() {
            for spec in LENGTH_CLASS_SPECS {
                out.push(format!("{}{name}", spec.prefix));
            }
        }
        for name in self.text_size.keys() {
            out.push(format!("text-size-{name}"));
        }
        for name in self.line_height.keys() {
            out.push(format!("leading-{name}"));
        }
        for name in self.opacity.keys() {
            out.push(format!("opacity-{name}"));
        }
        for name in self.shadow.keys() {
            out.push(format!("shadow-{name}"));
        }
        out
    }

    /// Returns every configured token in deterministic section/name order.
    pub fn tokens(&self) -> Vec<ThemeToken> {
        let mut tokens = Vec::new();
        for (kind, values) in [
            (ThemeTokenKind::Color, &self.colors),
            (ThemeTokenKind::Radius, &self.radius),
            (ThemeTokenKind::Spacing, &self.spacing),
            (ThemeTokenKind::TextSize, &self.text_size),
            (ThemeTokenKind::LineHeight, &self.line_height),
            (ThemeTokenKind::Opacity, &self.opacity),
            (ThemeTokenKind::Shadow, &self.shadow),
        ] {
            tokens.extend(values.iter().map(|(name, value)| ThemeToken {
                name: name.clone(),
                kind,
                value: value.clone(),
            }));
        }
        tokens
    }

    /// Explains a class backed by this theme, including generated GPUI code.
    pub fn explain_class(&self, class: &str) -> Option<ThemeExplanation> {
        let base = class.rsplit(':').next().unwrap_or(class);
        let (kind, name, value, _) = self.class_token(base)?;
        let expansion = if class.contains(':') {
            let mode = crate::ClassMode::PERMISSIVE.with_theme(Some(Arc::new(self.clone())));
            if !matches!(
                crate::analyze_class(class, mode.clone()).outcome,
                crate::ClassOutcome::Supported(_)
            ) {
                return None;
            }
            let methods = crate::codegen::class::parse_class_string_with_mode(
                class,
                mode,
                proc_macro2::Span::call_site(),
            );
            quote!(#(#methods)*).to_string()
        } else {
            self.style_class(base)?.to_tokens().to_string()
        };
        Some(ThemeExplanation {
            class: class.to_owned(),
            token: ThemeToken {
                name: name.to_owned(),
                kind,
                value: value.clone(),
            },
            expansion,
        })
    }

    fn class_token(
        &self,
        class: &str,
    ) -> Option<(ThemeTokenKind, &str, &TokenValue, &'static str)> {
        for (prefix, method) in [
            ("bg-", "bg"),
            ("text-", "text_color"),
            ("border-", "border_color"),
        ] {
            if let Some(name) = class.strip_prefix(prefix)
                && let Some((name, value)) = self.colors.get_key_value(name)
            {
                return Some((ThemeTokenKind::Color, name, value, method));
            }
        }
        if let Some(name) = class.strip_prefix("rounded-")
            && let Some((name, value)) = self.radius.get_key_value(name)
        {
            return Some((ThemeTokenKind::Radius, name, value, "rounded"));
        }
        if let Some((method, name, _)) = split_length_class(class)
            && let Some((name, value)) = self.spacing.get_key_value(name)
        {
            return Some((ThemeTokenKind::Spacing, name, value, method));
        }
        for (prefix, values, kind, method) in [
            (
                "text-size-",
                &self.text_size,
                ThemeTokenKind::TextSize,
                "text_size",
            ),
            (
                "leading-",
                &self.line_height,
                ThemeTokenKind::LineHeight,
                "line_height",
            ),
            (
                "opacity-",
                &self.opacity,
                ThemeTokenKind::Opacity,
                "opacity",
            ),
            ("shadow-", &self.shadow, ThemeTokenKind::Shadow, "shadow"),
        ] {
            if let Some(name) = class.strip_prefix(prefix)
                && let Some((name, value)) = values.get_key_value(name)
            {
                return Some((kind, name, value, method));
            }
        }
        None
    }
}

/// `include_bytes!` of the config file so cargo rebuilds when it changes.
pub fn rebuild_tracker(theme: &Theme) -> Option<TokenStream> {
    let path = theme.path.as_ref()?.to_str()?.to_owned();
    Some(quote! { const _: &[u8] = include_bytes!(#path); })
}
