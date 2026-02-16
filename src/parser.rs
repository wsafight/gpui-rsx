//! RSX 语法解析器
//!
//! 解析类似 JSX 的语法结构

use syn::{
    parse::{Parse, ParseStream},
    token, Expr, ExprLit, Ident, Lit, Pat, Result, Token,
};

/// RSX 宏体
///
/// 可以是单个元素或 Fragment（多根节点）
#[derive(Debug)]
pub enum RsxBody {
    /// 单个元素，如 `<div>...</div>`
    Single(RsxElement),
    /// Fragment，如 `<>.....</>`
    Fragment(Vec<RsxNode>),
}

/// RSX 元素
///
/// 表示一个 HTML-like 元素，如 `<div class="container">...</div>`
#[derive(Debug)]
pub struct RsxElement {
    pub name: Ident,
    pub attributes: Vec<RsxAttribute>,
    pub children: Vec<RsxNode>,
    #[allow(dead_code)]
    pub self_closing: bool,
}

/// RSX 属性
///
/// 表示元素的属性，如 `class="container"` 或 `onClick={handler}`
#[derive(Debug)]
pub enum RsxAttribute {
    /// 布尔属性，如 `flex`
    Flag(Ident),
    /// 值属性，如 `gap={px(16.0)}`
    Value { name: Ident, value: Expr },
    /// when 条件渲染，如 `when={(condition, |this| this.bg(...))}`
    When { condition: Expr, closure: Expr },
    /// when_some 条件渲染，如 `whenSome={(option, |this, value| ...)}`
    WhenSome { option: Expr, closure: Expr },
}

/// RSX 节点
///
/// 可以是元素、表达式、展开或 for 循环
#[derive(Debug)]
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
        let name: Ident = input.parse()?;

        // 解析属性
        let mut attributes = Vec::new();
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
                    syn::Expr::Lit(syn::ExprLit {
                        attrs: vec![],
                        lit,
                    })
                };

                // 特殊处理 when 和 whenSome 属性
                let attr_name_str = attr_name.to_string();
                if attr_name_str == "when" {
                    let (first, second) = parse_condition_tuple(value, "when")?;
                    attributes.push(RsxAttribute::When { condition: first, closure: second });
                } else if attr_name_str == "whenSome" {
                    let (first, second) = parse_condition_tuple(value, "whenSome")?;
                    attributes.push(RsxAttribute::WhenSome { option: first, closure: second });
                } else {
                    attributes.push(RsxAttribute::Value {
                        name: attr_name,
                        value,
                    });
                }
            } else {
                // 布尔属性: name
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
            let closing_name: Ident = input.parse()?;
            input.parse::<Token![>]>()?;

            // 验证标签名称匹配
            if name != closing_name {
                return Err(syn::Error::new_spanned(
                    &closing_name,
                    format!("closing tag '{closing_name}' does not match opening tag '{name}'"),
                ));
            }

            children
        };

        Ok(RsxElement {
            name,
            attributes,
            children,
            self_closing,
        })
    }
}

