//! Resolve shared runtime module paths for libraries, binaries, tests and examples.
use std::path::Path;

/// Resolve a bare module against the package's library when expanding another Cargo target.
/// Explicit Rust paths are preserved, including `crate::...` for binary-only packages.
pub(crate) fn runtime_path(configured: &str) -> syn::Path {
    let dir = std::env::var_os("CARGO_MANIFEST_DIR");
    let manifest = dir
        .as_deref()
        .map(Path::new)
        .and_then(|dir| std::fs::read_to_string(dir.join("Cargo.toml")).ok());
    let package = std::env::var("CARGO_PKG_NAME").ok();
    let current = std::env::var("CARGO_CRATE_NAME").ok();
    // `src/main.rs` shares the library's crate name, so only CARGO_BIN_NAME tells them apart.
    let is_bin = std::env::var_os("CARGO_BIN_NAME").is_some();
    let has_default_lib = dir
        .as_deref()
        .map(Path::new)
        .is_some_and(|dir| dir.join("src/lib.rs").is_file());
    resolve(
        configured,
        current.as_deref(),
        is_bin,
        package.as_deref(),
        manifest.as_deref(),
        has_default_lib,
    )
}

fn resolve(
    configured: &str,
    current: Option<&str>,
    is_bin: bool,
    package: Option<&str>,
    manifest: Option<&str>,
    has_default_lib: bool,
) -> syn::Path {
    let path: syn::Path = syn::parse_str(configured).expect("runtime path was validated by Theme");
    if path.leading_colon.is_some()
        || path.segments.len() > 1
        || path.is_ident("crate")
        || path.is_ident("self")
        || path.is_ident("super")
    {
        return path;
    }
    let library = library_name(manifest.unwrap_or(""), package, has_default_lib);
    let resolved = match library {
        Some(name) if is_bin || current.is_some_and(|current| current != name) => {
            format!("::{name}::{configured}")
        }
        _ => format!("crate::{configured}"),
    };
    syn::parse_str(&resolved).unwrap_or(path)
}

// Cargo's target name cannot be obtained from CARGO_PKG_NAME when [lib].name overrides it.
// Only this one section is needed; explicit runtime paths remain available for unusual manifests.
fn library_name(manifest: &str, package: Option<&str>, has_default_lib: bool) -> Option<String> {
    let mut in_lib = false;
    let mut has_lib = has_default_lib;
    for raw in manifest.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            in_lib = line.split('#').next().unwrap_or("").trim() == "[lib]";
            has_lib |= in_lib;
        } else if in_lib
            && let Some((key, value)) = line.split_once('=')
            && key.trim() == "name"
        {
            let value = value.trim();
            for quote in ['"', '\''] {
                if let Some(value) = value
                    .strip_prefix(quote)
                    .and_then(|v| v.split(quote).next())
                {
                    return Some(value.replace('-', "_"));
                }
            }
        }
    }
    has_lib
        .then(|| package.map(|name| name.replace('-', "_")))
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    #[test]
    fn library_uses_crate_and_other_targets_use_the_library_name() {
        let path = |current| {
            resolve(
                "rsx_runtime",
                Some(current),
                false,
                Some("my-app"),
                None,
                true,
            )
            .to_token_stream()
            .to_string()
        };
        assert_eq!(path("my_app"), "crate :: rsx_runtime");
        assert_eq!(path("integration"), ":: my_app :: rsx_runtime");
        assert_eq!(path("example"), ":: my_app :: rsx_runtime");
    }

    #[test]
    fn custom_library_names_and_binary_only_packages() {
        let custom = resolve(
            "rsx_runtime",
            Some("integration"),
            false,
            Some("app"),
            Some("[lib]\nname = 'custom_lib' # comment"),
            false,
        );
        assert_eq!(
            custom.to_token_stream().to_string(),
            ":: custom_lib :: rsx_runtime"
        );
        let bin = resolve("rsx_runtime", Some("cli"), true, Some("app"), None, false);
        assert_eq!(bin.to_token_stream().to_string(), "crate :: rsx_runtime");
    }

    #[test]
    fn explicit_paths_are_preserved() {
        for path in [
            "crate",
            "self",
            "super",
            "crate::rsx_runtime",
            "::other::rsx_runtime",
            "super::rsx_runtime",
        ] {
            let resolved = resolve(path, Some("example"), true, Some("app"), None, true);
            let expected: syn::Path = syn::parse_str(path).unwrap();
            assert_eq!(
                resolved.to_token_stream().to_string(),
                expected.to_token_stream().to_string()
            );
        }
    }

    #[test]
    fn library_names_come_from_quoted_sections() {
        assert_eq!(
            library_name(
                "[lib]\nname = \"custom-lib\" # trailing",
                Some("app"),
                false
            )
            .as_deref(),
            Some("custom_lib")
        );
        assert_eq!(
            library_name("[lib]\nname = 'single-quoted'", Some("app"), false).as_deref(),
            Some("single_quoted")
        );
        // Name keys outside [lib] are ignored.
        assert_eq!(
            library_name("[other]\nname = \"ignored\"", Some("app"), false),
            None
        );
        assert_eq!(
            library_name("[other]\nname = \"ignored\"", Some("app"), true).as_deref(),
            Some("app")
        );
        // A [lib] section implies a library even without a name key.
        assert_eq!(
            library_name("[lib]\npath = \"src/lib.rs\"", Some("my-app"), false).as_deref(),
            Some("my_app")
        );
        assert_eq!(
            library_name("", Some("my-app"), true).as_deref(),
            Some("my_app")
        );
        assert_eq!(library_name("", None, true), None);
        assert_eq!(library_name("", Some("app"), false), None);
    }

    #[test]
    fn unquoted_or_empty_library_names_fall_back_to_the_package() {
        assert_eq!(
            library_name("[lib]\nname = custom", Some("my-app"), false).as_deref(),
            Some("my_app")
        );
        assert_eq!(
            library_name("[lib]\nname = \"\"", Some("my-app"), false).as_deref(),
            Some("")
        );
    }

    #[test]
    fn bare_paths_use_the_crate_root_without_a_library_target() {
        let path = |current| {
            resolve("rsx_runtime", current, false, Some("my-app"), None, true)
                .to_token_stream()
                .to_string()
        };
        assert_eq!(path(None), "crate :: rsx_runtime");
        assert_eq!(path(Some("my_app")), "crate :: rsx_runtime");
        assert_eq!(path(Some("other")), ":: my_app :: rsx_runtime");
    }

    #[test]
    fn default_binary_with_the_library_name_uses_the_library() {
        let main = resolve(
            "rsx_runtime",
            Some("my_app"),
            true,
            Some("my-app"),
            None,
            true,
        );
        assert_eq!(
            main.to_token_stream().to_string(),
            ":: my_app :: rsx_runtime"
        );
    }
}
