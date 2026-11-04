//! Source locations and quick fixes for semantic errors in generated code.
use crate::{Diagnostic, DiagnosticSeverity, byte_offset};
use proc_macro2::{TokenStream, TokenTree};

pub(crate) fn semantic_errors(tokens: TokenStream, source: &str, out: &mut Vec<Diagnostic>) {
    let tokens: Vec<_> = tokens.into_iter().collect();
    for (index, token) in tokens.iter().enumerate() {
        if let TokenTree::Ident(name) = token
            && name == "compile_error"
            && let Some(TokenTree::Group(group)) = tokens.get(index + 2)
            && let Ok(literal) = syn::parse2::<syn::LitStr>(group.stream())
        {
            let diagnostic = Diagnostic {
                start: byte_offset(source, name.span(), false),
                end: byte_offset(source, name.span(), true),
                severity: DiagnosticSeverity::Error,
                message: literal.value(),
                suggestion: literal.value().contains("has no `id` or `key`").then(|| {
                    format!(
                        "{} key={{todo!(\"choose a stable unique key\") as usize}}",
                        &source[byte_offset(source, name.span(), false)
                            ..byte_offset(source, name.span(), true)]
                    )
                }),
            };
            if !out.iter().any(|previous| {
                previous.message == diagnostic.message
                    && (previous.start == diagnostic.start
                        || (previous.start < diagnostic.end && diagnostic.start < previous.end))
            }) {
                out.push(diagnostic);
            }
        }
        if let TokenTree::Group(group) = token {
            semantic_errors(group.stream(), source, out);
        }
    }
}
