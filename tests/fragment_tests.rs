use gpui::{prelude::*, *};
use gpui_rsx::{component, rsx, rsx_fragment};

#[component]
fn Label(#[prop(default = "label")] text: gpui::SharedString) -> impl IntoElement {
    div().child(text)
}

#[test]
fn erased_fragments_mix_stateless_stateful_text_and_component_children() {
    let parts: Vec<AnyElement> = rsx_fragment! {
        <>
            <div />
            <div id="stateful" />
            {"text"}
            <Label />
        </>
    };
    assert_eq!(parts.len(), 4);
    let _ = rsx! { <div>{...parts}</div> };
}

#[test]
fn erased_fragments_keep_nested_groups_loops_and_spreads_in_source_order() {
    let mut order = Vec::new();
    let parts: Vec<AnyElement> = rsx_fragment! {
        <>
            { { order.push(1); div() } }
            <>
                {for number in 2..4 {
                    { { order.push(number); div() } }
                }}
                {...{ order.push(4); Some(div().id("optional")) }}
            </>
            { { order.push(5); div() } }
        </>
    };
    assert_eq!(parts.len(), 5);
    assert_eq!(order, [1, 2, 3, 4, 5]);
}

#[test]
fn erased_fragments_support_empty_single_and_dynamic_roots() {
    let empty: Vec<AnyElement> = rsx_fragment! { <></> };
    assert!(empty.is_empty());
    let class = String::from("flex gap-4");
    let single: Vec<AnyElement> = rsx_fragment! { <div class={class} /> };
    assert_eq!(single.len(), 1);
}

#[test]
fn ordinary_fragments_preserve_concrete_element_types() {
    let parts: Vec<Div> = rsx! { <><div /><div /></> };
    assert_eq!(parts.len(), 2);
}
