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

pub mod codegen;
pub mod component;
pub mod diagnostics;
pub mod parser;
pub mod theme;

pub use codegen::class::{
    ClassAnalysis, ClassDiagnostic, ClassMode, ClassOutcome, SUPPORTED_VARIANTS, Severity,
    SpacingScale, analyze_class, class_diagnostics, known_classes, suggest_class,
};
pub use codegen::element::{generate_body_expansion_preview, generate_body_with_mode};
pub use codegen::tables::{GPUI_COMPONENT_REV, GPUI_SOURCE_REV};
pub use component::expand_component;
pub use parser::RsxBody;
pub use theme::Theme;

use proc_macro2::TokenStream;

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
    pub strict: bool,
    /// Scale forced by the macro itself (`rsx_tw!`), overriding `rsx.toml`.
    pub forced_scale: Option<SpacingScale>,
    /// Scale used when neither the macro nor `rsx.toml` chooses one (Cargo feature).
    pub default_scale: SpacingScale,
    pub report_dynamic_at_exit: bool,
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
    let scale = options
        .forced_scale
        .or_else(|| theme.and_then(|t| t.scale))
        .unwrap_or(options.default_scale);
    let mode = ClassMode {
        strict: options.strict,
        scale,
        theme,
        report_dynamic_at_exit: options.report_dynamic_at_exit,
    };
    let code = expand(input, mode);
    match theme.and_then(theme::rebuild_tracker) {
        Some(tracker) => quote::quote! { { #tracker #code } },
        None => code,
    }
}
