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
//! ```
//!
//! Only a small TOML subset is accepted (sections, `key = "string"`, `key = { expr = "..." }`
//! and `#` comments) so the macro does not need a TOML dependency.

use crate::codegen::class::SpacingScale;
use crate::codegen::tables::{GPUI_COMPONENT_THEME_COLORS, split_length_class};
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

/// File name looked up in `CARGO_MANIFEST_DIR`.
pub const CONFIG_FILE: &str = "rsx.toml";

#[derive(Clone, Debug, PartialEq)]
pub enum TokenValue {
    /// `0xRRGGBB` (`alpha == false`) or `0xRRGGBBAA`.
    Color { hex: u32, alpha: bool },
    /// A GPUI length expression such as `px(12.0)` or `rems(0.75)`.
    Length(String),
    /// An arbitrary Rust expression evaluated at render time.
    Expr(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Theme {
    /// Source file, used for rebuild tracking. `None` for in-memory themes.
    pub path: Option<PathBuf>,
    pub scale: Option<SpacingScale>,
    pub colors: BTreeMap<String, TokenValue>,
    pub radius: BTreeMap<String, TokenValue>,
    pub spacing: BTreeMap<String, TokenValue>,
}

impl Theme {
    /// Parses `rsx.toml` contents.
    pub fn parse(source: &str) -> Result<Self, String> {
        let mut theme = Theme::default();
        let mut section = String::new();
        let mut preset = None;
        let mut context = "cx".to_owned();

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
                    "options" | "colors" | "radius" | "spacing"
                ) {
                    return Err(format!(
                        "rsx.toml:{line_no}: unknown section `[{section}]` (expected options, colors, radius or spacing)"
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
                "radius" | "spacing" => {
                    let token = match value {
                        RawValue::Expr(expr) => TokenValue::Expr(validate_expr(&expr, line_no)?),
                        RawValue::Str(length) => parse_length(&length).ok_or_else(|| {
                            format!("rsx.toml:{line_no}: `{key}` must be a length like \"12px\" or \"0.75rem\"")
                        })?,
                    };
                    let map = if section == "radius" {
                        &mut theme.radius
                    } else {
                        &mut theme.spacing
                    };
                    map.insert(key, token);
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

        Ok(theme)
    }

    /// Loads `rsx.toml` from `dir`, caching by path and modification time.
    ///
    /// Returns `Ok(None)` when the file does not exist.
    pub fn load_from_dir(dir: &Path) -> Result<Option<&'static Theme>, String> {
        type Cache = Mutex<BTreeMap<PathBuf, (Option<SystemTime>, &'static Theme)>>;
        static CACHE: OnceLock<Cache> = OnceLock::new();

        let path = dir.join(CONFIG_FILE);
        let Ok(metadata) = std::fs::metadata(&path) else {
            return Ok(None);
        };
        let modified = metadata.modified().ok();
        let cache = CACHE.get_or_init(Default::default);
        if let Ok(cache) = cache.lock()
            && let Some((cached_at, theme)) = cache.get(&path)
            && *cached_at == modified
            && modified.is_some()
        {
            return Ok(Some(theme));
        }

        let source = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let mut theme = Theme::parse(&source)?;
        theme.path = Some(path.clone());
        // Leaked on purpose: one entry per (file, revision) for the life of the compiler process.
        let theme: &'static Theme = Box::leak(Box::new(theme));
        if let Ok(mut cache) = cache.lock() {
            cache.insert(path, (modified, theme));
        }
        Ok(Some(theme))
    }

    /// Expands a theme-token class (`bg-primary`, `rounded-card`, `p-gutter`).
    pub fn expand_class(&self, class: &str) -> Option<TokenStream> {
        for (prefix, method) in [
            ("bg-", "bg"),
            ("text-", "text_color"),
            ("border-", "border_color"),
        ] {
            if let Some(name) = class.strip_prefix(prefix)
                && let Some(value) = self.colors.get(name)
            {
                return Some(method_call(method, color_tokens(value)));
            }
        }
        if let Some(name) = class.strip_prefix("rounded-")
            && let Some(value) = self.radius.get(name)
        {
            return Some(method_call("rounded", value_tokens(value)));
        }
        if let Some((method, name, _)) = split_length_class(class)
            && let Some(value) = self.spacing.get(name)
        {
            return Some(method_call(method, value_tokens(value)));
        }
        None
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
        for name in self.spacing.keys() {
            for prefix in ["p-", "px-", "py-", "m-", "mx-", "my-", "gap-", "w-", "h-"] {
                out.push(format!("{prefix}{name}"));
            }
        }
        out
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
    for (index, c) in line.char_indices() {
        match c {
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
    let hex = color.strip_prefix('#')?;
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

fn color_tokens(value: &TokenValue) -> TokenStream {
    match value {
        TokenValue::Color { hex, alpha: false } => quote! { rgb(#hex) },
        TokenValue::Color { hex, alpha: true } => quote! { rgba(#hex) },
        other => value_tokens(other),
    }
}

fn value_tokens(value: &TokenValue) -> TokenStream {
    match value {
        TokenValue::Length(expr) | TokenValue::Expr(expr) => {
            expr.parse().expect("validated when rsx.toml was parsed")
        }
        TokenValue::Color { .. } => color_tokens(value),
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
}
