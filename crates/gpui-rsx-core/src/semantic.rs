use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;

#[derive(Debug)]
pub(crate) struct CodegenErrors {
    pub(crate) tokens: TokenStream,
    pub(crate) diagnostics: Vec<SemanticDiagnostic>,
}

impl From<SemanticDiagnostic> for CodegenErrors {
    fn from(diagnostic: SemanticDiagnostic) -> Self {
        Self {
            tokens: diagnostic.to_compile_error(),
            diagnostics: vec![diagnostic],
        }
    }
}

/// Helper-free expansion with typed semantic diagnostics and runtime requirements.
pub struct BodyExpansion {
    pub tokens: TokenStream,
    pub uses_inline_runtime: bool,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticDiagnosticCode {
    MissingLoopKey,
    MissingRequiredAttribute,
}

impl SemanticDiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingLoopKey => "rsx/missing-loop-key",
            Self::MissingRequiredAttribute => "rsx/missing-required-attribute",
        }
    }
}

/// A semantic error shared by macro expansion and editor tooling.
#[derive(Clone, Debug)]
pub struct SemanticDiagnostic {
    pub code: SemanticDiagnosticCode,
    pub span: Span,
    pub message: String,
    /// Replacement text for `span`, when a quick fix is available.
    pub suggestion: Option<String>,
}

impl SemanticDiagnostic {
    pub(crate) fn missing_loop_key<T: Spanned>(tag: &T, name: &str) -> Self {
        Self {
            code: SemanticDiagnosticCode::MissingLoopKey,
            span: tag.span(),
            message: crate::diagnostics::for_loop_missing_key_error(tag, name).to_string(),
            suggestion: Some(format!(
                "{name} key={{todo!(\"choose a stable unique key\") as usize}}"
            )),
        }
    }

    pub(crate) fn missing_required_attribute<T: Spanned>(
        tag: &T,
        name: &str,
        required: &str,
        example: &str,
    ) -> Self {
        Self {
            code: SemanticDiagnosticCode::MissingRequiredAttribute,
            span: tag.span(),
            message: crate::diagnostics::missing_required_attribute_error(
                tag, name, required, example,
            )
            .to_string(),
            suggestion: None,
        }
    }

    pub fn to_compile_error(&self) -> proc_macro2::TokenStream {
        syn::Error::new(self.span, &self.message).to_compile_error()
    }
}
