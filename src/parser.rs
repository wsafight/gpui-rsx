//! RSX 语法解析器
//!
//! 解析类似 JSX 的语法结构

use syn::{
    parse::{Parse, ParseStream},
    token, Expr, Ident, Result, Token,
};

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
}

/// RSX 节点
///
/// 可以是元素或表达式
#[derive(Debug)]
pub enum RsxNode {
    /// 子元素
    Element(RsxElement),
    /// 表达式（文本或其他）
    Expr(Expr),
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

                attributes.push(RsxAttribute::Value {
                    name: attr_name,
                    value,
                });
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
        let mut children = Vec::new();
        if !self_closing {
            loop {
                // 检查是否到达闭合标签
                if input.peek(Token![<]) && input.peek2(Token![/]) {
                    break;
                }

                // 检查是否已经没有内容了
                if input.is_empty() {
                    return Err(syn::Error::new(
                        input.span(),
                        format!("unclosed tag '{name}'"),
                    ));
                }

                if input.peek(token::Brace) {
                    // 表达式 {expr}
                    let content;
                    syn::braced!(content in input);
                    let expr: Expr = content.parse()?;
                    children.push(RsxNode::Expr(expr));
                } else if input.peek(Token![<]) {
                    // 子元素 <child>
                    children.push(RsxNode::Element(input.parse()?));
                } else {
                    return Err(syn::Error::new(
                        input.span(),
                        format!("unexpected token in <{name}>: expected '{{expr}}', '<child>' or '</{name}>'"),
                    ));
                }
            }

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
        }

        Ok(RsxElement {
            name,
            attributes,
            children,
            self_closing,
        })
    }
}
