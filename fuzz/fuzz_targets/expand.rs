#![no_main]

use gpui_rsx_core::{ClassMode, SpacingScale, analyze_class, class_diagnostics, expand};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|source: &str| {
    for mode in [
        ClassMode::PERMISSIVE,
        ClassMode::STRICT,
        ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind),
    ] {
        let _ = analyze_class(source, mode);
        let _ = class_diagnostics(source, mode);
        if let Ok(tokens) = source.parse::<proc_macro2::TokenStream>() {
            let _ = expand(tokens, mode);
        }
    }
});
