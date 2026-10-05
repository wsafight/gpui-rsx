//! Runtime class handling
//!
//! Generates code for runtime dynamic class string parsing and application.
//! Used when the value of a class attribute is an expression rather than a string literal.
//!
//! Optimization: Use thread_local to cache the complete helper source code, avoiding repeated generation and segmented parsing for multiple dynamic classes.

use super::class::{ClassMode, parse_single_class_with_mode};
use super::tables::{
    COLOR_FAMILIES, COLOR_SHADES, LENGTH_CLASS_SPECS, dynamic_common_classes, lookup_color,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::cell::OnceCell;

// Cache the concatenated string of all match branches (thread_local guarantees generation only once during compilation)
//
// Note: Cannot cache proc_macro2::TokenStream because its token handles are bound to the
// bridge connection of the current proc macro invocation. After each invocation ends, the bridge becomes invalid,
// and in the next invocation the old handles become dangling references, causing a "use-after-free" panic.
//
// Optimization: Cache the complete helper as a single string, doing only 1 parse per macro invocation,
// avoiding rebuilding and parsing the three code segments (common/color/numeric) separately.
thread_local! {
    static PERMISSIVE_HELPER_STR: OnceCell<String> = const { OnceCell::new() };
    static STRICT_HELPER_STR: OnceCell<String> = const { OnceCell::new() };
}

/// Get the complete dynamic class helper.
///
/// Caches a string rather than `TokenStream` to avoid retaining invalid token handles across the proc-macro bridge.
/// Each call site only needs to parse the complete helper once, replacing three separate parses for common/color/numeric.
fn get_cached_dynamic_class_helper(mode: ClassMode) -> TokenStream {
    let parse_cached = |cell: &OnceCell<String>| {
        let source = cell.get_or_init(|| generate_dynamic_class_helper(mode).to_string());
        source
            .parse::<TokenStream>()
            .expect("cached dynamic class helper is valid")
    };

    match mode {
        ClassMode::Permissive => PERMISSIVE_HELPER_STR.with(parse_cached),
        ClassMode::Strict => STRICT_HELPER_STR.with(parse_cached),
    }
}

/// Generate runtime class parsing code
///
/// When the class attribute is a dynamic expression, generates a closure that parses and applies classes at runtime.
///
/// # Supported classes
///
/// Dynamic classes support parsing categories:
/// 1. **Static match table** (fast path): Precompiled common classes in [`generate_common_class_matches`]
/// 2. **Color prefix parsing**: Full Tailwind color palette + `[#rgb]` / `[#rrggbb]` arbitrary hex
/// 3. **Numeric prefix parsing** (general path): For spacing/size/opacity classes, supports arbitrary numeric values
///    - `gap-7`, `gap-x-3`, `p-5`, `px-7`, `m-3`, `ml-5`, `w-48`, `h-16`, etc.
///    - `opacity-33`, etc.
///    - Automatically falls back to this path when static match misses, without needing to expand the precompiled list
/// 4. **Other classes** (such as Tailwind variants, custom classes) are silently ignored
///
/// Recommended options (in order of performance from highest to lowest):
/// 1. **String literal** (best): `class="flex gap-4"` -> Expanded at compile-time, supports all classes
/// 2. **Conditional expression** (second best): `class={if active { "flex" } else { "block" }}`
/// 3. **Dynamic expression**: `class={dynamic_str}` -> Supports spacing/size/opacity and arbitrary hex colors
///
/// # Code size optimization
///
/// The match table is extracted into an `#[inline(never)]` generic local function, bringing two benefits:
/// 1. Multiple `class={expr}` of the same element type share the same monomorphized instance (LLVM ICF merging)
/// 2. `#[inline(never)]` prevents the match table from being inlined into the parent function, reducing instruction cache pressure
///
/// # Generated code pattern
///
/// ```ignore
/// {
///     #[inline(never)]
///     fn __rsx_apply_class<E: Styled>(el: E, class: &str) -> E {
///         match class {
///             "flex" => el.flex(),
///             "gap-4" => el.gap(px(4.0)),
///             _ => {
///                 // Numeric prefix fallback: handle arbitrary numeric values (gap-7, p-5, etc.)
///                 if let Some(rest) = class.strip_prefix("gap-") {
///                     if let Ok(n) = rest.parse::<f32>() { return el.gap(px(n)); }
///                 }
///                 // ... remaining prefixes ...
///                 el
///             }
///         }
///     }
///     let __class_str: &str = __class_expr.as_ref();
///     if __class_str.is_empty() { __el } else {
///         __class_str.split_ascii_whitespace().fold(__el, __rsx_apply_class)
///     }
/// }
/// ```
pub(crate) fn generate_dynamic_class_code_with_mode(
    class_expr: &syn::Expr,
    mode: ClassMode,
) -> TokenStream {
    let helper = get_cached_dynamic_class_helper(mode);

    quote! {
        {
            #helper
            // AsRef<str>: &str, String, Cow<str> all pass zero-copy
            let __class_expr = #class_expr;
            let __class_str: &str = __class_expr.as_ref();
            // Empty string fast path: skip iterator creation (common in class={if c { "flex" } else { "" }})
            // split_ascii_whitespace is faster than split_whitespace - class names only contain ASCII characters
            if __class_str.is_empty() {
                __el
            } else {
                __class_str.split_ascii_whitespace().fold(__el, __rsx_apply_class)
            }
        }
    }
}

fn generate_dynamic_class_helper(mode: ClassMode) -> TokenStream {
    let common_classes = generate_common_class_matches();
    let color_fallbacks = generate_color_fallback_code();
    let numeric_fallbacks = generate_numeric_fallback_code();
    let unknown_fallback = match mode {
        ClassMode::Permissive => quote! {
            // Only print warnings in debug builds to avoid syscalls polluting logs every frame in release.
            // Warn only once for the same unknown class at the same generation site to prevent spamming stderr in render loops.
            #[cfg(debug_assertions)]
            if !class.is_empty() {
                fn __rsx_warn_unknown_dynamic_class_once(class: &str) {
                    static __RSX_WARNED_UNKNOWN_CLASSES: std::sync::OnceLock<
                        std::sync::Mutex<std::collections::HashSet<String>>
                    > = std::sync::OnceLock::new();

                    let warned = __RSX_WARNED_UNKNOWN_CLASSES
                        .get_or_init(|| std::sync::Mutex::new(std::collections::HashSet::new()));
                    let Ok(mut warned) = warned.lock() else {
                        return;
                    };
                    if warned.insert(class.to_owned()) {
                        eprintln!(
                            "[gpui-rsx] warning: dynamic class {:?} ignored (unsupported class type)\n  \
                             hint: use string literal class=\"{}\" instead to support all classes",
                            class, class
                        );
                    }
                }

                __rsx_warn_unknown_dynamic_class_once(class);
            }
            el
        },
        ClassMode::Strict => quote! {
            panic!(
                "[gpui-rsx] unsupported dynamic class {:?} in strict mode. \
                 Use rsx! or rsx_permissive! to ignore unsupported dynamic classes.",
                class
            );
        },
    };

    quote! {
        // The match table is extracted into an #[inline(never)] local function:
        // - Prevents inlining bloat; multiple class={expr} within the same component share the function body
        // - LLVM ICF can merge monomorphized instances of the same type
        #[inline(never)]
        fn __rsx_apply_class<E: Styled>(el: E, class: &str) -> E {

            match class {
                #(#common_classes)*
                _ => {
                    // Color prefix parsing: covers the full Tailwind color palette and arbitrary hex.
                    #color_fallbacks
                    // Numeric prefix fallback: when static match misses, try prefix + numeric parsing
                    // Covers arbitrary numeric values such as gap-7, px-5, ml-3, opacity-33, etc.
                    #numeric_fallbacks
                    #unknown_fallback
                }
            }
        }
    }
}

/// Generate fallback code for dynamic color parsing.
///
/// Compared to expanding full color palette match arms for text/bg/border prefixes, here we only
/// parse `prefix-family-shade` in the dynamic path, reducing the expansion size from 700+ color branches to 22 color family branches.
fn generate_color_fallback_code() -> TokenStream {
    let shade_arms = COLOR_SHADES.iter().enumerate().map(|(idx, shade)| {
        quote! { #shade => #idx, }
    });

    let family_arms = COLOR_FAMILIES.iter().map(|family| {
        let values = COLOR_SHADES.iter().map(|shade| {
            let key = format!("{family}_{shade}");
            lookup_color(&key).expect("COLOR_FAMILIES/COLOR_SHADES must match lookup_color")
        });
        quote! {
            #family => {
                const VALUES: [u32; 11] = [#(#values),*];
                Some(VALUES[shade_index])
            }
        }
    });

    quote! {
        fn __rsx_hex_digit(byte: u8) -> Option<u32> {
            match byte {
                b'0'..=b'9' => Some((byte - b'0') as u32),
                b'a'..=b'f' => Some((byte - b'a' + 10) as u32),
                b'A'..=b'F' => Some((byte - b'A' + 10) as u32),
                _ => None,
            }
        }

        fn __rsx_parse_hex_color(color: &str) -> Option<u32> {
            let inner = color.strip_prefix("[#")?.strip_suffix(']')?;
            let bytes = inner.as_bytes();
            match bytes.len() {
                8 => u32::from_str_radix(inner, 16).ok(),
                6 => u32::from_str_radix(inner, 16).ok(),
                4 => {
                    let r = __rsx_hex_digit(bytes[0])?;
                    let g = __rsx_hex_digit(bytes[1])?;
                    let b = __rsx_hex_digit(bytes[2])?;
                    let a = __rsx_hex_digit(bytes[3])?;
                    let expand = |n: u32| (n << 4) | n;
                    Some(expand(r) << 24 | expand(g) << 16 | expand(b) << 8 | expand(a))
                }
                3 => {
                    let r = __rsx_hex_digit(bytes[0])?;
                    let g = __rsx_hex_digit(bytes[1])?;
                    let b = __rsx_hex_digit(bytes[2])?;
                    Some(r << 20 | r << 16 | g << 12 | g << 8 | b << 4 | b)
                }
                _ => None,
            }
        }

        fn __rsx_parse_u8_component(raw: &str) -> Option<u8> {
            raw.trim().parse::<u8>().ok()
        }

        fn __rsx_parse_alpha_component(raw: &str) -> Option<u8> {
            let value = raw.trim().parse::<f32>().ok()?;
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return None;
            }
            Some((value * 255.0).round() as u8)
        }

        fn __rsx_parse_color_function(color: &str) -> Option<(u32, bool)> {
            let inner = color.strip_prefix('[')?.strip_suffix(']')?;

            if let Some(args) = inner.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
                let mut parts = args.split(',');
                let r = __rsx_parse_u8_component(parts.next()?)?;
                let g = __rsx_parse_u8_component(parts.next()?)?;
                let b = __rsx_parse_u8_component(parts.next()?)?;
                if parts.next().is_some() {
                    return None;
                }
                return Some((((r as u32) << 16) | ((g as u32) << 8) | b as u32, false));
            }

            if let Some(args) = inner.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
                let mut parts = args.split(',');
                let r = __rsx_parse_u8_component(parts.next()?)?;
                let g = __rsx_parse_u8_component(parts.next()?)?;
                let b = __rsx_parse_u8_component(parts.next()?)?;
                let a = __rsx_parse_alpha_component(parts.next()?)?;
                if parts.next().is_some() {
                    return None;
                }
                return Some((
                    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a as u32,
                    true,
                ));
            }

            None
        }

        fn __rsx_parse_named_color(color: &str) -> Option<u32> {
            if color == "black" {
                return Some(0x000000);
            }
            if color == "white" {
                return Some(0xffffff);
            }

            let (family, shade) = color.rsplit_once('-')?;
            let shade_index = match shade {
                #(#shade_arms)*
                _ => return None,
            };

            match family {
                #(#family_arms,)*
                _ => None,
            }
        }

        fn __rsx_parse_color(color: &str) -> Option<(u32, bool)> {
            if color == "transparent" {
                return Some((0x00000000, true));
            }
            if let Some(hex) = __rsx_parse_hex_color(color) {
                let hex_inner = color.strip_prefix("[#")?.strip_suffix(']')?;
                let is_rgba = hex_inner.len() == 8 || hex_inner.len() == 4;
                return Some((hex, is_rgba));
            }
            __rsx_parse_color_function(color)
                .or_else(|| __rsx_parse_named_color(color).map(|color| (color, false)))
        }

        if let Some(rest) = class.strip_prefix("text-")
            && let Some((color, is_rgba)) = __rsx_parse_color(rest)
        {
            if is_rgba {
                return el.text_color(gpui_kit::rgba(color));
            }
            return el.text_color(gpui_kit::rgb(color));
        }
        if let Some(rest) = class.strip_prefix("bg-")
            && let Some((color, is_rgba)) = __rsx_parse_color(rest)
        {
            if is_rgba {
                return el.bg(gpui_kit::rgba(color));
            }
            return el.bg(gpui_kit::rgb(color));
        }
        if let Some(rest) = class.strip_prefix("border-")
            && let Some((color, is_rgba)) = __rsx_parse_color(rest)
        {
            if is_rgba {
                return el.border_color(gpui_kit::rgba(color));
            }
            return el.border_color(gpui_kit::rgb(color));
        }
    }
}

