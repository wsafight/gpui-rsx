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

#[test]
fn explicit_id_wins_and_does_not_evaluate_an_unused_key() {
    let key_calls = std::cell::Cell::new(0);
    let element = gpui_rsx::rsx! {
        <button id="explicit" key={{ key_calls.set(key_calls.get() + 1); 3u64 }} onClick={|_, _, _| {}} />
    };
    assert_eq!(Element::id(&element), Some("explicit".into()));
    assert_eq!(key_calls.get(), 0);
}

#[test]
fn keys_on_plain_elements_are_ignored_without_evaluation() {
    let key_calls = std::cell::Cell::new(0);
    let element: gpui::Div = gpui_rsx::rsx! {
        <div key={{ key_calls.set(key_calls.get() + 1); 3u64 }} />
    };
    assert!(Element::id(&element).is_none());
    assert_eq!(key_calls.get(), 0);
}

#[test]
fn display_keys_are_borrowed_and_integer_references_remain_usable() {
    let key = String::from("row");
    let element = gpui_rsx::rsx! { <div key={&key} focusable /> };
    assert!(matches!(Element::id(&element), Some(ElementId::Name(_))));
    assert_eq!(key, "row");
    let key = 17u32;
    let element = gpui_rsx::rsx! { <div key={&key} focusable /> };
    assert!(Element::id(&element).is_some());
    assert_eq!(key, 17);
}

#[test]
fn auto_ids_are_stable_at_one_call_site_and_distinct_between_siblings() {
    fn render() -> gpui::Stateful<gpui::Div> {
        gpui_rsx::rsx! { <div onClick={|_, _, _| {}} /> }
    }
    assert_eq!(Element::id(&render()), Element::id(&render()));
    let siblings =
        gpui_rsx::rsx! { <><button onClick={|_, _, _| {}} /><button onClick={|_, _, _| {}} /></> };
    assert_ne!(Element::id(&siblings[0]), Element::id(&siblings[1]));
}
