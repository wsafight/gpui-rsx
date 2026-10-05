//! RSX syntax parser
//!
//! Parses JSX-like syntax structures

use crate::codegen::tables::is_stateful_class;
use crate::diagnostics::*;
use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::fmt;
use syn::{
    Expr, ExprLit, Ident, Lit, Pat, Result, Token,
    ext::IdentExt,
    parse::{Parse, ParseStream, Parser},
    spanned::Spanned,
    token,
};

/// RSX macro body
///
/// Can be a single element or a Fragment (multiple root nodes)
pub enum RsxBody {
    /// Single element, e.g. `<div>...</div>`
    Single(RsxElement),
    /// Fragment, e.g. `<>.....</>`
    Fragment(Vec<RsxNode>),
}

/// RSX element
///
/// Represents an HTML-like element, e.g. `<div class="container">...</div>`
pub struct RsxElement {
    pub name: RsxElementName,
    pub attributes: Vec<RsxAttribute>,
    pub children: Vec<RsxNode>,
}

/// RSX element name, supporting single-segment tags (`div`, `Button`) and path-style tags (`ui::TaskCard`).
pub struct RsxElementName {
    pub path: syn::Path,
}

impl RsxElementName {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            path: input.call(syn::Path::parse_mod_style)?,
        })
    }

    pub fn as_single_ident(&self) -> Option<&Ident> {
        (self.path.leading_colon.is_none() && self.path.segments.len() == 1)
            .then(|| &self.path.segments[0].ident)
    }

    pub fn span(&self) -> Span {
        self.path.span()
    }

    fn display_name(&self) -> String {
        let mut out = String::new();
        if self.path.leading_colon.is_some() {
            out.push_str("::");
        }
        for (index, segment) in self.path.segments.iter().enumerate() {
            if index > 0 {
                out.push_str("::");
            }
            out.push_str(&segment.ident.to_string());
        }
        out
    }
}

impl fmt::Display for RsxElementName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display_name())
    }
}

impl ToTokens for RsxElementName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.path.to_tokens(tokens);
    }
}

/// RSX attribute
///
/// Represents an element attribute, e.g. `class="container"` or `onClick={handler}`
pub enum RsxAttribute {
    /// Boolean attribute, e.g. `flex`
    Flag(Ident),
    /// Value attribute, e.g. `gap={px(16.0)}`
    Value { name: Ident, value: Expr },
    /// `when` conditional rendering, e.g. `when={(condition, |this| this.bg(...))}`
    When { condition: Expr, closure: Expr },
    /// `when_some` conditional rendering, e.g. `whenSome={(option, |this, value| ...)}`
    WhenSome { option: Expr, closure: Expr },
    /// `whenClass` conditional class, e.g. `whenClass={(active, "bg-blue-500 text-white")}`
    WhenClass {
        condition: Expr,
        class_lit: syn::LitStr,
    },
    /// GPUI state style class, e.g. `hoverClass="bg-blue-500"`
    StateClass {
        method: Ident,
        class_lit: syn::LitStr,
    },
}

/// RSX node
///
/// Can be an element, expression, spread, or for-loop
pub enum RsxNode {
    /// Child element
    Element(RsxElement),
    /// Expression (text or other)
    Expr(Expr),
    /// Spread child node list, e.g. `{...iter}`
    Spread(Expr),
    /// for-loop syntactic sugar, e.g. `{for item in iter { <child /> }}`
    For {
        binding: Box<Pat>,
        iter: Box<Expr>,
        body: Vec<RsxNode>,
    },
}

impl Parse for RsxBody {
    fn parse(input: ParseStream) -> Result<Self> {
        // Check if it is a Fragment: <>...</>
        if input.peek(Token![<]) && input.peek2(Token![>]) {
            // Parse <>
            input.parse::<Token![<]>()?;
            input.parse::<Token![>]>()?;

            // Parse child nodes
            let children = parse_children(input, None)?;

            // Parse </>
            input.parse::<Token![<]>()?;
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;

            Ok(RsxBody::Fragment(children))
        } else {
            // Single element
            let element: RsxElement = input.parse()?;
            Ok(RsxBody::Single(element))
        }
    }
}

impl Parse for RsxElement {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse opening tag <tag
        input.parse::<Token![<]>()?;
        let name = RsxElementName::parse(input)?;

