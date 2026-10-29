//! Analysis-time class classifications reused by semantic tokens and color requests.
use gpui_rsx_core::{ClassMode, ClassOutcome, StyleValue, analyze_class, resolve_style};

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

pub(crate) fn collect(
    source: &str,
    literals: &[(usize, usize)],
    mode: ClassMode,
) -> Vec<ClassToken> {
    let mut tokens = Vec::new();
    for &(start, end) in literals {
        let value = &source[start..end];
        let mut at = 0;
        for class in value.split_ascii_whitespace() {
            let begin = at + value[at..].find(class).unwrap_or(0);
            at = begin + class.len();
            let base = class.rsplit(':').next().unwrap_or(class);
            let kind = match analyze_class(class, mode.clone()).outcome {
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
            tokens.push(ClassToken {
                start: start + begin,
                end: start + at,
                kind,
                rgba,
            });
        }
    }
    tokens
}
