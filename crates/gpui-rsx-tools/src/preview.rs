//! Static layout approximation. Never evaluates Rust expressions or event handlers.
use crate::literal_string;
use gpui_rsx_core::syntax::{RsxAttribute, RsxBody, RsxElement, RsxNode};
use gpui_rsx_core::{ClassMode, StyleLength, StyleValue, resolve_style};

pub fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn render(source: &str, mode: ClassMode) -> Result<String, String> {
    let body = syn::parse_str::<RsxBody>(source).map_err(|e| e.to_string())?;
    render_body(&body, mode)
}

/// Renders an already parsed body, for hosts that also lint or expand it.
pub fn render_body(body: &RsxBody, mode: ClassMode) -> Result<String, String> {
    fn nodes(children: &[RsxNode], mode: ClassMode, out: &mut String) {
        for child in children {
            match child {
                RsxNode::Element(el) => element(el, mode.clone(), out),
                RsxNode::Fragment(children) => nodes(children, mode.clone(), out),
                RsxNode::Expr(expr) => match literal_string(expr) {
                    Some(text) => out.push_str(&escape(&text)),
                    None => out.push_str("<span class=\"rsx-placeholder\">{expression}</span>"),
                },
                RsxNode::Spread(_) => {
                    out.push_str("<span class=\"rsx-placeholder\">{children}</span>")
                }
                RsxNode::For { body, .. } => {
                    out.push_str("<div class=\"rsx-placeholder\">loop template</div>");
                    nodes(body, mode.clone(), out);
                }
                _ => out.push_str("<span class=\"rsx-placeholder\">unknown syntax</span>"),
            }
        }
    }
    fn element(el: &RsxElement, mode: ClassMode, out: &mut String) {
        let mut styles = Vec::new();
        let mut omitted = Vec::new();
        if el
            .attributes
            .iter()
            .any(|attr| matches!(attr, RsxAttribute::Flag(name) if name == "styled"))
            && let Some(classes) =
                gpui_rsx_core::codegen::tables::lookup_tag_default(&el.name.to_string())
        {
            for class in classes.split_ascii_whitespace() {
                if let Some(style) = css(class, mode.clone()) {
                    styles.push(style);
                } else {
                    omitted.push(class.to_owned());
                }
            }
        }
        for attr in &el.attributes {
            if matches!(attr, RsxAttribute::Flag(name) if name == "styled") {
                continue;
            }
            if let RsxAttribute::Value { name, value } = attr
                && name == "class"
            {
                if let Some(classes) = literal_string(value) {
                    for class in classes.split_ascii_whitespace() {
                        match css(class, mode.clone()) {
                            Some(style) => styles.push(style),
                            None => omitted.push(class.to_owned()),
                        }
                    }
                } else {
                    omitted.push("dynamic class".into());
                }
            } else {
                omitted.push("attribute/behavior".into());
            }
        }
        let name = el.name.to_string();
        out.push_str(&format!(
            "<div data-rsx-tag=\"{}\" style=\"{}\">",
            escape(&name),
            escape(&styles.join(";"))
        ));
        if name.chars().next().is_some_and(char::is_uppercase) {
            out.push_str(&format!(
                "<span class=\"rsx-placeholder\">component: {}</span>",
                escape(&name)
            ));
        }
        if !omitted.is_empty() {
            out.push_str(&format!(
                "<small class=\"rsx-placeholder\" title=\"{}\">partial preview</small>",
                escape(&omitted.join(", "))
            ));
        }
        nodes(&el.children, mode.clone(), out);
        out.push_str("</div>");
    }
    let mut out = String::new();
    match body {
        RsxBody::Single(el) => element(el, mode.clone(), &mut out),
        RsxBody::Fragment(children) => nodes(children, mode, &mut out),
        _ => return Err("preview does not support this RSX body".into()),
    }
    Ok(out)
}

