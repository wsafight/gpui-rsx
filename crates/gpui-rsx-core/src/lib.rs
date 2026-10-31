//! # gpui-rsx-core
//!
//! Parser and code generator behind the [`gpui-rsx`](https://docs.rs/gpui-rsx) macros.
//!
//! `gpui-rsx` is a `proc-macro` crate, and proc-macro crates cannot be used as normal
//! libraries. This crate holds all of the parsing and expansion logic as a regular
//! library so that tools such as a CLI, language server or online playground can
//! reuse the same implementation as the macro shell.
//!
//! ```
//! use gpui_rsx_core::{ClassMode, expand_str};
//!
//! let code = expand_str(r#"<div class="flex gap-4" />"#, ClassMode::PERMISSIVE).unwrap();
//! assert_eq!(code.to_string(), "div () . flex () . gap (px (4f32))");
//! ```

// Implementation paths remain available to existing integrations; the root facade
// and syntax module below define the supported public API for 0.9.
#[doc(hidden)]
pub mod codegen;
#[doc(hidden)]
pub mod component;
#[doc(hidden)]
pub mod diagnostics;
#[doc(hidden)]
pub mod parser;
mod runtime_config;
pub mod style;
pub mod theme;

pub use codegen::class::{
    ClassAnalysis, ClassDiagnostic, ClassMode, ClassModeBuilder, ClassOutcome, SUPPORTED_VARIANTS,
    Severity, SpacingScale, analyze_class, class_diagnostics, known_classes, resolve_style,
    suggest_class,
};
pub use codegen::element::{
    generate_body_expansion_preview, generate_body_with_mode, generate_body_without_runtime,
    generate_fragment_with_mode, generate_fragment_without_runtime,
};
pub use codegen::runtime::generate_runtime_module;
pub use codegen::tables::{GPUI_COMPONENT_REV, GPUI_SOURCE_REV};
pub use component::expand_component;
pub use parser::{RsxAttribute, RsxBody, RsxElement, RsxElementName, RsxNode};
pub use style::{StyleLength, StyleOperation, StyleValue};
pub use theme::{Theme, ThemeBuilder, ThemeExplanation, ThemeToken, ThemeTokenKind, TokenValue};

/// Parsed RSX syntax. Match enums with a fallback arm for future syntax additions.
pub mod syntax {
    pub use crate::{RsxAttribute, RsxBody, RsxElement, RsxElementName, RsxNode};
}

use proc_macro2::TokenStream;

/// RSX body macros exported by `gpui-rsx`, shared by tools and the generated spec.
pub const MACRO_NAMES: &[&str] = &[
    "rsx",
    "rsx_strict",
    "rsx_permissive",
    "rsx_tw",
    "rsx_expand",
    "rsx_fragment",
];

/// Parses and expands an RSX macro body. Parse errors become `compile_error!` tokens.
pub fn expand(input: TokenStream, mode: ClassMode) -> TokenStream {
    match syn::parse2::<RsxBody>(input) {
        Ok(body) => generate_body_with_mode(&body, mode),
        Err(error) => error.to_compile_error(),
    }
}

/// Parses RSX source text and expands it, for tools that start from strings.
pub fn expand_str(source: &str, mode: ClassMode) -> Result<TokenStream, syn::Error> {
    let tokens: TokenStream = source.parse().map_err(syn::Error::from)?;
    let body = syn::parse2::<RsxBody>(tokens)?;
    Ok(generate_body_with_mode(&body, mode))
}

/// Options a macro entry point resolves before expanding.
#[derive(Clone, Copy, Debug, Default)]
pub struct MacroOptions {
    strict: bool,
    erased_roots: bool,
    /// Scale forced by the macro itself (`rsx_tw!`), overriding `rsx.toml`.
    forced_scale: Option<SpacingScale>,
    /// Scale used when neither the macro nor `rsx.toml` chooses one (Cargo feature).
    default_scale: SpacingScale,
    report_dynamic_at_exit: bool,
}

impl MacroOptions {
    /// Creates macro options with the default permissive, pixel-scale behavior.
    pub const fn new() -> Self {
        Self {
            strict: false,
            erased_roots: false,
            forced_scale: None,
            default_scale: SpacingScale::Px,
            report_dynamic_at_exit: false,
        }
    }

    pub const fn with_strict(self, strict: bool) -> Self {
        Self { strict, ..self }
    }

    /// Makes every root an `AnyElement` and returns a `Vec`, including a single root.
    pub const fn with_erased_roots(self, erased_roots: bool) -> Self {
        Self {
            erased_roots,
            ..self
        }
    }

    pub const fn with_forced_scale(self, scale: Option<SpacingScale>) -> Self {
        Self {
            forced_scale: scale,
            ..self
        }
    }

    pub const fn with_default_scale(self, scale: SpacingScale) -> Self {
        Self {
            default_scale: scale,
            ..self
        }
    }

    pub const fn with_dynamic_report(self, report: bool) -> Self {
        Self {
            report_dynamic_at_exit: report,
            ..self
        }
    }

    pub const fn is_strict(&self) -> bool {
        self.strict
    }

    pub const fn forced_scale(&self) -> Option<SpacingScale> {
        self.forced_scale
    }

    pub const fn default_scale(&self) -> SpacingScale {
        self.default_scale
    }

    pub const fn reports_dynamic_at_exit(&self) -> bool {
        self.report_dynamic_at_exit
    }
}

/// Spacing-scale priority shared by the macros and tools: an explicit choice (`rsx_tw!` or a
/// tool flag), then `spacing-scale` in `rsx.toml`, then the `tailwind-scale` crate feature.
pub fn resolve_spacing_scale(
    forced: Option<SpacingScale>,
    theme: Option<&Theme>,
    feature_default: SpacingScale,
) -> SpacingScale {
    forced
        .or_else(|| theme.and_then(Theme::scale))
        .unwrap_or(feature_default)
}

