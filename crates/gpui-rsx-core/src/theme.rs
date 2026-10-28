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

    /// Parses `rsx.toml` contents.
    pub fn parse(source: &str) -> Result<Self, String> {
        let mut theme = Theme::default();
        let mut section = String::new();
        let mut preset = None;
        let mut context = "cx".to_owned();
        let mut keys = HashSet::new();

        for (index, raw_line) in source.lines().enumerate() {
            let line_no = index + 1;
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                section = name.trim().to_owned();
                if !matches!(
                    section.as_str(),
                    "options"
                        | "colors"
                        | "radius"
                        | "spacing"
                        | "text-size"
                        | "line-height"
                        | "opacity"
                        | "shadow"
                ) {
                    return Err(format!(
                        "rsx.toml:{line_no}: unknown section `[{section}]` (expected options, colors, radius, spacing, text-size, line-height, opacity or shadow)"
                    ));
                }
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("rsx.toml:{line_no}: expected `key = value`"))?;
            let key = key.trim().trim_matches('"').to_owned();
            if key.is_empty()
                || !key
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            {
                return Err(format!("rsx.toml:{line_no}: invalid key `{key}`"));
            }
            if !keys.insert((section.clone(), key.clone())) {
                return Err(format!(
                    "rsx.toml:{line_no}: duplicate key `{key}` in [{section}]"
                ));
            }
            let value =
                parse_value(value.trim()).map_err(|e| format!("rsx.toml:{line_no}: {e}"))?;

            match section.as_str() {
                "options" => {
                    let RawValue::Str(value) = value else {
                        return Err(format!("rsx.toml:{line_no}: `{key}` expects a string"));
                    };
                    match key.as_str() {
                        "spacing-scale" => {
                            theme.scale = Some(match value.as_str() {
                                "px" => SpacingScale::Px,
                                "tailwind" => SpacingScale::Tailwind,
                                other => {
                                    return Err(format!(
                                        "rsx.toml:{line_no}: spacing-scale must be \"px\" or \"tailwind\", found \"{other}\""
                                    ));
                                }
                            })
                        }
                        "preset" => preset = Some(value),
                        "context" => context = value,
                        other => {
                            return Err(format!(
                                "rsx.toml:{line_no}: unknown option `{other}` (expected spacing-scale, preset or context)"
                            ));
                        }
                    }
                }
                "colors" => {
                    let token = match value {
                        RawValue::Expr(expr) => TokenValue::Expr(validate_expr(&expr, line_no)?),
                        RawValue::Str(color) => parse_color(&color).ok_or_else(|| {
                            format!(
                                "rsx.toml:{line_no}: `{key}` is not a #rgb/#rrggbb/#rrggbbaa color"
                            )
                        })?,
                    };
                    theme.colors.insert(key, token);
                }
                "radius" | "spacing" | "text-size" | "line-height" => {
                    let token = match value {
                        RawValue::Expr(expr) => TokenValue::Expr(validate_expr(&expr, line_no)?),
                        RawValue::Str(length) => parse_length(&length).ok_or_else(|| {
                            format!("rsx.toml:{line_no}: `{key}` must be a length like \"12px\" or \"0.75rem\"")
                        })?,
                    };
                    let map = match section.as_str() {
                        "radius" => &mut theme.radius,
                        "spacing" => &mut theme.spacing,
                        "text-size" => &mut theme.text_size,
                        "line-height" => &mut theme.line_height,
                        _ => unreachable!("section was checked above"),
                    };
                    map.insert(key, token);
                }
                "opacity" => {
                    let token = match value {
                        RawValue::Expr(expr) => TokenValue::Expr(validate_expr(&expr, line_no)?),
                        RawValue::Str(opacity) => parse_opacity(&opacity).ok_or_else(|| {
                            format!(
                                "rsx.toml:{line_no}: `{key}` must be an opacity between 0 and 1, such as \"0.6\" or \"60%\""
                            )
                        })?,
                    };
                    theme.opacity.insert(key, token);
                }
                "shadow" => {
                    let RawValue::Expr(expr) = value else {
                        return Err(format!(
                            "rsx.toml:{line_no}: `{key}` expects an inline Rust expression like {{ expr = \"cx.theme().shadow\" }}"
                        ));
                    };
                    theme
                        .shadow
                        .insert(key, TokenValue::Expr(validate_expr(&expr, line_no)?));
                }
                _ => {
                    return Err(format!(
                        "rsx.toml:{line_no}: `{key}` must be inside a section"
                    ));
                }
            }
        }

        match preset.as_deref() {
            None => {}
            Some("gpui-component") => {
                validate_expr(&context, 0)?;
                for field in GPUI_COMPONENT_THEME_COLORS {
                    let name = field.replace('_', "-");
                    theme.colors.entry(name).or_insert_with(|| {
                        TokenValue::Expr(format!("{context}.theme().colors.{field}"))
                    });
                }
            }
            Some(other) => {
                return Err(format!(
                    "rsx.toml: unknown preset \"{other}\" (supported: \"gpui-component\")"
                ));
            }
        }

        theme.validate()?;
        Ok(theme)
    }

    /// Loads `rsx.toml` from `dir`, caching by canonical path and content fingerprint.
    ///
    /// Returns `Ok(None)` when the file does not exist.
    pub fn load_from_dir(dir: &Path) -> Result<Option<Arc<Theme>>, String> {
        static CACHE: OnceLock<Mutex<ThemeCache>> = OnceLock::new();

        let requested_path = dir.join(CONFIG_FILE);
        // Different relative paths can point at the same workspace config. Canonicalizing
        // avoids retaining duplicate entries and makes the cache bound meaningful.
        let cache_path =
            std::fs::canonicalize(&requested_path).unwrap_or_else(|_| requested_path.clone());
        let source = match std::fs::read_to_string(&cache_path) {
            Ok(source) => source,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "failed to read {}: {error}",
                    requested_path.display()
                ));
            }
        };
        let fingerprint = content_fingerprint(source.as_bytes());
        let cache = CACHE.get_or_init(Default::default);
        if let Ok(mut cache) = cache.lock()
            && let Some(theme) = cache.get(&cache_path, fingerprint)
        {
            return Ok(Some(theme));
        }

        let mut theme = Theme::parse(&source)?;
        theme.path = Some(cache_path.clone());
        let theme = Arc::new(theme);
        if let Ok(mut cache) = cache.lock() {
            cache.insert(cache_path, fingerprint, Arc::clone(&theme));
        }
        Ok(Some(theme))
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

    /// Validates in-memory token types, names and expressions before expansion.
    pub fn validate(&self) -> Result<(), String> {
        for token in self.tokens() {
            if token.name.is_empty()
                || !token
                    .name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            {
                return Err(format!("rsx.toml: invalid token name `{}`", token.name));
            }
            let valid = match &token.value {
                TokenValue::Expr(expr) => syn::parse_str::<syn::Expr>(expr).is_ok(),
                TokenValue::Color { hex, alpha } => {
                    token.kind == ThemeTokenKind::Color && (*alpha || *hex <= 0xffffff)
                }
                TokenValue::Length(expr) => {
                    matches!(
                        token.kind,
                        ThemeTokenKind::Radius
                            | ThemeTokenKind::Spacing
                            | ThemeTokenKind::TextSize
                            | ThemeTokenKind::LineHeight
                    ) && syn::parse_str::<syn::Expr>(expr).is_ok()
                }
                TokenValue::Number(value) => {
                    token.kind == ThemeTokenKind::Opacity
                        && value.is_finite()
                        && (0.0..=1.0).contains(value)
                }
            };
            if !valid {
                return Err(format!(
                    "rsx.toml: invalid value for [{}] token `{}`",
                    token.kind.as_str(),
                    token.name
                ));
            }
        }
        if self
            .text_size
            .keys()
            .any(|name| self.colors.contains_key(&format!("size-{name}")))
        {
            return Err(
                "rsx.toml: a text-size class conflicts with a color token named `size-*`".into(),
            );
        }
        Ok(())
    }

    /// Explains a class backed by this theme, including generated GPUI code.
    pub fn explain_class(&self, class: &str) -> Option<ThemeExplanation> {
        let base = class.rsplit(':').next().unwrap_or(class);
        let (kind, name, value, method) = self.class_token(base)?;
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
            method_call(method, value_tokens(value)).to_string()
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

const MAX_THEME_CACHE_ENTRIES: usize = 64;

#[derive(Default)]
struct ThemeCache {
    entries: HashMap<PathBuf, (u64, Arc<Theme>)>,
    order: VecDeque<PathBuf>,
}

impl ThemeCache {
    fn get(&mut self, path: &Path, fingerprint: u64) -> Option<Arc<Theme>> {
        let (cached_fingerprint, theme) = self.entries.get(path)?;
        if *cached_fingerprint != fingerprint {
            return None;
        }
        let theme = Arc::clone(theme);
        self.touch(path);
        Some(theme)
    }

    fn insert(&mut self, path: PathBuf, fingerprint: u64, theme: Arc<Theme>) {
        self.entries.insert(path.clone(), (fingerprint, theme));
        self.touch(&path);
        while self.entries.len() > MAX_THEME_CACHE_ENTRIES {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if oldest != path {
                self.entries.remove(&oldest);
            }
        }
    }

    fn touch(&mut self, path: &Path) {
        self.order.retain(|cached| cached != path);
        self.order.push_back(path.to_owned());
    }
}

fn content_fingerprint(source: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}

/// Builder for in-memory [`Theme`] values used by tools and integrations.
#[derive(Clone, Debug, Default)]
pub struct ThemeBuilder {
    theme: Theme,
}

impl ThemeBuilder {
    pub fn scale(mut self, scale: SpacingScale) -> Self {
        self.theme.scale = Some(scale);
        self
    }

    pub fn color(mut self, name: impl Into<String>, value: TokenValue) -> Self {
        self.theme.colors.insert(name.into(), value);
        self
    }

    pub fn radius(mut self, name: impl Into<String>, value: TokenValue) -> Self {
        self.theme.radius.insert(name.into(), value);
        self
    }

    pub fn spacing(mut self, name: impl Into<String>, value: TokenValue) -> Self {
        self.theme.spacing.insert(name.into(), value);
        self
    }

    pub fn text_size(mut self, name: impl Into<String>, value: TokenValue) -> Self {
        self.theme.text_size.insert(name.into(), value);
        self
    }

    pub fn line_height(mut self, name: impl Into<String>, value: TokenValue) -> Self {
        self.theme.line_height.insert(name.into(), value);
        self
    }

    pub fn opacity(mut self, name: impl Into<String>, value: f32) -> Self {
        self.theme
            .opacity
            .insert(name.into(), TokenValue::Number(value));
        self
    }

    pub fn shadow(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.theme
            .shadow
            .insert(name.into(), TokenValue::Expr(value.into()));
        self
    }

    pub fn build(self) -> Theme {
        self.theme
    }

    /// Builds a theme after validating token names, types and expression syntax.
    pub fn try_build(self) -> Result<Theme, String> {
        self.theme.validate()?;
        Ok(self.theme)
    }
}

/// `include_bytes!` of the config file so cargo rebuilds when it changes.
pub fn rebuild_tracker(theme: &Theme) -> Option<TokenStream> {
    let path = theme.path.as_ref()?.to_str()?.to_owned();
    Some(quote! { const _: &[u8] = include_bytes!(#path); })
}

enum RawValue {
    Str(String),
    Expr(String),
}

fn parse_value(value: &str) -> Result<RawValue, String> {
    if let Some(inner) = value.strip_prefix('{').and_then(|v| v.strip_suffix('}')) {
        let (key, rest) = inner
            .split_once('=')
            .ok_or("inline tables must look like { expr = \"...\" }")?;
        if key.trim() != "expr" {
            return Err(format!(
                "unknown inline key `{}` (expected `expr`)",
                key.trim()
            ));
        }
        return parse_string(rest.trim()).map(RawValue::Expr);
    }
    parse_string(value).map(RawValue::Str)
}

fn parse_string(value: &str) -> Result<String, String> {
    let inner = value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .ok_or_else(|| format!("expected a quoted string, found `{value}`"))?;
    Ok(inner.replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;
    for (index, c) in line.char_indices() {
        match c {
            // `\"` inside a string (which `parse_string` unescapes) does not close it.
            _ if escaped => escaped = false,
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

fn validate_expr(expr: &str, line_no: usize) -> Result<String, String> {
    syn::parse_str::<syn::Expr>(expr)
        .map(|_| expr.to_owned())
        .map_err(|e| format!("rsx.toml:{line_no}: `{expr}` is not a Rust expression: {e}"))
}

fn parse_color(color: &str) -> Option<TokenValue> {
    let hex = color
        .strip_prefix('#')
        .filter(|hex| hex.bytes().all(|b| b.is_ascii_hexdigit()))?;
    let digits = |s: &str| u32::from_str_radix(s, 16).ok();
    match hex.len() {
        6 => Some(TokenValue::Color {
            hex: digits(hex)?,
            alpha: false,
        }),
        8 => Some(TokenValue::Color {
            hex: digits(hex)?,
            alpha: true,
        }),
        3 => {
            let v = digits(hex)?;
            let (r, g, b) = ((v >> 8) & 0xf, (v >> 4) & 0xf, v & 0xf);
            Some(TokenValue::Color {
                hex: (r * 0x11) << 16 | (g * 0x11) << 8 | (b * 0x11),
                alpha: false,
            })
        }
        _ => None,
    }
}

fn parse_length(length: &str) -> Option<TokenValue> {
    let number = |raw: &str| raw.parse::<f32>().ok().filter(|n| n.is_finite());
    if let Some(raw) = length.strip_suffix("px") {
        return Some(TokenValue::Length(format!("px({:?})", number(raw)?)));
    }
    if let Some(raw) = length.strip_suffix("rem") {
        return Some(TokenValue::Length(format!("rems({:?})", number(raw)?)));
    }
    None
}

fn parse_opacity(opacity: &str) -> Option<TokenValue> {
    let value = if let Some(raw) = opacity.strip_suffix('%') {
        raw.parse::<f32>().ok()? / 100.0
    } else {
        opacity.parse::<f32>().ok()?
    };
    value
        .is_finite()
        .then_some(value)
        .filter(|value| (0.0..=1.0).contains(value))
        .map(TokenValue::Number)
}

fn color_tokens(value: &TokenValue) -> TokenStream {
    match value {
        TokenValue::Color { hex, alpha: false } => quote! { rgb(#hex) },
        TokenValue::Color { hex, alpha: true } => quote! { rgba(#hex) },
        other => value_tokens(other),
    }
}

fn value_tokens(value: &TokenValue) -> TokenStream {
    match value {
        TokenValue::Length(expr) | TokenValue::Expr(expr) => expr.parse().unwrap_or_else(|error| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("invalid theme expression: {error}"),
            )
            .to_compile_error()
        }),
        TokenValue::Color { .. } => color_tokens(value),
        TokenValue::Number(number) if number.is_finite() => quote! { #number },
        TokenValue::Number(_) => syn::Error::new(
            proc_macro2::Span::call_site(),
            "theme number must be finite",
        )
        .to_compile_error(),
    }
}

fn method_call(method: &str, value: TokenStream) -> TokenStream {
    let ident = syn::Ident::new(method, proc_macro2::Span::call_site());
    quote! { .#ident(#value) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tokens_and_options() {
        let theme = Theme::parse(
            r##"
            # design tokens
            [options]
            spacing-scale = "tailwind"

            [colors]
            primary = "#2563eb"
            overlay = "#00000080"   # alpha
            surface = { expr = "cx.theme().background" }

            [radius]
            card = "12px"

            [spacing]
            gutter = "1.5rem"
            "##,
        )
        .unwrap();
        assert_eq!(theme.scale, Some(SpacingScale::Tailwind));
        assert_eq!(
            theme.expand_class("bg-primary").unwrap().to_string(),
            quote! { .bg(rgb(2450411u32)) }.to_string()
        );
        assert!(
            theme
                .expand_class("text-overlay")
                .unwrap()
                .to_string()
                .contains("rgba")
        );
        assert_eq!(
            theme.expand_class("border-surface").unwrap().to_string(),
            quote! { .border_color(cx.theme().background) }.to_string()
        );
        assert_eq!(
            theme.expand_class("rounded-card").unwrap().to_string(),
            quote! { .rounded(px(12.0)) }.to_string()
        );
        assert_eq!(
            theme.expand_class("gap-x-gutter").unwrap().to_string(),
            quote! { .gap_x(rems(1.5)) }.to_string()
        );
        assert!(theme.expand_class("bg-unknown").is_none());
    }

    #[test]
    fn parses_extended_token_sections_and_explains_classes() {
        let theme = Theme::parse(
            "[text-size]\nbody = \"14px\"\n[line-height]\nbody = \"20px\"\n[opacity]\nmuted = \"60%\"\n[shadow]\ncard = { expr = \"cx.theme().shadow\" }\n",
        )
        .unwrap();
        assert_eq!(theme.text_size().len(), 1);
        assert_eq!(theme.line_height().len(), 1);
        assert_eq!(theme.opacity().get("muted"), Some(&TokenValue::Number(0.6)));
        assert_eq!(theme.shadow().len(), 1);
        for class in [
            "text-size-body",
            "leading-body",
            "opacity-muted",
            "shadow-card",
        ] {
            assert!(theme.expand_class(class).is_some(), "{class}");
            assert!(theme.class_names().contains(&class.to_owned()), "{class}");
        }
        let explanation = theme.explain_class("hover:opacity-muted").unwrap();
        assert_eq!(explanation.token.kind, ThemeTokenKind::Opacity);
        assert_eq!(explanation.token.name, "muted");
        assert!(explanation.expansion.contains("opacity"));
        assert_eq!(theme.tokens().len(), 4);
        assert!(theme.explain_class("dark:opacity-muted").is_none());
        assert!(explanation.expansion.contains("hover"));
    }

    #[test]
    fn validates_in_memory_themes_and_rejects_ambiguous_configs() {
        assert!(
            Theme::builder()
                .opacity("muted", f32::NAN)
                .try_build()
                .is_err()
        );
        assert!(Theme::builder().shadow("card", "cx.(").try_build().is_err());
        assert!(Theme::builder().opacity("muted", 0.6).try_build().is_ok());
        assert!(
            Theme::parse("[colors]\nbrand=\"#123456\"\nbrand=\"#654321\"\n")
                .unwrap_err()
                .contains("duplicate key")
        );
        assert!(
            Theme::parse("[colors]\nsize-body=\"#123456\"\n[text-size]\nbody=\"14px\"\n")
                .unwrap_err()
                .contains("conflicts")
        );
    }

    #[test]
    fn cache_evicts_the_least_recently_used_entry() {
        let mut cache = ThemeCache::default();
        for i in 0..MAX_THEME_CACHE_ENTRIES {
            cache.insert(PathBuf::from(i.to_string()), 0, Arc::new(Theme::new()));
        }
        assert!(cache.get(Path::new("0"), 0).is_some());
        cache.insert(PathBuf::from("new"), 0, Arc::new(Theme::new()));
        assert_eq!(cache.entries.len(), MAX_THEME_CACHE_ENTRIES);
        assert!(cache.get(Path::new("1"), 0).is_none());
        assert!(cache.get(Path::new("0"), 0).is_some());
    }

    #[test]
    fn gpui_component_preset_maps_semantic_colors() {
        let theme =
            Theme::parse("[options]\npreset = \"gpui-component\"\ncontext = \"app\"\n").unwrap();
        assert_eq!(
            theme
                .expand_class("bg-primary-foreground")
                .unwrap()
                .to_string(),
            quote! { .bg(app.theme().colors.primary_foreground) }.to_string()
        );
        // Theme.list is ListSettings; the color must bypass Theme's own fields.
        assert_eq!(
            theme.expand_class("bg-list").unwrap().to_string(),
            quote! { .bg(app.theme().colors.list) }.to_string()
        );
        assert!(
            theme
                .class_names()
                .contains(&"text-muted-foreground".to_owned())
        );
    }

    #[test]
    fn reports_errors_with_line_numbers() {
        let err = Theme::parse("[colors]\nprimary = \"blue\"\n").unwrap_err();
        assert!(err.starts_with("rsx.toml:2:"), "{err}");
        let err = Theme::parse("[colors]\nx = { expr = \"cx.(\" }\n").unwrap_err();
        assert!(err.contains("not a Rust expression"), "{err}");
        let err = Theme::parse("[themes]\n").unwrap_err();
        assert!(err.contains("unknown section"), "{err}");
    }

    #[test]
    fn escaped_quotes_do_not_end_strings_before_comments() {
        let theme = Theme::parse(
            r##"
            [colors]
            brand = { expr = "f(\"#a\")" } # trailing comment
            "##,
        )
        .unwrap();
        assert!(matches!(
            theme.colors.get("brand"),
            Some(TokenValue::Expr(expr)) if expr == r##"f("#a")"##
        ));
    }

    #[test]
    fn class_names_cover_every_length_prefix() {
        let theme = Theme::parse("[spacing]\ngutter = \"12px\"\n").unwrap();
        let names = theme.class_names();
        for class in ["pt-gutter", "min-w-gutter", "gap-x-gutter", "p-gutter"] {
            assert!(names.iter().any(|n| n == class), "{class}");
            assert!(theme.expand_class(class).is_some(), "{class}");
        }
    }

    #[test]
    fn class_names_are_derived_from_the_length_prefix_table() {
        let theme = Theme::parse("[spacing]\ngutter = \"12px\"\n").unwrap();
        let names = theme.class_names();
        for spec in LENGTH_CLASS_SPECS {
            let class = format!("{}{}", spec.prefix, "gutter");
            assert!(names.contains(&class), "{class} is missing from completion");
            assert!(
                theme.expand_class(&class).is_some(),
                "{class} does not expand"
            );
        }
        assert_eq!(
            names.iter().filter(|name| name.contains("gutter")).count(),
            LENGTH_CLASS_SPECS.len()
        );
    }

    #[test]
    fn color_values_require_only_hex_digits() {
        for color in ["#+12345", "#12345g", "#1234567", "#"] {
            let err = Theme::parse(&format!("[colors]\nbrand = \"{color}\"\n")).unwrap_err();
            assert!(
                err.contains("not a #rgb/#rrggbb/#rrggbbaa color"),
                "{color}: {err}"
            );
        }
        // All accepted spellings still parse, and alpha is tracked.
        let theme = Theme::parse(
            "[colors]\nshort = \"#abc\"\nplain = \"#123456\"\nwith-alpha = \"#12345678\"\n",
        )
        .unwrap();
        assert!(matches!(
            theme.colors.get("short"),
            Some(TokenValue::Color {
                hex: 0xaabbcc,
                alpha: false
            })
        ));
        assert!(matches!(
            theme.colors.get("with-alpha"),
            Some(TokenValue::Color {
                hex: 0x12345678,
                alpha: true
            })
        ));
        assert!(matches!(
            theme.colors.get("plain"),
            Some(TokenValue::Color {
                hex: 0x123456,
                alpha: false
            })
        ));
    }

    #[test]
    fn comments_after_escaped_backslashes_and_hashes_parse() {
        // A `\` inside a string escapes the next quote, so `#` after it is still a comment.
        let theme = Theme::parse(
            r##"
            [colors]
            first = { expr = "f(\"x\")" } # comment
            second = { expr = "g(\"#y\")" } # another
            "##,
        )
        .unwrap();
        assert_eq!(theme.colors.len(), 2, "{:?}", theme.colors);
        assert!(matches!(
            theme.colors.get("second"),
            Some(TokenValue::Expr(expr)) if expr == r##"g("#y")"##
        ));

        // A `#` inside a string is not a comment.
        let theme = Theme::parse("[colors]\nbrand = \"#123456\" # real comment\n").unwrap();
        assert_eq!(theme.colors.len(), 1);
    }

    #[test]
    fn rejects_malformed_lines_with_line_numbers() {
        let cases = [
            ("[colors]\nbrand blue\n", "expected `key = value`"),
            ("[colors]\n\"\" = \"#123456\"\n", "invalid key"),
            ("[colors]\nbr and = \"#123456\"\n", "invalid key"),
            (
                "[options]\nspacing-scale = \"ems\"\n",
                "must be \"px\" or \"tailwind\"",
            ),
            ("[options]\nnope = \"x\"\n", "unknown option `nope`"),
            (
                "[options]\nspacing-scale = { expr = \"x\" }\n",
                "expects a string",
            ),
            ("[radius]\ncard = \"big\"\n", "must be a length"),
            (
                "[opacity]\nmuted = \"120%\"\n",
                "must be an opacity between 0 and 1",
            ),
            (
                "[shadow]\ncard = \"shadow\"\n",
                "expects an inline Rust expression",
            ),
            (
                "[colors]\nbrand = { colour = \"x\" }\n",
                "unknown inline key",
            ),
            (
                "[colors]\nbrand = { expr }\n",
                "inline tables must look like",
            ),
            ("[colors]\nbrand = #123456\n", "expected a quoted string"),
            ("brand = \"#123456\"\n", "must be inside a section"),
            ("[options]\npreset = \"other\"\n", "unknown preset"),
            (
                "[options]\npreset = \"gpui-component\"\ncontext = \"cx.(\"\n",
                "not a Rust expression",
            ),
        ];
        for (source, expected) in cases {
            let error = Theme::parse(source).unwrap_err();
            assert!(error.contains(expected), "{source:?}: {error}");
            assert!(error.starts_with("rsx.toml"), "{source:?}: {error}");
        }
    }

    #[test]
    fn explicit_colors_override_preset_values() {
        let theme = Theme::parse(
            "[options]\npreset = \"gpui-component\"\n[colors]\nprimary = \"#123456\"\n",
        )
        .unwrap();
        assert!(matches!(
            theme.colors.get("primary"),
            Some(TokenValue::Color {
                hex: 0x123456,
                alpha: false
            })
        ));
        // Preset colors that were not overridden still resolve through the context.
        assert!(matches!(
            theme.colors.get("muted-foreground"),
            Some(TokenValue::Expr(_))
        ));
    }

    #[test]
    fn class_names_include_radius_tokens() {
        let theme = Theme::parse("[radius]\ncard = \"12px\"\n").unwrap();
        assert!(theme.class_names().contains(&"rounded-card".to_owned()));
    }

    #[test]
    fn load_from_dir_reads_once_and_tracks_the_path() {
        let root = std::env::temp_dir().join(format!("rsx-theme-load-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();

        // No config file: no theme, no error.
        assert!(Theme::load_from_dir(&root).unwrap().is_none());

        std::fs::write(root.join(CONFIG_FILE), "[colors]\nbrand = \"#123456\"\n").unwrap();
        let theme = Theme::load_from_dir(&root).unwrap().unwrap();
        assert_eq!(
            theme.path.as_deref(),
            Some(
                std::fs::canonicalize(root.join(CONFIG_FILE))
                    .unwrap()
                    .as_path()
            )
        );
        assert!(theme.expand_class("bg-brand").is_some());

        // A second call hits the cache and returns the same shared allocation.
        let again = Theme::load_from_dir(&root).unwrap().unwrap();
        assert!(std::sync::Arc::ptr_eq(&theme, &again));

        // Rebuild tracking points at the config file.
        let tracker = rebuild_tracker(&theme).unwrap().to_string();
        assert!(tracker.contains("include_bytes"), "{tracker}");
        assert!(tracker.contains("rsx.toml"), "{tracker}");
        assert!(rebuild_tracker(&Theme::default()).is_none());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn load_from_dir_reloads_when_content_changes() {
        let root =
            std::env::temp_dir().join(format!("rsx-theme-fingerprint-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join(CONFIG_FILE);
        std::fs::write(&path, "[colors]\nbrand = \"#123456\"\n").unwrap();
        let first = Theme::load_from_dir(&root).unwrap().unwrap();

        // Keep the same path and shape while changing the value. Content fingerprints make
        // this reliable even on filesystems with coarse modification-time resolution.
        std::fs::write(&path, "[colors]\nbrand = \"#654321\"\n").unwrap();
        let second = Theme::load_from_dir(&root).unwrap().unwrap();
        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(
            second.colors().get("brand"),
            Some(&TokenValue::Color {
                hex: 0x654321,
                alpha: false,
            })
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stable_theme_builder_creates_in_memory_tokens() {
        let theme = Theme::builder()
            .scale(SpacingScale::Tailwind)
            .color(
                "accent",
                TokenValue::Color {
                    hex: 0x123456,
                    alpha: false,
                },
            )
            .spacing("gutter", TokenValue::Length("px(12.0)".into()))
            .build();

        assert_eq!(theme.scale(), Some(SpacingScale::Tailwind));
        assert!(theme.expand_class("bg-accent").is_some());
        assert!(theme.expand_class("gap-gutter").is_some());
    }
}