/// Generate fallback match code for numeric prefixes.
///
/// When the static match table misses, handles arbitrary numeric classes through prefix identification + `parse::<f32>()`.
/// Each if-let uses early `return`; if none match, execution flow falls through to caller's `el`.
///
/// Longer prefixes are checked first (`gap-x-` before `gap-`) to ensure exact matching:
/// For `gap-x-4`, `strip_prefix("gap-")` yields `"x-4"`, and `parse::<f32>()` fails,
/// naturally falling back to the `gap-x-` branch without needing extra sorting.
fn generate_numeric_fallback_code() -> TokenStream {
    let length_fallbacks = LENGTH_CLASS_SPECS.iter().map(generate_length_fallback);
    let usize_fallbacks = [("line-clamp-", "line_clamp")]
        .into_iter()
        .map(|(prefix, method)| generate_integer_fallback(prefix, method, "usize"));
    let u16_fallbacks = [
        ("col-span-", "col_span"),
        ("row-span-", "row_span"),
        ("grid-cols-", "grid_cols"),
        ("grid-rows-", "grid_rows"),
    ]
    .into_iter()
    .map(|(prefix, method)| generate_integer_fallback(prefix, method, "u16"));
    let i16_fallbacks = [
        ("col-start-", "col_start"),
        ("col-end-", "col_end"),
        ("row-start-", "row_start"),
        ("row-end-", "row_end"),
    ]
    .into_iter()
    .map(|(prefix, method)| generate_integer_fallback(prefix, method, "i16"));

    quote! {
        trait __RsxFiniteFloat {
            fn __rsx_finite(self) -> Result<f32, ()>;
        }

        impl __RsxFiniteFloat for Result<f32, std::num::ParseFloatError> {
            fn __rsx_finite(self) -> Result<f32, ()> {
                match self {
                    Ok(n) if n.is_finite() => Ok(n),
                    _ => Err(()),
                }
            }
        }

        #(#length_fallbacks)*
        // --- opacity: opacity-50 -> 0.50 ---
        if let Some(rest) = class.strip_prefix("opacity-") {
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() {
                if (0.0..=100.0).contains(&n) {
                    return el.opacity(n / 100.0);
                }
            }
        }
        #(#usize_fallbacks)*
        #(#u16_fallbacks)*
        #(#i16_fallbacks)*
    }
}

