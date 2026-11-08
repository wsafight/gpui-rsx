mod common;

use common::*;
use gpui_rsx::rsx;
use std::cell::RefCell;

#[test]
fn static_and_dynamic_opacity_accept_the_same_decimal_percentages() {
    take_opacity_calls();

    let _zero = rsx! { <div class="opacity-0" /> };
    let _decimal = rsx! { <div class="opacity-33.5" /> };
    let _hundred = rsx! { <div class="opacity-100" /> };
    let static_calls = take_opacity_calls();

    for class in ["opacity-0", "opacity-33.5", "opacity-100"] {
        let _element = rsx! { <div class={class} /> };
    }
    let dynamic_calls = take_opacity_calls();

    assert_eq!(static_calls, vec![0.0, 0.335, 1.0]);
    assert_eq!(dynamic_calls, static_calls);
}

#[test]
fn dynamic_opacity_rejects_out_of_range_and_non_finite_percentages() {
    let unknown = RefCell::new(Vec::new());

    for class in [
        "opacity--0.1",
        "opacity-100.1",
        "opacity-NaN",
        "opacity-inf",
        "opacity--inf",
    ] {
        let _element = rsx! {
            <div
                class={class}
                onUnknownClass={|class: &str| unknown.borrow_mut().push(class.to_owned())}
            />
        };
    }

    assert_eq!(
        unknown.into_inner(),
        vec![
            "opacity--0.1".to_owned(),
            "opacity-100.1".to_owned(),
            "opacity-NaN".to_owned(),
            "opacity-inf".to_owned(),
            "opacity--inf".to_owned(),
        ]
    );
}
