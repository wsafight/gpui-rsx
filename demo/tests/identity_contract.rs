use gpui::{Element, ElementId, div, prelude::*};
use gpui_rsx::rsx;

#[test]
#[rustfmt::skip]
fn distinct_macro_calls_and_sibling_elements_get_distinct_auto_ids() {
    let first = rsx! { <div onClick={|_, _, _| {}} /> }; let second = rsx! { <div onClick={|_, _, _| {}} /> };
    assert_ne!(Element::id(&first), Element::id(&second));
    let children = rsx! { <><div onClick={|_, _, _| {}} /><div onClick={|_, _, _| {}} /></> };
    assert_ne!(Element::id(&children[0]), Element::id(&children[1]));
}

#[test]
fn explicit_identity_survives_source_relocation_and_sibling_insertion() {
    fn original() -> gpui::Stateful<gpui::Div> {
        rsx! { <div id="stable-control" onClick={|_, _, _| {}} /> }
    }
    fn relocated() -> gpui::Stateful<gpui::Div> {
        let _inserted = rsx! { <div /> };
        rsx! { <div id="stable-control" onClick={|_, _, _| {}} /> }
    }
    assert_eq!(Element::id(&original()), Element::id(&relocated()));
    assert_eq!(
        Element::id(&original()),
        Some(ElementId::from("stable-control"))
    );
}

#[test]
fn explicit_instance_and_row_identity_survives_reordering() {
    fn identities(instance: &'static str, rows: &[u64]) -> Vec<ElementId> {
        rows.iter()
            .map(|row| {
                let element = rsx! { <div id={(instance, *row)} onClick={|_, _, _| {}} /> };
                Element::id(&element).unwrap()
            })
            .collect()
    }
    let before = identities("first", &[1, 2]);
    let after = identities("first", &[3, 2, 1]);
    let other_instance = identities("second", &[1, 2]);
    assert_eq!(before[0], after[2]);
    assert_eq!(before[1], after[1]);
    assert_ne!(before[0], other_instance[0]);
    assert_ne!(after[0], after[1]);
}
