//! Static layout approximation. Never evaluates Rust expressions or event handlers.
use crate::literal_string;
use gpui_rsx_core::parser::{RsxAttribute, RsxBody, RsxElement, RsxNode};
use gpui_rsx_core::{ClassMode, ClassOutcome, analyze_class};

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
    fn nodes(children: &[RsxNode], mode: ClassMode, out: &mut String) {
        for child in children {
            match child {
                RsxNode::Element(el) => element(el, mode, out),
                RsxNode::Expr(expr) => match literal_string(expr) {
                    Some(text) => out.push_str(&escape(&text)),
                    None => out.push_str("<span class=\"rsx-placeholder\">{expression}</span>"),
                },
                RsxNode::Spread(_) => {
                    out.push_str("<span class=\"rsx-placeholder\">{children}</span>")
                }
                RsxNode::For { body, .. } => {
                    out.push_str("<div class=\"rsx-placeholder\">loop template</div>");
                    nodes(body, mode, out);
                }
            }
        }
    }
    fn element(el: &RsxElement, mode: ClassMode, out: &mut String) {
        let mut styles = Vec::new();
        let mut omitted = Vec::new();
        for attr in &el.attributes {
            if let RsxAttribute::Value { name, value } = attr
                && name == "class"
            {
                if let Some(classes) = literal_string(value) {
                    for class in classes.split_ascii_whitespace() {
                        match css(class, mode) {
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
        nodes(&el.children, mode, out);
        out.push_str("</div>");
    }
    let mut out = String::new();
    match &body {
        RsxBody::Single(el) => element(el, mode, &mut out),
        RsxBody::Fragment(children) => nodes(children, mode, &mut out),
    }
    Ok(out)
}

fn css(class: &str, mode: ClassMode) -> Option<String> {
    if !matches!(
        analyze_class(class, mode).outcome,
        ClassOutcome::Supported(_)
    ) || class.contains(':')
    {
        return None;
    }
    let direct = match class {
        "flex" => "display:flex",
        "flex-col" => "flex-direction:column",
        "flex-row" => "flex-direction:row",
        "flex-wrap" => "flex-wrap:wrap",
        "flex-1" => "flex:1",
        "items-center" => "align-items:center",
        "items-start" => "align-items:flex-start",
        "items-end" => "align-items:flex-end",
        "justify-center" => "justify-content:center",
        "justify-between" => "justify-content:space-between",
        "justify-end" => "justify-content:flex-end",
        "hidden" => "display:none",
        "block" => "display:block",
        "relative" => "position:relative",
        "absolute" => "position:absolute",
        "w-full" => "width:100%",
        "h-full" => "height:100%",
        "size-full" => "width:100%;height:100%",
        "overflow-hidden" => "overflow:hidden",
        "overflow-scroll" => "overflow:auto",
        "border" => "border:1px solid",
        "font-bold" => "font-weight:700",
        "font-semibold" => "font-weight:600",
        "font-medium" => "font-weight:500",
        "text-center" => "text-align:center",
        "text-left" => "text-align:left",
        "text-right" => "text-align:right",
        "text-xs" => "font-size:12px",
        "text-sm" => "font-size:14px",
        "text-base" => "font-size:16px",
        "text-lg" => "font-size:18px",
        "text-xl" => "font-size:20px",
        "text-2xl" => "font-size:24px",
        "rounded" => "border-radius:4px",
        "rounded-sm" => "border-radius:2px",
        "rounded-md" => "border-radius:6px",
        "rounded-lg" => "border-radius:8px",
        "rounded-full" => "border-radius:9999px",
        "underline" => "text-decoration:underline",
        "italic" => "font-style:italic",
        _ => "",
    };
    if !direct.is_empty() {
        return Some(direct.into());
    }
    if let Some((method, value, _)) = gpui_rsx_core::codegen::tables::split_length_class(class) {
        let value = if value.starts_with('[') {
            value.trim_matches(['[', ']']).to_owned()
        } else if let Some((n, d)) = value.split_once('/') {
            format!(
                "{}%",
                n.parse::<f32>().ok()? / d.parse::<f32>().ok()? * 100.0
            )
        } else {
            format!("{}px", value.parse::<f32>().ok()? * mode.scale.factor())
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
            _ => return None,
        };
        return Some(
            props
                .iter()
                .map(|p| format!("{p}:{value}"))
                .collect::<Vec<_>>()
                .join(";"),
        );
    }
    for (prefix, prop) in [
        ("bg-", "background-color"),
        ("text-", "color"),
        ("border-", "border-color"),
    ] {
        if let Some(value) = class.strip_prefix(prefix)
            && let Some(color) = gpui_rsx_core::codegen::class::color_rgba(value)
        {
            return Some(format!("{prop}:#{color:08x}"));
        }
    }
    class
        .strip_prefix("opacity-")
        .and_then(|n| n.parse::<u8>().ok())
        .map(|n| format!("opacity:{}", f32::from(n) / 100.0))
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
