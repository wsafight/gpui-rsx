use gpui::prelude::*;
use gpui::*;
use gpui_rsx::rsx;

struct DragPreview;

impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child("drag preview")
    }
}

#[allow(dead_code)]
fn stateful_attribute_contracts() {
    let _ = rsx! { <div accessibilityId={"platform-id"} /> };
    let _ = rsx! { <div ariaDescription={"description"} /> };
    let _ = rsx! { <div ariaKeyShortcuts={"Ctrl+K"} /> };
    let _ = rsx! { <div role={Role::ListItem} ariaActiveDescendant /> };
    let _ = rsx! {
        <div
            role={Role::List}
            a11ySyntheticChildren={|_: &mut A11ySubtreeBuilder| {}}
        />
    };
    let _ = rsx! { <div ariaNumericValueStep={1.0f64} /> };
    let _ = rsx! { <div ariaValue={"value"} /> };
    let _ = rsx! { <div ariaPlaceholder={"placeholder"} /> };
    let _ = rsx! { <div restrictScrollToAxis /> };

    let _ = rsx! { <div accessibility_id={"platform-id"} /> };
    let _ = rsx! { <div aria_description={"description"} /> };
    let _ = rsx! { <div aria_keyshortcuts={"Ctrl+K"} /> };
    let _ = rsx! { <div role={Role::ListItem} aria_active_descendant /> };
    let _ = rsx! {
        <div
            role={Role::List}
            a11y_synthetic_children={|_: &mut A11ySubtreeBuilder| {}}
        />
    };
    let _ = rsx! { <div aria_numeric_value_step={1.0f64} /> };
    let _ = rsx! { <div aria_value={"value"} /> };
    let _ = rsx! { <div aria_placeholder={"placeholder"} /> };
    let _ = rsx! { <div restrict_scroll_to_axis /> };

    let _ = rsx! {
        <div id="explicit-contract-id" ariaDescription={"explicit ID"} />
    };
}

#[allow(dead_code)]
fn external_drag_contract() {
    let drag_value = String::from("payload");
    let _ = rsx! {
        <div
            onDrag={(drag_value, |_, _, _, cx| cx.new(|_| DragPreview))}
            externalDragPayload={|_: &String, _, _| None}
        />
    };

    let drag_value = String::from("payload");
    let _ = rsx! {
        <div
            on_drag={(drag_value, |_, _, _, cx| cx.new(|_| DragPreview))}
            external_drag_payload={|_: &String, _, _| None}
        />
    };
}

#[allow(dead_code)]
fn interactive_event_contracts() {
    let _ = rsx! { <div onMouseExit={|_, _, _| {}} /> };
    let _ = rsx! { <div onMousePressure={|_, _, _| {}} /> };
    let _ = rsx! { <div captureMousePressure={|_, _, _| {}} /> };
    let _ = rsx! { <div onPinch={|_, _, _| {}} /> };
    let _ = rsx! { <div capturePinch={|_, _, _| {}} /> };

    let _ = rsx! { <div on_mouse_exit={|_, _, _| {}} /> };
    let _ = rsx! { <div on_mouse_pressure={|_, _, _| {}} /> };
    let _ = rsx! { <div capture_mouse_pressure={|_, _, _| {}} /> };
    let _ = rsx! { <div on_pinch={|_, _, _| {}} /> };
    let _ = rsx! { <div capture_pinch={|_, _, _| {}} /> };
}

#[allow(dead_code)]
fn styled_contracts() {
    let _ = rsx! { <div scrollbarWidth={px(8.0)} /> };
    let _ = rsx! { <div class="text-ellipsis-start text-ellipsis-middle" /> };
    let _ = rsx! {
        <div
            gridColsMinContent={1u16}
            gridColsMaxContent={2u16}
            gridRowsMinContent={3u16}
            gridRowsMaxContent={4u16}
        />
    };
    let _ = rsx! {
        <div boxShadow={vec![BoxShadow::new(px(0.0), px(1.0), hsla(0.0, 0.0, 0.0, 0.2))]} />
    };
}

fn main() {}
