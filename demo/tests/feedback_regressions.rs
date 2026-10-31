#![allow(unknown_lints)]
#![deny(float_literal_f32_fallback)]

use gpui::{AnyElement, Element, ElementId, SharedString, div, prelude::*};
use gpui_rsx::{component, rsx, rsx_fragment};

#[component]
fn Counter(
    #[prop(default = 7)] value: u32,
    #[prop(default = 7)] size: usize,
    #[prop(default = 7.0)] ratio: f32,
    #[prop(default = "ready")] label: SharedString,
) -> impl IntoElement {
    div().child(format!("{value}:{size}:{ratio}:{label}"))
}

#[component]
#[derive(gpui::IntoElement)]
struct Badge {
    #[prop(default = 0xff0000)]
    color: u32,
}

impl RenderOnce for Badge {
    fn render(self, _: &mut gpui::Window, _: &mut gpui::App) -> impl IntoElement {
        div().bg(gpui::rgb(self.color))
    }
}

#[test]
fn numeric_defaults_keep_target_types_in_function_and_struct_components() {
    let counter = rsx! { <Counter /> }.build();
    assert_eq!(counter.value, 7);
    assert_eq!(counter.size, 7);
    assert_eq!(counter.ratio, 7.0);
    assert_eq!(counter.label.as_ref(), "ready");
    let _: AnyElement = counter.into_any_element();
    let badge = rsx! { <Badge /> }.build();
    assert_eq!(badge.color, 0xff0000);
    let _: AnyElement = badge.into_any_element();
}

#[test]
fn dynamic_keys_accept_owned_borrowed_shared_and_large_display_values() {
    fn borrowed(key: &str) -> gpui::Stateful<gpui::Div> {
        rsx! { <div key={key} onClick={|_, _, _| {}} /> }
    }

    let owned = String::from("row");
    let shared = SharedString::from("shared");
    let elements = [
        rsx! { <div key={owned} onClick={|_, _, _| {}} /> },
        borrowed(&owned),
        rsx! { <div key={shared} onClick={|_, _, _| {}} /> },
        rsx! { <div key={u128::MAX} onClick={|_, _, _| {}} /> },
    ];
    for element in elements {
        assert!(matches!(Element::id(&element), Some(ElementId::Name(_))));
    }
    assert_eq!(owned, "row");
    assert_eq!(shared.as_ref(), "shared");
    let number = 7usize;
    let integer = rsx! { <div key={number} onClick={|_, _, _| {}} /> };
    assert!(matches!(
        Element::id(&integer),
        Some(ElementId::NamedInteger(_, 7))
    ));
}

#[test]
fn erased_fragments_accept_mixed_roots_and_preserve_ordinary_fragment_types() {
    let parts: Vec<AnyElement> = rsx_fragment! {
        <>
            <div />
            <div id="stateful" />
            <Counter />
            <Badge />
            {"label"}
        </>
    };
    assert_eq!(parts.len(), 5);
    let _: AnyElement = rsx! { <div>{...parts}</div> }.into_any_element();
    let concrete: Vec<gpui::Div> = rsx! { <><div /><div /></> };
    assert_eq!(concrete.len(), 2);
}
