//! Export the actual parser/codegen vocabulary for code-generation tools.
use std::path::Path;

use gpui_rsx_core::codegen::tables::{GPUI_METHODS, HTML_TAGS, LENGTH_CLASS_SPECS, T_STATEFUL};
use gpui_rsx_core::{ClassMode, ClassOutcome, SUPPORTED_VARIANTS, analyze_class, known_classes};
use serde_json::{Value, json};

const EXAMPLES: &[&str] = &[
    r#"<div class="flex flex-col gap-4 p-4 bg-blue-500"><span>{"Hello"}</span></div>"#,
    r#"<button class="hover:bg-blue-600 active:opacity-75" onClick={handler}>{"Save"}</button>"#,
    r#"<ul>{for item in items { <li key={item.id} onClick={handler}>{item.name.clone()}</li> }}</ul>"#,
    r#"<div class="w-[280px] h-1/2 gap-[14px]" />"#,
];

pub fn document() -> Value {
    let classes: Vec<_> = known_classes()
        .iter()
        .filter(|name| {
            matches!(
                analyze_class(name, ClassMode::STRICT).outcome,
                ClassOutcome::Supported(_)
            )
        })
        .collect();
    let methods: Vec<_> = GPUI_METHODS
        .iter()
        .map(|m| {
            json!({
                "name": m.name, "arity": m.arity, "requires_id": m.traits & T_STATEFUL != 0,
            })
        })
        .collect();
    let lengths: Vec<_> = LENGTH_CLASS_SPECS
        .iter()
        .map(|s| {
            json!({
                "prefix": s.prefix, "method": s.method,
                "percent": s.family.allows_percent(), "fraction": s.family.allows_fraction(),
            })
        })
        .collect();
    json!({
        "schema_version": 1,
        "gpui_revision": gpui_rsx_core::GPUI_SOURCE_REV,
        "gpui_component_revision": gpui_rsx_core::GPUI_COMPONENT_REV,
        "macros": ["rsx", "rsx_strict", "rsx_permissive", "rsx_tw", "rsx_expand"],
        "component_attribute_macro": "component",
        "tags": { "div_aliases": HTML_TAGS, "special": ["img", "svg", "canvas"], "custom": "Rust path; use base={constructor} when arguments are required" },
        "attributes": {
            "special": ["class", "id", "key", "base", "styled", "when", "whenSome", "whenClass", "hoverClass", "focusClass", "activeClass", "onUnknownClass"],
            "methods": methods,
            "naming": "GPUI snake_case or generated camelCase aliases; width/height map to w/h; multiple arguments use a tuple",
        },
        "classes": {
            "static_whitelist": classes, "static_variants": SUPPORTED_VARIANTS,
            "length_families": lengths, "default_numeric_unit_px": 1, "tailwind_numeric_unit_px": 4,
            "arbitrary_lengths": ["px", "rem", "% (sizing only)"],
            "arbitrary_colors": ["#rgb", "#rrggbb", "#rrggbbaa", "rgb(r,g,b)", "rgba(r,g,b,a)"],
            "dynamic_limitations": ["No state variants", "No rsx.toml tokens", "Only Styled methods; stateful classes need static literals"],
            "unsupported_variants": ["dark", "sm", "md", "lg", "xl", "2xl"],
        },
        "syntax": { "fragment": "<>...</>", "children_spread": "{...children}", "loop": "{for item in items { <div key={item.id} /> }}" },
        "examples": EXAMPLES,
    })
}

pub fn run(root: &Path, check: bool) -> Result<(), String> {
    let mut spec = serde_json::to_string_pretty(&document()).map_err(|e| e.to_string())?;
    spec.push('\n');
    crate::write_outputs(
        &[
            (root.join("spec/rsx-spec.json"), spec),
            (
                root.join("spec/prompt.md"),
                include_str!("../data/spec-prompt.md").into(),
            ),
        ],
        check,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_examples_expand_in_strict_mode() {
        for source in EXAMPLES {
            let expanded = gpui_rsx_core::expand_str(source, ClassMode::STRICT).unwrap();
            assert!(
                !expanded.to_string().contains("compile_error"),
                "{source}: {expanded}"
            );
        }
        assert!(
            document()["classes"]["static_whitelist"]
                .as_array()
                .unwrap()
                .len()
                > 700
        );
    }
}
