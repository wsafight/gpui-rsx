//! RSX 语法解析器
//!
//! 解析类似 JSX 的语法结构

use crate::codegen::tables::is_stateful_class;
use crate::diagnostics::*;
use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::fmt;
use syn::{
    Expr, ExprLit, Ident, Lit, Pat, Result, Token,
    parse::{Parse, ParseStream, Parser},
    spanned::Spanned,
    token,
};

/// RSX 宏体
///
/// 可以是单个元素或 Fragment（多根节点）
pub enum RsxBody {
    /// 单个元素，如 `<div>...</div>`
    Single(RsxElement),
    /// Fragment，如 `<>.....</>`
    Fragment(Vec<RsxNode>),
}

/// RSX 元素
///
/// 表示一个 HTML-like 元素，如 `<div class="container">...</div>`
pub struct RsxElement {
    pub name: RsxElementName,
    pub attributes: Vec<RsxAttribute>,
    pub children: Vec<RsxNode>,
}

/// RSX 元素名，支持单段 tag（`div`、`Button`）和路径型 tag（`ui::TaskCard`）。
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

/// RSX 属性
///
/// 表示元素的属性，如 `class="container"` 或 `onClick={handler}`
pub enum RsxAttribute {
    /// 布尔属性，如 `flex`
    Flag(Ident),
    /// 值属性，如 `gap={px(16.0)}`
    Value { name: Ident, value: Expr },
    /// when 条件渲染，如 `when={(condition, |this| this.bg(...))}`
    When { condition: Expr, closure: Expr },
    /// when_some 条件渲染，如 `whenSome={(option, |this, value| ...)}`
    WhenSome { option: Expr, closure: Expr },
    /// whenClass 条件 class，如 `whenClass={(active, "bg-blue-500 text-white")}`
    WhenClass {
        condition: Expr,
        class_lit: syn::LitStr,
    },
    /// GPUI 状态样式 class，如 `hoverClass="bg-blue-500"`
    StateClass {
        method: Ident,
        class_lit: syn::LitStr,
    },
}

/// RSX 节点
///
/// 可以是元素、表达式、展开或 for 循环
pub enum RsxNode {
    /// 子元素
    Element(RsxElement),
    /// 表达式（文本或其他）
    Expr(Expr),
    /// 展开子节点列表，如 `{...iter}`
    Spread(Expr),
    /// for 循环语法糖，如 `{for item in iter { <child /> }}`
    For {
        binding: Box<Pat>,
        iter: Box<Expr>,
        body: Vec<RsxNode>,
    },
}

impl Parse for RsxBody {
    fn parse(input: ParseStream) -> Result<Self> {
        // 检查是否是 Fragment: <>...</>
        if input.peek(Token![<]) && input.peek2(Token![>]) {
            // 解析 <>
            input.parse::<Token![<]>()?;
            input.parse::<Token![>]>()?;

            // 解析子节点
            let children = parse_children(input, None)?;

            // 解析 </>
            input.parse::<Token![<]>()?;
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;

            Ok(RsxBody::Fragment(children))
        } else {
            // 单个元素
            let element: RsxElement = input.parse()?;
            Ok(RsxBody::Single(element))
        }
    }
}

