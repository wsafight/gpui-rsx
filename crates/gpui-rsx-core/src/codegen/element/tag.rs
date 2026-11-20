//! Tag construction and child-slot routing helpers for RSX elements.
use proc_macro2::TokenStream;
use quote::quote;

use crate::SemanticDiagnostic;
use crate::codegen::key::make_auto_id;
use crate::codegen::tables::HTML_TAGS;
use crate::parser::{RsxAttribute, RsxElement, RsxElementName};
use crate::semantic::CodegenErrors;

#[derive(Default)]
pub(super) struct TagInputs<'a> {
    pub args: Option<&'a [syn::Expr]>,
    pub img_source: Option<&'a syn::Expr>,
    pub canvas_prepaint: Option<&'a syn::Expr>,
    pub canvas_paint: Option<&'a syn::Expr>,
    pub list_id: Option<TokenStream>,
    pub list_count: Option<&'a syn::Expr>,
    pub list_render: Option<&'a syn::Expr>,
}

pub(super) fn is_native_tag(name: &RsxElementName, tag_str: &str) -> bool {
    name.as_single_ident().is_some()
        && (HTML_TAGS.contains(&tag_str)
            || matches!(tag_str, "svg" | "img" | "canvas" | "uniform_list"))
}

pub(super) fn extract_child_slot_ident(element: &RsxElement) -> Option<syn::Ident> {
    let slot_lit = element.attributes.iter().find_map(|attr| match attr {
        RsxAttribute::Value {
            name,
            value:
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }),
        } if name == "slot" => Some(lit),
        _ => None,
    })?;
    let raw = slot_lit.value();
    let mut snake = String::with_capacity(raw.len() + 2);
    for (idx, ch) in raw.chars().enumerate() {
        if ch == '-' {
            snake.push('_');
        } else if ch.is_ascii_uppercase() {
            if idx > 0 {
                snake.push('_');
            }
            snake.push(ch.to_ascii_lowercase());
        } else {
            snake.push(ch);
        }
    }
    syn::parse_str::<syn::Ident>(&snake).ok().map(|mut ident| {
        ident.set_span(slot_lit.span());
        ident
    })
}

/// HTML 标签 → `div()`，特殊标签 → 同名函数，自定义组件 → 同名函数调用或 `Type::new(args...)`
pub(super) fn generate_tag(
    tag_str: &str,
    name: &RsxElementName,
    inputs: TagInputs<'_>,
) -> Result<TokenStream, CodegenErrors> {
    if let Some(args) = inputs.args {
        if is_native_tag(name, tag_str) {
            let err =
                crate::diagnostics::unsupported_native_constructor_args_error(&name.path, tag_str);
            return Err(CodegenErrors {
                tokens: err.to_compile_error(),
                diagnostics: Vec::new(),
            });
        }
        let path = &name.path;
        let is_type_ctor = path.segments.last().is_some_and(|seg| {
            use syn::ext::IdentExt;
            seg.ident
                .unraw()
                .to_string()
                .starts_with(|c: char| c.is_ascii_uppercase())
        });
        return Ok(if is_type_ctor {
            quote! { #path::new(#(#args),*) }
        } else {
            quote! { #path(#(#args),*) }
        });
    }

    if name.as_single_ident().is_none() {
        let path = &name.path;
        return Ok(quote! { #path() });
    }

    let path = &name.path;
    Ok(match tag_str {
        "svg" => quote! { svg() },
        "img" => {
            let Some(source) = inputs.img_source else {
                return Err(SemanticDiagnostic::missing_required_attribute(
                    &name.path,
                    "img",
                    "src",
                    r#"<img src={"path/to/image.png"} />"#,
                )
                .into());
            };
            quote! { img(#source) }
        }
        "canvas" => {
            let Some(prepaint) = inputs.canvas_prepaint else {
                return Err(SemanticDiagnostic::missing_required_attribute(
                    &name.path,
                    "canvas",
                    "prepaint",
                    r#"<canvas prepaint={|bounds, window, cx| state} paint={|bounds, state, window, cx| { ... }} />"#,
                )
                .into());
            };
            let Some(paint) = inputs.canvas_paint else {
                return Err(SemanticDiagnostic::missing_required_attribute(
                    &name.path,
                    "canvas",
                    "paint",
                    r#"<canvas prepaint={|bounds, window, cx| state} paint={|bounds, state, window, cx| { ... }} />"#,
                )
                .into());
            };
            quote! { canvas(#prepaint, #paint) }
        }
        "uniform_list" => {
            let Some(count) = inputs.list_count else {
                return Err(SemanticDiagnostic::missing_required_attribute(
                    &name.path,
                    "uniform_list",
                    "count",
                    r#"<uniform_list count={items.len()} render={|range, window, cx| { ... }} />"#,
                )
                .into());
            };
            let Some(render) = inputs.list_render else {
                return Err(SemanticDiagnostic::missing_required_attribute(
                    &name.path,
                    "uniform_list",
                    "render",
                    r#"<uniform_list count={items.len()} render={|range, window, cx| { ... }} />"#,
                )
                .into());
            };
            let id = inputs.list_id.unwrap_or_else(|| make_auto_id(name));
            quote! { uniform_list(#id, #count, #render) }
        }
        tag if HTML_TAGS.contains(&tag) => quote! { div() },
        _ => quote! { #path() },
    })
}
