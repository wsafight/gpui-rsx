use gpui_rsx_core::{ClassMode, Theme, class_diagnostics};
use gpui_rsx_tools::{ClassTokenKind, DiagnosticSeverity, analyze_rust_document};

#[test]
fn repeated_classes_keep_diagnostics_colors_and_literal_offsets() {
    let classes = "bg-brand hover:bg-brand w-[-4px] flex-colum";
    let source = format!(
        r##"fn view(){{rsx!{{<div class="{classes}"><span class="{classes}"/><b whenClass={{(active, "{classes}")}}/></div>}};}}"##
    );
    let theme = Theme::parse("[colors]\nbrand=\"#123456\"\n").unwrap();
    let mode = ClassMode::PERMISSIVE.with_theme(Some(theme.into()));
    let expected = class_diagnostics(classes, mode.clone());
    let analysis = analyze_rust_document(&source, mode);
    assert_eq!(analysis.literals.len(), 3);
    assert_eq!(analysis.class_tokens.len(), 12);
    assert_eq!(analysis.diagnostics.len(), 3 * expected.len());
    for (&(start, _), diagnostics) in analysis
        .literals
        .iter()
        .zip(analysis.diagnostics.chunks(expected.len()))
    {
        for (actual, expected) in diagnostics.iter().zip(&expected) {
            assert_eq!(actual.start, start + expected.offset);
            assert_eq!(actual.end, actual.start + expected.class.len());
            assert_eq!(actual.message, expected.message);
            assert_eq!(actual.suggestion, expected.suggestion);
        }
    }
    for tokens in analysis.class_tokens.chunks(4) {
        assert_eq!(tokens[0].kind, ClassTokenKind::Theme);
        assert_eq!(tokens[1].kind, ClassTokenKind::Theme);
        assert_eq!(tokens[0].rgba, Some(0x123456ff));
        assert_eq!(tokens[1].rgba, Some(0x123456ff));
        assert_eq!(tokens[2].kind, ClassTokenKind::Supported);
        assert_eq!(tokens[3].kind, ClassTokenKind::Invalid);
    }
}

#[test]
fn escaped_literals_and_macro_modes_do_not_share_offsets_or_severity() {
    let source = r##"fn view(){rsx!{<div class="flex-colum"><span class="flex\u{2d}colum"/></div>};rsx_strict!{<div class="flex-colum"/>};}"##;
    let analysis = analyze_rust_document(source, ClassMode::PERMISSIVE);
    assert_eq!(analysis.class_tokens.len(), 2);
    assert_eq!(analysis.diagnostics.len(), 3);
    assert_eq!(
        analysis.diagnostics[0].severity,
        DiagnosticSeverity::Warning
    );
    assert!(analysis.diagnostics[0].suggestion.is_some());
    let escaped = &analysis.diagnostics[1];
    assert_eq!(&source[escaped.start..escaped.end], r#""flex\u{2d}colum""#);
    assert_eq!(escaped.suggestion, None);
    assert_eq!(analysis.diagnostics[2].severity, DiagnosticSeverity::Error);
}

#[test]
fn analysis_results_do_not_outlive_their_theme_request() {
    let source = r#"fn view(){rsx!{<div class="bg-brand"/>};}"#;
    for (color, rgba) in [("#123456", 0x123456ff), ("#abcdef", 0xabcdefff)] {
        let theme = Theme::parse(&format!("[colors]\nbrand=\"{color}\"\n")).unwrap();
        let mode = ClassMode::PERMISSIVE.with_theme(Some(theme.into()));
        let analysis = analyze_rust_document(source, mode);
        assert_eq!(analysis.class_tokens[0].rgba, Some(rgba));
        assert_eq!(analysis.class_tokens[0].kind, ClassTokenKind::Theme);
    }
    let analysis = analyze_rust_document(source, ClassMode::STRICT);
    assert_eq!(analysis.class_tokens[0].kind, ClassTokenKind::Invalid);
    assert_eq!(analysis.diagnostics[0].severity, DiagnosticSeverity::Error);
}

#[test]
fn html_escaping_preserves_unicode_and_escapes_each_original_character_once() {
    for text in [
        "",
        "plain 中文😀",
        "&<>'\"",
        "&amp;",
        "😀<中文&\"é'>尾",
        "&&<<>>''\"\"",
    ] {
        let expected = text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;");
        assert_eq!(gpui_rsx_tools::preview::escape(text), expected);
    }
}
