//! Semi-automatic JSX/TSX conversion using SWC. JavaScript behavior is never executed.
use gpui_rsx_core::{ClassMode, ClassOutcome, analyze_class};
use serde::Serialize;
use swc_common::{FileName, SourceMap, Span, Spanned, sync::Lrc};
use swc_ecma_ast::*;
use swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax, lexer::Lexer};
use swc_ecma_visit::{Visit, VisitWith};

#[derive(Debug, Serialize)]
pub struct Conversion {
    pub rsx: String,
    pub diagnostics: Vec<ConversionDiagnostic>,
}
#[derive(Debug, Serialize)]
pub struct ConversionDiagnostic {
    pub start: usize,
    pub end: usize,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct ConversionOptions {
    pub tailwind: bool,
}

struct Converter {
    tailwind: bool,
    base: u32,
    diagnostics: Vec<ConversionDiagnostic>,
    roots: Vec<String>,
}
impl Converter {
    fn note(&mut self, span: Span, message: impl Into<String>) {
        self.diagnostics.push(ConversionDiagnostic {
            start: span.lo.0.saturating_sub(self.base) as usize,
            end: span.hi.0.saturating_sub(self.base) as usize,
            message: message.into(),
        });
    }
    fn element(&mut self, el: &JSXElement) -> String {
        let name = match &el.opening.name {
            JSXElementName::Ident(name) if is_rust_ident(&name.sym) => name.sym.to_string(),
            JSXElementName::Ident(name) => {
                self.note(
                    el.span,
                    format!(
                        "`<{}>` is not a valid Rust tag name; replaced with div",
                        name.sym
                    ),
                );
                "div".into()
            }
            _ => {
                self.note(
                    el.span,
                    "Namespaced/member component needs a Rust constructor; replaced with div",
                );
                "div".into()
            }
        };
        let mut result = format!("<{name}");
        for attr in &el.opening.attrs {
            match attr {
                JSXAttrOrSpread::SpreadElement(spread) => {
                    self.note(spread.span(), "JSX spread props require manual conversion")
                }
                JSXAttrOrSpread::JSXAttr(attr) => {
                    let key = match &attr.name {
                        JSXAttrName::Ident(i) => i.sym.to_string(),
                        _ => {
                            self.note(attr.span, "Namespaced attribute requires manual conversion");
                            continue;
                        }
                    };
                    let key = if key == "className" { "class" } else { &key };
                    // `aria-label` or `type` would make the whole output unparsable.
                    if !is_rust_ident(key) {
                        self.note(
                            attr.span,
                            format!("`{key}` is not a valid Rust attribute name; omitted"),
                        );
                        continue;
                    }
                    if key.starts_with("on")
                        || matches!(key, "style" | "ref" | "dangerouslySetInnerHTML")
                    {
                        self.note(
                            attr.span,
                            format!("`{key}` requires a Rust/GPUI implementation; omitted"),
                        );
                        continue;
                    }
                    if let Some(JSXAttrValue::Str(text)) = &attr.value {
                        let value = text.value.to_string_lossy().to_string();
                        if key == "class" {
                            let mut supported = Vec::new();
                            for class in value.split_ascii_whitespace() {
                                let class = if self.tailwind {
                                    class.to_owned()
                                } else {
                                    tailwind_to_px(class)
                                };
                                if matches!(
                                    analyze_class(
                                        &class,
                                        ClassMode::STRICT.with_scale(if self.tailwind {
                                            gpui_rsx_core::SpacingScale::Tailwind
                                        } else {
                                            gpui_rsx_core::SpacingScale::Px
                                        })
                                    )
                                    .outcome,
                                    ClassOutcome::Supported(_)
                                ) {
                                    supported.push(class.clone());
                                } else {
                                    self.note(attr.span,format!("Unsupported class `{class}` omitted; use a GPUI method or theme token"));
                                }
                            }
                            result
                                .push_str(&format!(" class={}", rust_string(&supported.join(" "))));
                        } else {
                            result.push_str(&format!(" {key}={{{}}}", rust_string(&value)));
                        }
                    } else if let Some(JSXAttrValue::JSXExprContainer(container)) = &attr.value
                        && let JSXExpr::Expr(expr) = &container.expr
                        && let Some(expr) = simple_expression(expr)
                    {
                        result.push_str(&format!(" {key}={{{expr}}}"));
                        if key == "class" && !self.tailwind {
                            self.note(attr.span, "Dynamic class strings keep their numeric values; review pixel spacing or use Tailwind output");
                        }
                    } else if attr.value.is_none() {
                        self.note(
                            attr.span,
                            format!("Boolean prop `{key}` needs a GPUI method; omitted"),
                        );
                    } else {
                        self.note(
                            attr.span,
                            format!(
                                "Expression prop `{key}` needs manual Rust conversion; omitted"
                            ),
                        );
                    }
                }
            }
        }
        if el.children.is_empty() {
            result.push_str(" />");
        } else {
            result.push('>');
            result.push_str(&self.children(&el.children));
            result.push_str(&format!("</{name}>"));
        }
        result
    }
    fn children(&mut self, children: &[JSXElementChild]) -> String {
        let mut result = String::new();
        for child in children {
            match child {
                JSXElementChild::JSXElement(el) => result.push_str(&self.element(el)),
                JSXElementChild::JSXFragment(fragment) => {
                    result.push_str("<>");
                    result.push_str(&self.children(&fragment.children));
                    result.push_str("</>");
                }
                JSXElementChild::JSXText(text) => {
                    let value = jsx_text(&text.value.to_string_lossy());
                    if !value.is_empty() {
                        result.push_str(&format!("{{{}}}", rust_string(&value)));
                    }
                }
                JSXElementChild::JSXExprContainer(container) => {
                    if let JSXExpr::Expr(expr) = &container.expr {
                        if let Expr::Lit(Lit::Str(text)) = &**expr {
                            result.push_str(&format!(
                                "{{{}}}",
                                rust_string(&text.value.to_string_lossy())
                            ));
                        } else if let Expr::Lit(Lit::Num(number)) = &**expr {
                            result.push_str(&format!(
                                "{{{}}}",
                                rust_string(&number.value.to_string())
                            ));
                        } else if let Some(expr) = simple_expression(expr) {
                            result.push_str(&format!("{{{expr}}}"));
                        } else {
                            self.note(container.span,"JavaScript expression replaced with a text placeholder; implement in Rust");
                            result.push_str("{\"TODO: JavaScript expression\"}");
                        }
                    }
                }
                JSXElementChild::JSXSpreadChild(spread) => {
                    self.note(spread.span(), "Spread children need a Rust iterator");
                    result.push_str("{\"TODO: spread children\"}");
                }
            }
        }
        result
    }
}
impl Visit for Converter {
    fn visit_jsx_element(&mut self, el: &JSXElement) {
        let root = self.element(el);
        self.roots.push(root);
    }
    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment) {
        let root = format!("<>{}</>", self.children(&fragment.children));
        self.roots.push(root);
    }
}
fn jsx_text(text: &str) -> String {
    let lines: Vec<_> = text.split('\n').collect();
    let mut out = String::new();
    for (index, line) in lines.iter().enumerate() {
        let line = line.replace('\t', " ");
        let line = if index > 0 { line.trim_start() } else { &line };
        let line = if index + 1 < lines.len() {
            line.trim_end()
        } else {
            line
        };
        if line.is_empty() {
            continue;
        }
        if !out.is_empty() && index > 0 {
            out.push(' ');
        }
        out.push_str(line);
    }
    out
}

