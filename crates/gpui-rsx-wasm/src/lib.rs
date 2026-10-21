use wasm_bindgen::prelude::*;

/// All operations use the same parser and code generator as the Rust macros.
#[wasm_bindgen]
pub fn analyze(source: &str, strict: bool, tailwind: bool) -> String {
    let mode = gpui_rsx_core::ClassMode {
        strict,
        scale: if tailwind {
            gpui_rsx_core::SpacingScale::Tailwind
        } else {
            gpui_rsx_core::SpacingScale::Px
        },
        ..gpui_rsx_core::ClassMode::PERMISSIVE
    };
    serde_json::json!({
        "diagnostics": gpui_rsx_tools::analyze_body(source, mode),
        "expansion": gpui_rsx_tools::expand(source, mode),
        "preview": gpui_rsx_tools::preview::render(source, mode),
    })
    .to_string()
}

#[wasm_bindgen]
pub fn format(source: &str) -> Result<String, JsValue> {
    gpui_rsx_tools::format::body(source).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn convert(source: &str) -> String {
    serde_json::to_string(&gpui_rsx_tools::convert::convert(source))
        .unwrap_or_else(|e| format!("{{\"Err\":{e:?}}}"))
}
