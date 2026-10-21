//! P1 semantics: Tailwind state variants, `rsx_tw!` scale, and dynamic unknown-class handling.

mod common;

use common::*;
use gpui_rsx::{rsx, rsx_expand, rsx_strict, rsx_tw};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn state_variants_expand_to_style_refinement_hooks() {
    let expanded = rsx_expand! {
        <div class="bg-blue-500 hover:bg-blue-600 focus:border-2 focus-visible:underline group-hover:italic" />
    };
    let compact = expanded.split_whitespace().collect::<String>();
    assert!(
        compact.contains(".hover(|__style|__style.bg(rgb(2450411u32)))"),
        "{compact}"
    );
    assert!(
        compact.contains(".focus(|__style|__style.border_2())"),
        "{compact}"
    );
    assert!(
        compact.contains(".focus_visible(|__style|__style.underline())"),
        "{compact}"
    );
    assert!(
        compact.contains(".group_hover(\"\",|__style|__style.italic())"),
        "{compact}"
    );
    assert!(
        !compact.contains(".id("),
        "hover/focus do not need an element id: {compact}"
    );

    let _el = rsx! {
        <div class="bg-blue-500 hover:bg-blue-600 hover:text-white focus:border-2 group-hover/menu:italic" />
    };
}

#[test]
fn active_variant_injects_auto_id_like_active_class() {
    take_last_auto_id();
    let _el = rsx! { <button class="active:opacity-75 group-active/row:underline" /> };
    let id = take_last_auto_id().expect("active: requires a stateful element");
    assert!(id.contains("__rsx_button_"), "{id}");
}

#[test]
fn tailwind_scale_macro_uses_4px_units() {
    take_length_calls();
    let _el = rsx_tw! { <div class="gap-4 p-2.5 w-64 h-[14px]" /> };
    assert_eq!(
        take_length_calls(),
        vec![("px", 16.0), ("px", 10.0), ("px", 256.0), ("px", 14.0)]
    );

    // Dynamic classes follow the same scale.
    let cls = String::from("gap-3 mx-0.5");
    let _el = rsx_tw! { <div class={cls} /> };
    assert_eq!(take_length_calls(), vec![("px", 12.0), ("px", 2.0)]);

    // The default macro follows the crate's feature selection.
    let _el = rsx! { <div class="gap-4" /> };
    assert_eq!(take_length_calls(), vec![("px", scaled_px(4.0))]);
}

#[test]
fn strict_dynamic_classes_can_report_instead_of_panicking() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = seen.clone();
    let cls = String::from("flex not-a-class dark:bg-black");
    let _el = rsx_strict! {
        <div class={cls} onUnknownClass={move |class: &str| sink.borrow_mut().push(class.to_owned())} />
    };
    assert_eq!(*seen.borrow(), ["not-a-class", "dark:bg-black"]);
}

#[test]
#[should_panic(expected = "unsupported dynamic class \"not-a-class\" in strict mode")]
fn strict_dynamic_classes_still_panic_by_default() {
    let cls = String::from("not-a-class");
    let _el = rsx_strict! { <div class={cls} /> };
}

#[test]
fn permissive_dynamic_unknown_classes_are_ignored() {
    let cls = String::from("flex md:flex-row");
    let _el = rsx! { <div class={cls} /> };
}

#[test]
fn generated_gpui_methods_are_reachable_from_classes() {
    // Directional rounded presets and border widths come straight from the generated tables.
    let expanded = rsx_expand! { <div class="rounded-t-md border-x-2 mx-auto" /> };
    let compact = expanded.split_whitespace().collect::<String>();
    assert_eq!(compact, "div().rounded_t_md().border_x_2().mx_auto()");
}
