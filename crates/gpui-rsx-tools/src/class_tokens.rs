//! Analysis-time class classifications reused by semantic tokens and color requests.
use crate::{BodyAnalysis, Diagnostic, DiagnosticSeverity};
use gpui_rsx_core::{
    ClassDiagnostic, ClassMode, ClassOutcome, Severity, StyleValue, analyze_class, resolve_style,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClassTokenKind {
    Supported,
    Theme,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassToken {
    pub start: usize,
    pub end: usize,
    pub kind: ClassTokenKind,
    pub rgba: Option<u32>,
}

struct ClassInfo {
    kind: ClassTokenKind,
    rgba: Option<u32>,
    diagnostic: Option<ClassDiagnostic>,
}

pub(crate) struct ClassAnalyzer {
    mode: ClassMode,
    classes: HashMap<String, ClassInfo>,
}

impl ClassAnalyzer {
    pub(crate) fn new(mode: ClassMode) -> Self {
        Self {
            mode,
            classes: HashMap::new(),
        }
    }

    pub(crate) fn collect_literal(
        &mut self,
        value: &str,
        literal: (usize, usize),
        content: Option<(usize, usize)>,
        result: &mut BodyAnalysis,
    ) {
        for class in value.split_ascii_whitespace() {
            let begin = class.as_ptr() as usize - value.as_ptr() as usize;
            let mode = &self.mode;
            let info = self.classes.entry(class.to_owned()).or_insert_with(|| {
                let analysis = analyze_class(class, mode.clone());
                let base = class.rsplit(':').next().unwrap_or(class);
                let kind = match analysis.outcome {
                    ClassOutcome::Supported(_)
                        if mode.theme().and_then(|t| t.style_class(base)).is_some() =>
                    {
                        ClassTokenKind::Theme
                    }
                    ClassOutcome::Supported(_) => ClassTokenKind::Supported,
                    _ => ClassTokenKind::Invalid,
                };
                let rgba = resolve_style(base, mode.clone())
                    .and_then(Result::ok)
                    .and_then(|op| match op.value() {
                        Some(StyleValue::Color { hex, alpha }) => {
                            Some(if *alpha { *hex } else { (*hex << 8) | 255 })
                        }
                        _ => None,
                    });
                ClassInfo {
                    kind,
                    rgba,
                    diagnostic: analysis.into_diagnostic(class, mode.clone()),
                }
            });
            if let Some(diagnostic) = &info.diagnostic {
                result.diagnostics.push(Diagnostic {
                    start: content.map_or(literal.0, |(start, _)| start + begin),
                    end: content.map_or(literal.1, |(start, _)| start + begin + class.len()),
                    severity: if diagnostic.severity == Severity::Error {
                        DiagnosticSeverity::Error
                    } else {
                        DiagnosticSeverity::Warning
                    },
                    message: diagnostic.message.clone(),
                    suggestion: content.and(diagnostic.suggestion.clone()),
                });
            }
            if let Some((start, _)) = content {
                result.class_tokens.push(ClassToken {
                    start: start + begin,
                    end: start + begin + class.len(),
                    kind: info.kind,
                    rgba: info.rgba,
                });
            }
        }
    }
}
