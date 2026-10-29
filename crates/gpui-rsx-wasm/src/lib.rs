use wasm_bindgen::prelude::*;

/// All operations use the same parser and code generator as the Rust macros.
#[wasm_bindgen]
pub fn analyze(source: &str, strict: bool, tailwind: bool) -> String {
    analyze_with_theme(source, strict, tailwind, "")
}

/// Parses an in-memory rsx.toml. Expressions are displayed but never executed.
#[wasm_bindgen]
pub fn analyze_with_theme(
    source: &str,
    strict: bool,
    tailwind: bool,
    theme_source: &str,
) -> String {
    let (theme, theme_diagnostics) = match gpui_rsx_core::Theme::parse(theme_source) {
        Ok(theme) => (Some(std::sync::Arc::new(theme)), Vec::<String>::new()),
        Err(error) => (None, vec![error]),
    };
    let scale = theme
        .as_ref()
        .and_then(|t| t.scale())
        .unwrap_or(if tailwind {
            gpui_rsx_core::SpacingScale::Tailwind
        } else {
            gpui_rsx_core::SpacingScale::Px
        });
    let mode = gpui_rsx_core::ClassMode::builder()
        .strict(strict)
        .scale(scale)
        .theme(theme)
        .build();
    let report = gpui_rsx_tools::report_body(source, mode);
    let result = serde_json::json!({
        "diagnostics": report.diagnostics,
        "theme_diagnostics": theme_diagnostics,
        "tailwind": scale == gpui_rsx_core::SpacingScale::Tailwind,
        "expansion": report.expansion,
        "preview": report.preview,
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
    convert_with_options(source, false)
}

#[wasm_bindgen]
pub fn convert_with_options(source: &str, tailwind: bool) -> String {
    let mut options = gpui_rsx_tools::convert::ConversionOptions::default();
    options.tailwind = tailwind;
    let result = serde_json::to_string(&gpui_rsx_tools::convert::convert_with_options(
        source, options,
    ))
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

    #[test]
    fn analyze_with_theme_reports_theme_errors_without_panicking() {
        let source = "<div class=\"bg-brand\"/>";
        let themed = analyze_with_theme(source, false, false, "[colors]\nbrand = \"#123456\"\n");
        let report: Value = serde_json::from_str(&themed).unwrap();
        assert!(report["theme_diagnostics"].as_array().unwrap().is_empty());
        assert!(report["expansion"]["Ok"].as_str().unwrap().contains("rgb"));

        let broken = analyze_with_theme(source, false, false, "[nope]\n");
        let report: Value = serde_json::from_str(&broken).unwrap();
        assert_eq!(report["theme_diagnostics"].as_array().unwrap().len(), 1);
        assert!(
            report["theme_diagnostics"][0]
                .as_str()
                .unwrap()
                .contains("unknown section")
        );
    }

    #[test]
    fn analyze_with_theme_prefers_theme_scale_over_the_tailwind_flag() {
        let source = "<div class=\"gap-4\"/>";
        let report: Value = serde_json::from_str(&analyze_with_theme(
            source,
            false,
            false,
            "[options]\nspacing-scale = \"tailwind\"\n",
        ))
        .unwrap();
        assert_eq!(report["tailwind"], true);
        assert!(
            report["expansion"]["Ok"]
                .as_str()
                .unwrap()
                .contains("16f32")
        );
    }

    #[test]
    fn format_returns_formatted_bodies() {
        assert_eq!(
            format("<div class=\"flex\"><span /></div>").unwrap(),
            "<div class=\"flex\">\n    <span />\n</div>"
        );
        assert_eq!(format("<div />").unwrap(), "<div />");
    }

    #[test]
    fn convert_with_options_honours_tailwind() {
        // Tailwind conversion keeps numeric classes and switches to `rsx_tw!`.
        let converted: Value =
            serde_json::from_str(&convert_with_options("<div className=\"p-4\"/>", true)).unwrap();
        assert!(converted["Ok"]["rsx"].as_str().unwrap().contains("rsx_tw!"));
        assert!(converted["Ok"]["rsx"].as_str().unwrap().contains("p-4"));
        // The pixel-preserving conversion rewrites the class instead.
        let plain: Value =
            serde_json::from_str(&convert_with_options("<div className=\"p-4\"/>", false)).unwrap();
        assert_eq!(plain["Ok"]["rsx"], "<div class=\"p-16\" />");
    }
}