fn css(class: &str, mode: ClassMode) -> Option<String> {
    let operation = resolve_style(class, mode)?.ok()?;
    let method = operation.method();
    let Some(value) = operation.value() else {
        return preset_css(method).map(str::to_owned);
    };
    let value = match value {
        StyleValue::Color { hex, alpha } => {
            let prop = match method {
                "bg" => "background-color",
                "text_color" => "color",
                "border_color" => "border-color",
                _ => return None,
            };
            let rgba = if *alpha { *hex } else { (*hex << 8) | 0xff };
            return Some(format!("{prop}:#{rgba:08x}"));
        }
        StyleValue::Length(length) => match length {
            StyleLength::Px(value) => format!("{value}px"),
            StyleLength::Rem(value) => format!("{value}rem"),
            StyleLength::Relative(value) => format!("{}%", value * 100.0),
            _ => return None,
        },
        StyleValue::Number(number) if number.is_finite() && method == "opacity" => {
            return Some(format!("opacity:{number}"));
        }
        StyleValue::Expression(expr) => {
            if method == "font_weight" {
                let weight = match expr.as_str() {
                    "FontWeight::THIN" => 100,
                    "FontWeight::EXTRA_LIGHT" => 200,
                    "FontWeight::LIGHT" => 300,
                    "FontWeight::NORMAL" => 400,
                    "FontWeight::MEDIUM" => 500,
                    "FontWeight::SEMIBOLD" => 600,
                    "FontWeight::BOLD" => 700,
                    "FontWeight::EXTRA_BOLD" => 800,
                    "FontWeight::BLACK" => 900,
                    _ => return None,
                };
                return Some(format!("font-weight:{weight}"));
            }
            // Recognize only normalized literal lengths; never execute expressions.
            let expr = expr.trim();
            let (number, unit) =
                if let Some(inner) = expr.strip_prefix("px(").and_then(|v| v.strip_suffix(')')) {
                    (inner, "px")
                } else {
                    let inner = expr
                        .strip_prefix("rems(")
                        .and_then(|v| v.strip_suffix(')'))?;
                    (inner, "rem")
                };
            let number = number
                .trim()
                .parse::<f32>()
                .ok()
                .filter(|n| n.is_finite())?;
            format!("{number}{unit}")
        }
        _ => return None,
    };
    let props: &[&str] = match method {
        "w" => &["width"],
        "h" => &["height"],
        "size" => &["width", "height"],
        "p" => &["padding"],
        "px" => &["padding-left", "padding-right"],
        "py" => &["padding-top", "padding-bottom"],
        "pt" => &["padding-top"],
        "pb" => &["padding-bottom"],
        "pl" => &["padding-left"],
        "pr" => &["padding-right"],
        "m" => &["margin"],
        "mx" => &["margin-left", "margin-right"],
        "my" => &["margin-top", "margin-bottom"],
        "mt" => &["margin-top"],
        "mb" => &["margin-bottom"],
        "ml" => &["margin-left"],
        "mr" => &["margin-right"],
        "gap" => &["gap"],
        "gap_x" => &["column-gap"],
        "gap_y" => &["row-gap"],
        "min_w" => &["min-width"],
        "max_w" => &["max-width"],
        "min_h" => &["min-height"],
        "max_h" => &["max-height"],
        "rounded" => &["border-radius"],
        "text_size" => &["font-size"],
        "line_height" => &["line-height"],
        _ => return None,
    };
    Some(
        props
            .iter()
            .map(|prop| format!("{prop}:{value}"))
            .collect::<Vec<_>>()
            .join(";"),
    )
}

fn preset_css(method: &str) -> Option<&'static str> {
    Some(match method {
        "flex" => "display:flex",
        "flex_col" => "flex-direction:column",
        "flex_row" => "flex-direction:row",
        "flex_wrap" => "flex-wrap:wrap",
        "flex_1" => "flex:1",
        "flex_grow_1" => "flex-grow:1",
        "flex_shrink_1" => "flex-shrink:1",
        "items_center" => "align-items:center",
        "items_start" => "align-items:flex-start",
        "items_end" => "align-items:flex-end",
        "justify_center" => "justify-content:center",
        "justify_between" => "justify-content:space-between",
        "justify_end" => "justify-content:flex-end",
        "hidden" => "display:none",
        "block" => "display:block",
        "relative" => "position:relative",
        "absolute" => "position:absolute",
        "w_full" => "width:100%",
        "h_full" => "height:100%",
        "size_full" => "width:100%;height:100%",
        "overflow_hidden" => "overflow:hidden",
        "overflow_scroll" => "overflow:auto",
        "border_1" => "border:1px solid",
        "text_center" => "text-align:center",
        "text_left" => "text-align:left",
        "text_right" => "text-align:right",
        "text_xs" => "font-size:12px",
        "text_sm" => "font-size:14px",
        "text_base" => "font-size:16px",
        "text_lg" => "font-size:18px",
        "text_xl" => "font-size:20px",
        "text_2xl" => "font-size:24px",
        "text_3xl" => "font-size:30px",
        "rounded" => "border-radius:4px",
        "rounded_sm" => "border-radius:2px",
        "rounded_md" => "border-radius:6px",
        "rounded_lg" => "border-radius:8px",
        "rounded_full" => "border-radius:9999px",
        "underline" => "text-decoration:underline",
        "text_decoration_none" => "text-decoration:none",
        "italic" => "font-style:italic",
        _ => return None,
    })
}

pub fn page(content: &str, refresh: bool) -> String {
    format!(
        r#"<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>RSX static preview</title><style>body{{font:14px system-ui;margin:24px;background:#f4f4f5;color:#18181b}}.rsx-placeholder{{font:11px monospace;background:#fef3c7;color:#92400e;padding:2px 4px;display:inline-block}}#preview{{background:white;padding:16px;border:1px solid #ddd;min-height:200px}}</style><h1>RSX static preview</h1><p>Approximate layout. Rust expressions, events, custom components, theme expressions and unsupported styles are placeholders.</p><main id="preview">{content}</main>{}</html>"#,
        if refresh {
            r#"<script>let previous='';setInterval(async()=>{try{const html=await(await fetch('/content',{cache:'no-store'})).text();if(html!==previous){document.querySelector('#preview').innerHTML=html;previous=html}}catch{}},500)</script>"#
        } else {
            ""
        }
    )
}
