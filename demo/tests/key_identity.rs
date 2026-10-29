use gpui::{Element, ElementId, div, prelude::*};

#[test]
fn concrete_integer_keys_use_named_integer_ids() {
    let key = 42u64;
    let element = gpui_rsx::rsx! { <div key={key} focusable /> };
    assert!(matches!(
        Element::id(&element),
        Some(ElementId::NamedInteger(_, 42))
    ));
    let key = String::from("row");
    let element = gpui_rsx::rsx! { <div key={key} focusable /> };
    assert!(matches!(Element::id(&element), Some(ElementId::Name(_))));
    assert_eq!(key, "row");
}