fn generate_length_fallback(spec: &super::tables::LengthClassSpec) -> TokenStream {
    let prefix = spec.prefix;
    let method = syn::Ident::new(spec.method, Span::call_site());
    let percent = if spec.family.allows_percent() {
        quote! {
            if let Some(raw) = inner.strip_suffix('%') {
                if let Ok(n) = raw.parse::<f32>().__rsx_finite() {
                    return el.#method(gpui_kit::relative(n / 100.0));
                }
            }
        }
    } else {
        quote! {}
    };
    let fraction = if spec.family.allows_fraction() {
        quote! {
            if let Some((num, den)) = rest.split_once('/') {
                if let (Ok(num), Ok(den)) = (
                    num.parse::<f32>().__rsx_finite(),
                    den.parse::<f32>().__rsx_finite(),
                ) {
                    if den > 0.0 {
                        return el.#method(gpui_kit::relative(num / den));
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        if let Some(rest) = class.strip_prefix(#prefix) {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() {
                        return el.#method(gpui_kit::px(n));
                    }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() {
                        return el.#method(gpui_kit::rems(n));
                    }
                }
                #percent
            }
            #fraction
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() {
                return el.#method(rems(n * 0.25));
            }
        }
    }
}

fn generate_integer_fallback(prefix: &'static str, method: &'static str, ty: &str) -> TokenStream {
    let method = syn::Ident::new(method, Span::call_site());
    let ty: TokenStream = ty.parse().expect("integer fallback type is valid");

    quote! {
        if let Some(rest) = class.strip_prefix(#prefix) {
            if let Ok(n) = rest.parse::<#ty>() {
                return el.#method(n);
            }
        }
    }
}

/// Generate match branches for common classes.
///
/// Returns a list of match arms, each matching a class string and applying the corresponding method.
/// Cached via thread_local, called only once throughout the compilation process.
///
fn generate_common_class_matches() -> impl Iterator<Item = TokenStream> {
    dynamic_common_classes().map(|class_str| {
        let method_call = parse_dynamic_common_class(class_str);
        quote! {
            #class_str => #method_call,
        }
    })
}

fn parse_dynamic_common_class(class: &str) -> TokenStream {
    match class {
        "debug-outline" => quote! {
            {
                #[cfg(debug_assertions)]
                {
                    el.debug()
                }
                #[cfg(not(debug_assertions))]
                {
                    el
                }
            }
        },
        _ => {
            let method_call = parse_single_class_with_mode(class, ClassMode::Permissive);
            quote! { el #method_call }
        }
    }
}