        // Parse attributes (pre-allocated capacity; typical elements have 3-8 attributes)
        let mut attributes = Vec::with_capacity(4);
        while !input.peek(Token![>]) && !input.peek(Token![/]) {
            let attr_name = syn::Ident::parse_any(input)?;

            if input.peek(Token![=]) {
                // Value attribute: name={value}
                input.parse::<Token![=]>()?;
                let value: Expr = if input.peek(token::Brace) {
                    // {expression} - parse full expression inside braces
                    let content;
                    syn::braced!(content in input);
                    content.parse()?
                } else {
                    // Non-braced values only accept literals (e.g. "string", 42).
                    // Cannot use Expr::parse, otherwise it greedily consumes trailing / > operators.
                    let lit: syn::Lit = input.parse()?;
                    syn::Expr::Lit(syn::ExprLit { attrs: vec![], lit })
                };

                // Special handling for when and whenSome attributes (compare Ident directly to avoid to_string() allocation)
                if is_group_drag_over_attr(&attr_name) {
                    return Err(unsupported_generic_attribute_error(&attr_name));
                } else if attr_name == "whiteSpace" {
                    return Err(unsupported_jsx_attribute_error(&attr_name));
                } else if attr_name == "when" {
                    let (first, second) = if let Expr::Tuple(tuple) = &value {
                        if tuple.elems.len() == 2 {
                            let mut iter = tuple.elems.iter();
                            (iter.next().unwrap().clone(), iter.next().unwrap().clone())
                        } else {
                            return Err(condition_tuple_wrong_count_error(
                                &value,
                                "when",
                                tuple.elems.len(),
                            ));
                        }
                    } else {
                        let condition = value;
                        let closure: Expr = syn::parse_quote!(|__el| __el);
                        (condition, closure)
                    };
                    attributes.push(RsxAttribute::When {
                        condition: first,
                        closure: second,
                    });
                } else if attr_name == "whenSome" {
                    let (first, second) = parse_condition_tuple(value, "whenSome")?;
                    attributes.push(RsxAttribute::WhenSome {
                        option: first,
                        closure: second,
                    });
                } else if attr_name == "whenClass" {
                    let (condition, class_expr) = parse_condition_tuple(value, "whenClass")?;
                    let class_lit = parse_when_class_lit(class_expr)?;
                    attributes.push(RsxAttribute::WhenClass {
                        condition,
                        class_lit,
                    });
                } else if let Some(state_method) = state_class_method(&attr_name) {
                    let class_lit = parse_state_class_lit(value, &attr_name)?;
                    attributes.push(RsxAttribute::StateClass {
                        method: Ident::new(state_method, attr_name.span()),
                        class_lit,
                    });
                } else {
                    attributes.push(RsxAttribute::Value {
                        name: attr_name,
                        value,
                    });
                }
            } else {
                // Boolean attribute: name
                if is_group_drag_over_attr(&attr_name) {
                    return Err(unsupported_generic_attribute_error(&attr_name));
                }
                if attr_name == "whiteSpace" {
                    return Err(unsupported_jsx_attribute_error(&attr_name));
                }
                attributes.push(RsxAttribute::Flag(attr_name));
            }
        }

        // Check if it is a self-closing tag />
        let self_closing = if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            true
        } else {
            input.parse::<Token![>]>()?;
            false
        };

        // Parse child nodes
        let children = if self_closing {
            Vec::new()
        } else {
            let children = parse_children(input, Some(&name))?;

            // Parse closing tag </tag>
            input.parse::<Token![<]>()?;
            input.parse::<Token![/]>()?;
            let closing_name = RsxElementName::parse(input)?;
            input.parse::<Token![>]>()?;

            // Verify tag names match
            let opening_display = name.to_string();
            let closing_display = closing_name.to_string();
            if opening_display != closing_display {
                return Err(tag_mismatch_error(
                    &closing_name.path,
                    &closing_display,
                    &opening_display,
                ));
            }

            children
        };

        Ok(RsxElement {
            name,
            attributes,
            children,
        })
    }
}

/// Parses child node list
///
/// `parent_name` being None indicates a Fragment context,
/// while Some indicates child nodes of a named element.
fn parse_children(
    input: ParseStream,
    parent_name: Option<&RsxElementName>,
) -> Result<Vec<RsxNode>> {
    let mut children = Vec::with_capacity(4);

    loop {
        // Check if reaching the closing tag
        if input.peek(Token![<]) && input.peek2(Token![/]) {
            break;
        }

        // Check if there is no more content
        if input.is_empty() {
            return Err(match parent_name {
                Some(name) => unclosed_tag_error(input.span(), &name.to_string()),
                None => unclosed_fragment_error(input.span()),
            });
        }

        if input.peek(Token![<]) && input.peek2(Token![>]) {
            input.parse::<Token![<]>()?;
            input.parse::<Token![>]>()?;
            let nested = parse_children(input, None)?;
            input.parse::<Token![<]>()?;
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            children.extend(nested);
            continue;
        }

        if let Some(node) = try_parse_child_node(input)? {
            children.push(node);
        } else {
            return Err(match parent_name {
                Some(name) => invalid_child_in_tag_error(input.span(), &name.to_string()),
                None => invalid_child_in_fragment_error(input.span()),
            });
        }
    }

    Ok(children)
}

