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
    let result = serde_json::json!({
        "diagnostics": gpui_rsx_tools::analyze_body(source, mode.clone()),
        "expansion": gpui_rsx_tools::expand(source, mode.clone()),
        "preview": gpui_rsx_tools::preview::render(source, mode),
    })
    .to_string();
    // The playground calls this on every keystroke; span bookkeeping would otherwise grow forever.
    gpui_rsx_tools::release_span_memory();
    result
}

#[wasm_bindgen]
pub fn format(source: &str) -> Result<String, JsValue> {
    let result = gpui_rsx_tools::format::body(source).map_err(|e| JsValue::from_str(&e));
    gpui_rsx_tools::release_span_memory();
    result
}

#[wasm_bindgen]
pub fn convert(source: &str) -> String {
    let result = serde_json::to_string(&gpui_rsx_tools::convert::convert(source))
        .unwrap_or_else(|e| format!("{{\"Err\":{e:?}}}"));
    gpui_rsx_tools::release_span_memory();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn analyze_json(source: &str, strict: bool, tailwind: bool) -> Value {
        serde_json::from_str(&analyze(source, strict, tailwind)).unwrap()
    }

    #[test]
    fn analyze_reports_diagnostics_expansion_and_preview() {
        let report = analyze_json("<div class=\"flex-colum\"/>", false, false);
        let diagnostic = &report["diagnostics"][0];
        assert_eq!(diagnostic["severity"], "warning");
        assert_eq!(diagnostic["suggestion"], "flex-col");
        assert!(
            report["expansion"]["Ok"]
                .as_str()
                .unwrap()
                .contains("flex_colum")
        );
        assert!(
            report["preview"]["Ok"]
                .as_str()
                .unwrap()
                .contains("partial")
        );
    }

    #[test]
    fn analyze_flags_follow_the_requested_mode() {
        // Strict mode turns the unknown class into an error.
        let strict = analyze_json("<div class=\"nonsense\"/>", true, false);
        assert_eq!(strict["diagnostics"][0]["severity"], "error");

        // The tailwind flag rescales numeric lengths.
        let scaled = analyze_json("<div class=\"gap-4\"/>", false, true);
        assert!(
            scaled["expansion"]["Ok"]
                .as_str()
                .unwrap()
                .contains("16f32")
        );
        let unscaled = analyze_json("<div class=\"gap-4\"/>", false, false);
        assert!(
            unscaled["expansion"]["Ok"]
                .as_str()
                .unwrap()
                .contains("4f32")
        );
    }

    #[test]
    fn analyze_reports_parse_errors_instead_of_panicking() {
        let report = analyze_json("<div", false, false);
        assert!(report["expansion"]["Err"].is_string(), "{report}");
        assert!(report["preview"]["Err"].is_string(), "{report}");
    }

    #[test]
    fn convert_serializes_both_ok_and_err() {
        let converted: Value = serde_json::from_str(&convert("<div className=\"p-4\"/>")).unwrap();
        assert_eq!(converted["Ok"]["rsx"], "<div class=\"p-16\" />");
        assert_eq!(converted["Ok"]["diagnostics"].as_array().unwrap().len(), 0);

        let failed: Value = serde_json::from_str(&convert("not jsx")).unwrap();
        assert!(failed["Err"].is_string(), "{failed}");
    }
}
