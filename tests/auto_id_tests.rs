use gpui::{Element, ElementId, div, prelude::*};
use gpui_rsx::{component, rsx};
use std::{cell::Cell, rc::Rc};

mod ui {
    pub fn card() -> gpui::Div {
        gpui::div()
    }
}

trait OnClick: Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) {}
impl<F: Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App)> OnClick for F {}
trait OnHover: Fn(&bool, &mut gpui::Window, &mut gpui::App) {}
impl<F: Fn(&bool, &mut gpui::Window, &mut gpui::App)> OnHover for F {}

#[component]
fn Card(
    #[prop(callback)] on_click: Rc<dyn OnClick>,
    #[prop(callback)] on_hover: Rc<dyn OnHover>,
) -> impl IntoElement {
    div()
        .id("card")
        .on_click(move |event, window, cx| on_click(event, window, cx))
        .on_hover(move |hovered, window, cx| on_hover(hovered, window, cx))
}

#[test]
fn native_elements_and_path_functions_receive_auto_ids() {
    let native: gpui::Stateful<gpui::Div> = rsx! { <div onClick={|_, _, _| {}} /> };
    let path: gpui::Stateful<gpui::Div> = rsx! { <ui::card onClick={|_, _, _| {}} /> };
    let base: gpui::Stateful<gpui::Div> =
        rsx! { <Custom base={div().flex()} onHover={|_, _, _| {}} /> };
    for element in [native, path, base] {
        assert!(Element::id(&element).is_some());
    }
}

#[test]
fn component_event_props_keep_the_builder_type_and_infer_closures() {
    let _ = rsx! { <Card onClick={|_, window, _| { let _ = window; }} onHover={|hovered, _, _| { let _: &bool = hovered; }} /> }.build();
}

#[test]
fn interactive_builders_without_element_keep_their_own_setters() {
    struct EventBuilder;
    impl gpui::GeneratedInteractive for EventBuilder {}
    impl InteractiveElement for EventBuilder {}
    impl EventBuilder {
        fn on_click(
            self,
            _: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App),
        ) -> Self {
            self
        }
    }
    let _: EventBuilder = rsx! { <Builder base={EventBuilder} onClick={|_, _, _| {}} /> };
    let _: EventBuilder = rsx! { <button base={EventBuilder} onClick={|_, _, _| {}} /> };
}

#[test]
fn keyed_component_builders_evaluate_expressions_once() {
    let calls = Cell::new(0);
    let element = rsx! {
        <Card base={{ calls.set(calls.get() + 1); Card() }}
            key={{ assert_eq!(calls.get(), 1); calls.set(calls.get() + 100); 7u64 }}
            onClick={|_, _, _| {}} onHover={|_, _, _| {}} />
    };
    let _ = element.build();
    assert_eq!(calls.get(), 101);
}

#[test]
fn key_expressions_keep_question_mark_control_flow() {
    fn render(key: Option<u64>) -> Option<gpui::Stateful<gpui::Div>> {
        Some(rsx! { <ui::card key={key?} onClick={|_, _, _| {}} /> })
    }
    assert!(render(Some(42)).is_some());
    assert!(render(None).is_none());
}

#[test]
fn explicit_ids_and_keyed_path_elements_keep_their_identity() {
    let explicit = rsx! { <ui::card id="explicit" onClick={|_, _, _| {}} /> };
    assert_eq!(Element::id(&explicit), Some(ElementId::from("explicit")));
    let calls = Cell::new(0);
    let keyed =
        rsx! { <ui::card key={{ calls.set(calls.get() + 1); 42u64 }} onClick={|_, _, _| {}} /> };
    assert!(Element::id(&keyed).is_some());
    assert_eq!(calls.get(), 1);
    let _ = rsx! {
        <div>{for key in 0..2u64 {
            <Card key={key} onClick={|_, _, _| {}} onHover={|_, _, _| {}} />
            <ui::card key={key} onClick={|_, _, _| {}} />
        }}</div>
    };
}