/// 解析子节点列表
///
/// `parent_name` 为 None 时表示 Fragment 上下文，
/// 为 Some 时表示某个命名元素的子节点。
fn parse_children(input: ParseStream, parent_name: Option<&Ident>) -> Result<Vec<RsxNode>> {
    let mut children = Vec::new();

    loop {
        // 检查是否到达闭合标签
        if input.peek(Token![<]) && input.peek2(Token![/]) {
            break;
        }

        // 检查是否已经没有内容了
        if input.is_empty() {
            let label = match parent_name {
                Some(name) => format!("unclosed tag '{name}'"),
                None => "unclosed fragment '<>'".to_string(),
            };
            return Err(syn::Error::new(input.span(), label));
        }

        if input.peek(token::Brace) {
            // 表达式 {expr} 或展开 {...expr} 或 for 循环 {for ...}
            let content;
            syn::braced!(content in input);

            if content.peek(Token![..]) {
                // Rust tokenizer 将 `...` 分割为 `..` (Range) 和 `.` (Dot)，
                // 因此需要分两步解析，这是 proc-macro 中处理 `...` 的标准方式。
                content.parse::<Token![..]>()?;
                content.parse::<Token![.]>()?;
                let expr: Expr = content.parse()?;
                children.push(RsxNode::Spread(expr));
            } else if content.peek(Token![for]) {
                // for 循环语法糖: {for item in iter { <child /> }}
                children.push(parse_for_loop(&content)?);
            } else {
                let expr: Expr = content.parse()?;
                children.push(RsxNode::Expr(expr));
            }
        } else if input.peek(Token![<]) {
            // 子元素 <child>
            children.push(RsxNode::Element(input.parse()?));
        } else if input.peek(syn::LitStr) {
            // 裸字符串字面量: "Hello"
            let lit: syn::LitStr = input.parse()?;
            children.push(RsxNode::Expr(Expr::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Str(lit),
            })));
        } else {
            let context = match parent_name {
                Some(name) => format!("unexpected token in <{name}>: expected '{{expr}}', '\"text\"', '<child>' or '</{name}>'"),
                None => "unexpected token in fragment: expected '{expr}', '\"text\"', '<child>' or '</>'".to_string(),
            };
            return Err(syn::Error::new(input.span(), context));
        }
    }

    Ok(children)
}

/// 解析 for 循环: `for item in iter { <child /> ... }`
fn parse_for_loop(content: ParseStream) -> Result<RsxNode> {
    content.parse::<Token![for]>()?;

    // 解析绑定模式（支持简单 ident 和元组解构等）
    let binding: Pat = Pat::parse_single(content)?;

    content.parse::<Token![in]>()?;

    // 解析迭代器表达式（到 `{` 之前的所有内容）
    // 由于迭代器后面跟着 `{ body }`，我们需要小心解析。
    // 使用 Expr::parse 会贪婪消费大括号，所以我们按 token 手动收集。
    let mut iter_tokens = proc_macro2::TokenStream::new();
    while !content.peek(token::Brace) {
        if content.is_empty() {
            return Err(syn::Error::new(
                content.span(),
                "expected '{' after for-in expression",
            ));
        }
        let tt: proc_macro2::TokenTree = content.parse()?;
        iter_tokens.extend(std::iter::once(tt));
    }
    let iter_expr: Expr = syn::parse2(iter_tokens)?;

    // 解析 body: { children... }
    let body_content;
    syn::braced!(body_content in content);

    let mut body = Vec::new();
    while !body_content.is_empty() {
        if body_content.peek(Token![<]) {
            body.push(RsxNode::Element(body_content.parse()?));
        } else if body_content.peek(token::Brace) {
            let inner;
            syn::braced!(inner in body_content);
            if inner.peek(Token![..]) {
                inner.parse::<Token![..]>()?;
                inner.parse::<Token![.]>()?;
                let expr: Expr = inner.parse()?;
                body.push(RsxNode::Spread(expr));
            } else if inner.peek(Token![for]) {
                body.push(parse_for_loop(&inner)?);
            } else {
                let expr: Expr = inner.parse()?;
                body.push(RsxNode::Expr(expr));
            }
        } else if body_content.peek(syn::LitStr) {
            let lit: syn::LitStr = body_content.parse()?;
            body.push(RsxNode::Expr(Expr::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Str(lit),
            })));
        } else {
            return Err(syn::Error::new(
                body_content.span(),
                "unexpected token in for-loop body",
            ));
        }
    }

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
            let first = iter.next().unwrap();
            let second = iter.next().unwrap();
            Ok((first, second))
        } else {
            Err(syn::Error::new_spanned(
                tuple,
                format!("{attr_name} attribute expects a tuple of (condition, closure)"),
            ))
        }
    } else {
        Err(syn::Error::new_spanned(
            value,
            format!("{attr_name} attribute expects a tuple of (condition, closure)"),
        ))
    }
}
