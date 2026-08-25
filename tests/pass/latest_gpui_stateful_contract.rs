use gpui_rsx::rsx;

struct MockElement;
struct MockStatefulElement;
struct MockElementId;

impl From<&'static str> for MockElementId {
    fn from(_: &'static str) -> Self {
        Self
    }
}

fn div() -> MockElement {
    MockElement
}

impl MockElement {
    fn id(self, _: impl Into<MockElementId>) -> MockStatefulElement {
        MockStatefulElement
    }
}

impl MockStatefulElement {
    fn accessibility_id(self, _: &str) -> Self { self }
    fn aria_description(self, _: &str) -> Self { self }
    fn aria_keyshortcuts(self, _: &str) -> Self { self }
    fn aria_active_descendant(self) -> Self { self }
    fn a11y_synthetic_children(self, _: impl FnOnce() -> ()) -> Self { self }
    fn aria_numeric_value_step(self, _: f64) -> Self { self }
    fn aria_value(self, _: &str) -> Self { self }
    fn aria_placeholder(self, _: &str) -> Self { self }
    fn restrict_scroll_to_axis(self) -> Self { self }
    fn external_drag_payload(self, _: impl FnOnce(()) -> ()) -> Self { self }
}

fn main() {
    let _ = rsx! { <div accessibilityId={"platform-id"} /> };
    let _ = rsx! { <div ariaDescription={"description"} /> };
    let _ = rsx! { <div ariaKeyShortcuts={"Ctrl+K"} /> };
    let _ = rsx! { <div ariaActiveDescendant /> };
    let _ = rsx! { <div a11ySyntheticChildren={|| ()} /> };
    let _ = rsx! { <div ariaNumericValueStep={1.0f64} /> };
    let _ = rsx! { <div ariaValue={"value"} /> };
    let _ = rsx! { <div ariaPlaceholder={"placeholder"} /> };
    let _ = rsx! { <div restrictScrollToAxis /> };
    let _ = rsx! { <div externalDragPayload={|_| ()} /> };

    let _ = rsx! { <div accessibility_id={"platform-id"} /> };
    let _ = rsx! { <div aria_description={"description"} /> };
    let _ = rsx! { <div aria_keyshortcuts={"Ctrl+K"} /> };
    let _ = rsx! { <div aria_active_descendant /> };
    let _ = rsx! { <div a11y_synthetic_children={|| ()} /> };
    let _ = rsx! { <div aria_numeric_value_step={1.0f64} /> };
    let _ = rsx! { <div aria_value={"value"} /> };
    let _ = rsx! { <div aria_placeholder={"placeholder"} /> };
    let _ = rsx! { <div restrict_scroll_to_axis /> };
    let _ = rsx! { <div external_drag_payload={|_| ()} /> };
}
