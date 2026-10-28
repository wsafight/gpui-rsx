//! Stable-toolchain fuzz smoke test: random token soup and random class strings must never
//! panic the parser or code generator — every failure has to be a `compile_error!`.
//!
//! The coverage-guided version lives in `fuzz/` (`cargo +nightly fuzz run expand`).

use gpui_rsx_core::{ClassMode, SpacingScale, analyze_class, class_diagnostics, expand};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// xorshift64*: deterministic, dependency-free.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[(self.next() % items.len() as u64) as usize]
    }
}

const TOKENS: &[&str] = &[
    "<",
    ">",
    "/",
    "</",
    "/>",
    "<>",
    "</>",
    "div",
    "span",
    "Button",
    "ui::Card",
    "img",
    "svg",
    "canvas",
    "class",
    "id",
    "key",
    "base",
    "when",
    "whenSome",
    "whenClass",
    "hoverClass",
    "onClick",
    "onUnknownClass",
    "styled",
    "flex",
    "=",
    "{",
    "}",
    "(",
    ")",
    ",",
    "|el|",
    "el",
    "for",
    "in",
    "item",
    "items",
    "...",
    "\"flex gap-4\"",
    "\"hover:bg-red-500 md:flex\"",
    "\"w-[\"",
    "\"\"",
    "42",
    "-1",
    "true",
    "x.y",
    "&",
    "'a'",
];

const CLASS_PARTS: &[&str] = &[
    "flex",
    "-",
    ":",
    "/",
    "[",
    "]",
    "#",
    "px",
    "rem",
    "%",
    "hover",
    "group",
    "dark",
    "md",
    "bg",
    "text",
    "border",
    "w",
    "h",
    "gap",
    "p",
    "m",
    "x",
    "t",
    "rounded",
    "font",
    "opacity",
    "line-clamp",
    "red",
    "500",
    "4",
    "0.5",
    "NaN",
    "inf",
    "1/2",
    "0/0",
    "rgb(",
    "rgba(",
    ")",
    ",",
    "999",
    "2xl",
    "fn",
    "self",
    "_",
    "é",
    "🙂",
];

fn assert_no_panic(label: &str, input: &str, f: impl FnOnce()) {
    if catch_unwind(AssertUnwindSafe(f)).is_err() {
        panic!("{label} panicked on input: {input:?}");
    }
}

#[test]
fn random_token_soup_never_panics() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    for _ in 0..4000 {
        let len = 1 + rng.next() % 24;
        let source: String = (0..len)
            .map(|_| rng.pick(TOKENS))
            .collect::<Vec<_>>()
            .join(" ");
        let Ok(tokens) = source.parse::<proc_macro2::TokenStream>() else {
            continue;
        };
        for mode in [ClassMode::PERMISSIVE, ClassMode::STRICT] {
            let tokens = tokens.clone();
            assert_no_panic("expand", &source, || {
                let _ = expand(tokens, mode);
            });
        }
    }
}

#[test]
fn random_classes_never_panic() {
    let mut rng = Rng(0xdead_beef_cafe_f00d);
    let modes = [
        ClassMode::PERMISSIVE,
        ClassMode::STRICT,
        ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind),
    ];
    for _ in 0..3000 {
        let len = 1 + rng.next() % 6;
        let class: String = (0..len).map(|_| rng.pick(CLASS_PARTS)).collect();
        for mode in &modes {
            assert_no_panic("analyze_class", &class, || {
                let _ = analyze_class(&class, mode.clone());
                let _ = class_diagnostics(&class, mode.clone());
            });
            let source = format!("<div class={class:?} hoverClass=\"flex\" />");
            if let Ok(tokens) = source.parse::<proc_macro2::TokenStream>() {
                assert_no_panic("expand", &source, || {
                    let _ = expand(tokens, mode.clone());
                });
            }
        }
    }
}

/// Inputs that previously panicked inside `Ident::new` or `Literal::f32_suffixed`.
#[test]
fn known_panic_inputs_expand_without_panicking() {
    let tailwind = ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind);
    let cases = [
        ("<div r#myAttr={1} />", ClassMode::PERMISSIVE),
        ("<div r#onFoo />", ClassMode::PERMISSIVE),
        ("<div class=\"w-1e38\" />", tailwind),
        ("<div class=\"w-3e38/0.001\" />", ClassMode::PERMISSIVE),
    ];
    for (source, mode) in cases {
        let tokens: proc_macro2::TokenStream = source.parse().unwrap();
        assert_no_panic("expand", source, || {
            let _ = expand(tokens.clone(), mode.clone());
        });
        let class = source.split('"').nth(1).unwrap_or("");
        assert_no_panic("analyze_class", class, || {
            let _ = analyze_class(class, mode.clone());
        });
    }
}