/// Attempts to parse a single child node from the input stream
///
/// Handles all child node types: `{expr}`, `{...spread}`, `{for ...}`, `<element>`, `"string"`.
/// If the current token does not match any known type, returns `Ok(None)` and leaves error handling to caller.
fn try_parse_child_node(input: ParseStream) -> Result<Option<RsxNode>> {
    if input.peek(token::Brace) {
        let content;
        syn::braced!(content in input);

        if content.peek(Token![..]) {
            // The Rust tokenizer splits `...` into `..` (Range) and `.` (Dot),
            // so it must be parsed in two steps; this is the standard way to handle `...` in proc-macros.
            content.parse::<Token![..]>()?;
            content.parse::<Token![.]>()?;
            let expr: Expr = content.parse()?;
            Ok(Some(RsxNode::Spread(expr)))
        } else if content.peek(Token![for]) {
            Ok(Some(parse_for_loop(&content)?))
        } else {
            let expr: Expr = content.parse()?;
            Ok(Some(RsxNode::Expr(expr)))
        }
    } else if input.peek(Token![<]) {
        Ok(Some(RsxNode::Element(input.parse()?)))
    } else if input.peek(syn::LitStr) {
        let lit: syn::LitStr = input.parse()?;
        Ok(Some(RsxNode::Expr(Expr::Lit(ExprLit {
            attrs: vec![],
            lit: Lit::Str(lit),
        }))))
    } else {
        Ok(None)
    }
}

/// Parses a for-loop: `for item in iter { <child /> ... }`
fn parse_for_loop(content: ParseStream) -> Result<RsxNode> {
    content.parse::<Token![for]>()?;

    // Parse binding pattern (supports simple ident, tuple destructuring, etc.)
    let binding: Pat = Pat::parse_single(content)?;

    content.parse::<Token![in]>()?;

    // Parse remaining tokens: the last top-level `{...}` is the RSX body, preceding tokens form the iterator expr.
    // This prevents iterator blocks like `for item in { items.iter() } { ... }` from being prematurely truncated.
    let mut remaining = Vec::new();
    while !content.is_empty() {
        remaining.push(content.parse::<TokenTree>()?);
    }

    let body_group = match remaining.pop() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => group,
        Some(tt) => return Err(for_loop_missing_brace_error(tt.span())),
        None => return Err(for_loop_missing_brace_error(content.span())),
    };

    let iter_expr: Expr = syn::parse2(remaining.into_iter().collect())?;

    let body = (|body_content: ParseStream| {
        let mut body = Vec::with_capacity(2);
        while !body_content.is_empty() {
            if let Some(node) = try_parse_child_node(body_content)? {
                body.push(node);
            } else {
                return Err(for_loop_invalid_body_error(body_content.span()));
            }
        }
        Ok(body)
    })
    .parse2(body_group.stream())?;

    Ok(RsxNode::For {
        binding: Box::new(binding),
        iter: Box::new(iter_expr),
        body,
    })
}

/// Parses the tuple value `(first, second)` of conditional attributes (when/whenSome)
fn parse_condition_tuple(value: Expr, attr_name: &str) -> Result<(Expr, Expr)> {
    if let Expr::Tuple(tuple) = value {
        if tuple.elems.len() == 2 {
            let mut iter = tuple.elems.into_iter();
            // len() == 2 confirmed above, next() cannot return None
            let first = iter.next().unwrap();
            let second = iter.next().unwrap();
            Ok((first, second))
        } else {
            Err(condition_tuple_wrong_count_error(
                &tuple,
                attr_name,
                tuple.elems.len(),
            ))
        }
    } else {
        Err(condition_tuple_wrong_type_error(&value, attr_name))
    }
}

fn parse_when_class_lit(value: Expr) -> Result<syn::LitStr> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = value
    {
        if let Some(class) = lit_str
            .value()
            .split_ascii_whitespace()
            .find(|c| is_stateful_class(c))
        {
            return Err(when_class_stateful_error(&lit_str, class));
        }
        Ok(lit_str)
    } else {
        Err(when_class_string_literal_error(&value))
    }
}

fn state_class_method(attr_name: &Ident) -> Option<&'static str> {
    if attr_name == "hoverClass" {
        Some("hover")
    } else if attr_name == "focusClass" {
        Some("focus")
    } else if attr_name == "activeClass" {
        Some("active")
    } else {
        None
    }
}

fn is_group_drag_over_attr(attr_name: &Ident) -> bool {
    attr_name == "groupDragOver" || attr_name == "group_drag_over"
}

fn parse_state_class_lit(value: Expr, attr_name: &Ident) -> Result<syn::LitStr> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = value
    {
        let class_value = lit_str.value();
        if let Some(class) = class_value
            .split_ascii_whitespace()
            .find(|class| is_stateful_class(class) || matches!(*class, "debug-outline"))
        {
            return Err(state_class_unsupported_class_error(
                attr_name, &lit_str, class,
            ));
        }
        Ok(lit_str)
    } else {
        Err(state_class_string_literal_error(attr_name, &value))
    }
}
