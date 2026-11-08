//! Source locations and quick fixes supplied by core semantic analysis.
use crate::{Diagnostic, DiagnosticSeverity, byte_offset};
use gpui_rsx_core::SemanticDiagnostic;

pub(crate) fn semantic_errors(
    diagnostics: Vec<SemanticDiagnostic>,
    source: &str,
    out: &mut Vec<Diagnostic>,
) {
    out.extend(diagnostics.into_iter().map(|diagnostic| Diagnostic {
        start: byte_offset(source, diagnostic.span, false),
        end: byte_offset(source, diagnostic.span, true),
        severity: DiagnosticSeverity::Error,
        code: Some(diagnostic.code.as_str().to_owned()),
        message: diagnostic.message,
        suggestion: diagnostic.suggestion,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_rsx_core::{ClassMode, RsxBody, generate_body_with_diagnostics};

    #[test]
    fn quick_fixes_do_not_depend_on_error_wording() {
        let source = "<div>{for item in items {<button onClick={handler}/>}}</div>";
        let body: RsxBody = syn::parse_str(source).unwrap();
        let mut expansion = generate_body_with_diagnostics(&body, ClassMode::PERMISSIVE);
        expansion.diagnostics[0].message = "提示文案可以独立修改".into();
        let mut diagnostics = Vec::new();
        semantic_errors(expansion.diagnostics, source, &mut diagnostics);
        let diagnostic = &diagnostics[0];
        assert_eq!(diagnostic.code.as_deref(), Some("rsx/missing-loop-key"));
        let mut fixed = source.to_owned();
        fixed.replace_range(
            diagnostic.start..diagnostic.end,
            diagnostic.suggestion.as_deref().unwrap(),
        );
        assert!(crate::analyze_body(&fixed, ClassMode::PERMISSIVE).is_empty());
    }

    #[test]
    fn old_json_still_deserializes_and_omits_absent_codes() {
        let json = r#"{"start":0,"end":1,"severity":"error","message":"existing client"}"#;
        let diagnostic: Diagnostic = serde_json::from_str(json).unwrap();
        assert!(diagnostic.code.is_none());
        assert!(
            serde_json::to_value(diagnostic)
                .unwrap()
                .get("code")
                .is_none()
        );
    }

    #[test]
    fn application_compile_errors_are_not_misreported_as_rsx_semantics() {
        let source = "<div>{compile_error!(\"has no `id` or `key`\")}</div>";
        assert!(crate::analyze_body(source, ClassMode::PERMISSIVE).is_empty());
    }
}