fn rust_string(text: &str) -> String {
    format!("{:?}", text)
}

/// Accepts a JSX expression or an entire TSX module and extracts its JSX roots.
pub fn convert(source: &str) -> Result<Conversion, String> {
    convert_with_options(source, ConversionOptions::default())
}

/// Tailwind output preserves numeric utility values and wraps the body in `rsx_tw!`.
pub fn convert_with_options(
    source: &str,
    options: ConversionOptions,
) -> Result<Conversion, String> {
    let map: Lrc<SourceMap> = Default::default();
    let file = map.new_source_file(
        FileName::Custom("input.tsx".into()).into(),
        source.to_owned(),
    );
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*file),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module().map_err(|e| {
        format!(
            "JSX parse error at byte {}: {:?}",
            e.span().lo.0.saturating_sub(file.start_pos.0),
            e.kind()
        )
    })?;
    if let Some(e) = parser.take_errors().first() {
        return Err(format!("JSX parse error: {:?}", e.kind()));
    }
    let mut converter = Converter {
        tailwind: options.tailwind,
        base: file.start_pos.0,
        diagnostics: Vec::new(),
        roots: Vec::new(),
    };
    module.visit_with(&mut converter);
    if converter.roots.is_empty() {
        return Err("No JSX element found".into());
    }
    let joined = if converter.roots.len() == 1 {
        converter.roots.remove(0)
    } else {
        format!("<>{}</>", converter.roots.join(""))
    };
    let rsx = crate::format::body(&joined)?;
    let rsx = if options.tailwind {
        format!("rsx_tw! {{\n{rsx}\n}}")
    } else {
        rsx
    };
    Ok(Conversion {
        rsx,
        diagnostics: converter.diagnostics,
    })
}

/// RSX tags and attributes parse as plain Rust identifiers (no `-`, no keywords).
fn is_rust_ident(name: &str) -> bool {
    syn::parse_str::<syn::Ident>(name).is_ok()
}

/// JSX uses Tailwind's 4px scale (`p-4` is 16px) while `rsx!` defaults to pixels, so numeric
/// length classes are rescaled to keep the same size (`p-4` → `p-16`).
fn tailwind_to_px(class: &str) -> String {
    let (variant, base) = class.rsplit_once(':').map_or(("", class), |(v, b)| (v, b));
    if let Some((_, value, _)) = gpui_rsx_core::codegen::tables::split_length_class(base)
        && let Ok(n) = value.parse::<f32>()
        && n.is_finite()
    {
        let prefix = &base[..base.len() - value.len()];
        let separator = if variant.is_empty() { "" } else { ":" };
        return format!("{variant}{separator}{prefix}{}", n * 4.0);
    }
    class.to_owned()
}

// These expressions share Rust syntax. Calls, operators, indexing and optional chaining
// require semantic conversion and deliberately retain a review note/placeholder.
fn simple_expression(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Ident(ident)
            if is_rust_ident(&ident.sym)
                && !matches!(
                    ident.sym.as_ref(),
                    "undefined" | "NaN" | "Infinity" | "arguments"
                ) =>
        {
            Some(ident.sym.to_string())
        }
        Expr::Member(member) => {
            let MemberProp::Ident(property) = &member.prop else {
                return None;
            };
            is_rust_ident(&property.sym).then_some(())?;
            Some(format!(
                "{}.{}",
                simple_expression(&member.obj)?,
                property.sym
            ))
        }
        _ => None,
    }
}