impl Parse for RsxElement {
    fn parse(input: ParseStream) -> Result<Self> {
        // 解析开始标签 <tag
        input.parse::<Token![<]>()?;
        let name = RsxElementName::parse(input)?;

        // 解析属性（预分配容量，典型元素有 3-8 个属性）
        let mut attributes = Vec::with_capacity(4);
        while !input.peek(Token![>]) && !input.peek(Token![/]) {
            let attr_name: Ident = input.parse()?;

            if input.peek(Token![=]) {
                // 值属性: name={value}
                input.parse::<Token![=]>()?;
                let value: Expr = if input.peek(token::Brace) {
                    // {expression} — 大括号内解析完整表达式
                    let content;
                    syn::braced!(content in input);
                    content.parse()?
                } else {
                    // 非大括号值只接受字面量（如 "string"、42）。
                    // 不能用 Expr::parse，否则它会贪婪消费后续的 / > 等运算符。
                    let lit: syn::Lit = input.parse()?;
                    syn::Expr::Lit(syn::ExprLit { attrs: vec![], lit })
                };

                // 特殊处理 when 和 whenSome 属性（直接比较 Ident，避免 to_string() 分配）
                if is_group_drag_over_attr(&attr_name) {
                    return Err(unsupported_generic_attribute_error(&attr_name));
                } else if attr_name == "whiteSpace" {
                    return Err(unsupported_jsx_attribute_error(&attr_name));
                } else if attr_name == "when" {
                    let (first, second) = parse_condition_tuple(value, "when")?;
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
                // 布尔属性: name
                if is_group_drag_over_attr(&attr_name) {
                    return Err(unsupported_generic_attribute_error(&attr_name));
                }
                if attr_name == "whiteSpace" {
                    return Err(unsupported_jsx_attribute_error(&attr_name));
                }
                attributes.push(RsxAttribute::Flag(attr_name));
            }
        }

        // 检查是否是自闭合标签 />
        let self_closing = if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            true
        } else {
            input.parse::<Token![>]>()?;
            false
        };

        // 解析子节点
        let children = if self_closing {
            Vec::new()
        } else {
            let children = parse_children(input, Some(&name))?;

            // 解析闭合标签 </tag>
            input.parse::<Token![<]>()?;
            input.parse::<Token![/]>()?;
            let closing_name = RsxElementName::parse(input)?;
            input.parse::<Token![>]>()?;

            // 验证标签名称匹配
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

/// 解析子节点列表
///
/// `parent_name` 为 None 时表示 Fragment 上下文，
/// 为 Some 时表示某个命名元素的子节点。
fn parse_children(
    input: ParseStream,
    parent_name: Option<&RsxElementName>,
) -> Result<Vec<RsxNode>> {
    let mut children = Vec::with_capacity(4);

    loop {
        // 检查是否到达闭合标签
        if input.peek(Token![<]) && input.peek2(Token![/]) {
            break;
        }

        // 检查是否已经没有内容了
        if input.is_empty() {
            return Err(match parent_name {
                Some(name) => unclosed_tag_error(input.span(), &name.to_string()),
                None => unclosed_fragment_error(input.span()),
            });
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

/// 尝试从输入流中解析单个子节点
///
/// 处理所有子节点类型：`{expr}`, `{...spread}`, `{for ...}`, `<element>`, `"string"`。
/// 如果当前 token 不匹配任何已知类型，返回 `Ok(None)`，由调用方决定如何报错。
fn try_parse_child_node(input: ParseStream) -> Result<Option<RsxNode>> {
    if input.peek(token::Brace) {
        let content;
        syn::braced!(content in input);

        if content.peek(Token![..]) {
            // Rust tokenizer 将 `...` 分割为 `..` (Range) 和 `.` (Dot)，
            // 因此需要分两步解析，这是 proc-macro 中处理 `...` 的标准方式。
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

/// 解析 for 循环: `for item in iter { <child /> ... }`
fn parse_for_loop(content: ParseStream) -> Result<RsxNode> {
    content.parse::<Token![for]>()?;

    // 解析绑定模式（支持简单 ident 和元组解构等）
    let binding: Pat = Pat::parse_single(content)?;

    content.parse::<Token![in]>()?;

    // 解析剩余 token：最后一个顶层 `{...}` 是 RSX body，前面的 token 是 iterator expr。
    // 这样 `for item in { items.iter() } { ... }` 这类 iterator block 不会被提前截断。
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

/// 解析条件属性（when/whenSome）的元组值 `(first, second)`
fn parse_condition_tuple(value: Expr, attr_name: &str) -> Result<(Expr, Expr)> {
    if let Expr::Tuple(tuple) = value {
        if tuple.elems.len() == 2 {
            let mut iter = tuple.elems.into_iter();
            // len() == 2 已在上方确认，next() 不可能返回 None
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