/// Expands a macro invocation, loading `rsx.toml` from `CARGO_MANIFEST_DIR` when present.
///
/// When a config file is used, the output is wrapped in a block that `include_bytes!`s it so
/// cargo rebuilds the crate after the file changes.
pub fn expand_macro(input: TokenStream, options: MacroOptions) -> TokenStream {
    let theme = match std::env::var_os("CARGO_MANIFEST_DIR") {
        Some(dir) => match Theme::load_from_dir(std::path::Path::new(&dir)) {
            Ok(theme) => theme,
            Err(message) => return quote::quote! { compile_error!(#message) },
        },
        None => None,
    };
    let scale = resolve_spacing_scale(
        options.forced_scale(),
        theme.as_deref(),
        options.default_scale(),
    );
    let mut mode = ClassMode::builder()
        .strict(options.is_strict())
        .scale(scale)
        .theme(theme.clone())
        .report_dynamic_at_exit(options.reports_dynamic_at_exit())
        .build();
    if let Some(path) = theme.as_deref().and_then(Theme::dynamic_runtime) {
        mode = mode.with_dynamic_runtime_path(runtime_config::runtime_path(path));
    }
    let code = if options.erased_roots {
        match syn::parse2::<RsxBody>(input) {
            Ok(body) => generate_fragment_with_mode(&body, mode),
            Err(error) => error.to_compile_error(),
        }
    } else {
        expand(input, mode)
    };
    match theme.as_deref().and_then(theme::rebuild_tracker) {
        Some(tracker) => quote::quote! { { #tracker #code } },
        None => code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `CARGO_MANIFEST_DIR` is process-wide, so tests that change it must not overlap.
    fn with_manifest_dir<T>(dir: Option<&std::path::Path>, f: impl FnOnce() -> T) -> T {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let previous = std::env::var_os("CARGO_MANIFEST_DIR");
        // SAFETY: the mutex above serializes every test that touches this variable.
        unsafe {
            match dir {
                Some(dir) => std::env::set_var("CARGO_MANIFEST_DIR", dir),
                None => std::env::remove_var("CARGO_MANIFEST_DIR"),
            }
        }
        let result = f();
        unsafe {
            match previous {
                Some(value) => std::env::set_var("CARGO_MANIFEST_DIR", value),
                None => std::env::remove_var("CARGO_MANIFEST_DIR"),
            }
        }
        result
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("rsx-core-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn options() -> MacroOptions {
        MacroOptions::default()
    }

    #[test]
    fn expand_macro_without_a_theme_uses_the_default_scale() {
        let dir = temp_dir("no-theme");
        let code = with_manifest_dir(Some(&dir), || {
            expand_macro(quote::quote! { <div class="gap-4"/> }, options())
        })
        .to_string();
        assert!(code.contains("gap (px (4f32))"), "{code}");
        assert!(!code.contains("include_bytes"), "{code}");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn expand_macro_uses_the_theme_scale_and_tracks_the_config_file() {
        let dir = temp_dir("theme-scale");
        std::fs::write(
            dir.join("rsx.toml"),
            "[options]\nspacing-scale = \"tailwind\"\n",
        )
        .unwrap();

        let code = with_manifest_dir(Some(&dir), || {
            expand_macro(quote::quote! { <div class="gap-4 bg-brand"/> }, options())
        })
        .to_string();
        assert!(code.contains("gap (px (16f32))"), "{code}");
        assert!(code.contains("include_bytes"), "{code}");
        assert!(code.contains("rsx.toml"), "{code}");

        // An explicit `rsx_tw!` scale wins over the config file.
        let forced = with_manifest_dir(Some(&dir), || {
            expand_macro(
                quote::quote! { <div class="gap-4"/> },
                options().with_forced_scale(Some(SpacingScale::Px)),
            )
        })
        .to_string();
        assert!(forced.contains("gap (px (4f32))"), "{forced}");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn expand_macro_reports_broken_configs_as_compile_errors() {
        let dir = temp_dir("broken-theme");
        std::fs::write(dir.join("rsx.toml"), "[nope]\n").unwrap();
        let code = with_manifest_dir(Some(&dir), || {
            expand_macro(quote::quote! { <div/> }, options())
        })
        .to_string();
        assert!(code.contains("compile_error"), "{code}");
        assert!(code.contains("unknown section"), "{code}");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn expand_macro_without_a_manifest_dir_skips_theme_loading() {
        let code = with_manifest_dir(None, || {
            expand_macro(quote::quote! { <div class="flex"/> }, options())
        })
        .to_string();
        assert!(code.contains(". flex ()"), "{code}");
        assert!(!code.contains("compile_error"), "{code}");
    }

    #[test]
    fn macro_options_default_to_pixel_units_without_reporting() {
        let options = MacroOptions::default();
        assert!(!options.is_strict());
        assert_eq!(options.forced_scale(), None);
        assert_eq!(options.default_scale(), SpacingScale::Px);
        assert!(!options.reports_dynamic_at_exit());
    }

    #[test]
    fn stable_macro_options_builder_keeps_explicit_overrides() {
        let options = MacroOptions::new()
            .with_strict(true)
            .with_forced_scale(Some(SpacingScale::Tailwind))
            .with_default_scale(SpacingScale::Px)
            .with_dynamic_report(true);

        assert!(options.is_strict());
        assert_eq!(options.forced_scale(), Some(SpacingScale::Tailwind));
        assert_eq!(options.default_scale(), SpacingScale::Px);
        assert!(options.reports_dynamic_at_exit());
    }
}
