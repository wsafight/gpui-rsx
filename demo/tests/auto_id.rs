//! Exercise auto-ID dispatch against real GPUI and gpui-component builders.
use gpui::{Element, ElementId, component::button::Button, div, prelude::*};
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
fn native_and_path_elements_still_receive_auto_ids() {
    let native: gpui::Stateful<gpui::Div> = rsx! { <div onClick={|_, _, _| {}} /> };
    let path: gpui::Stateful<gpui::Div> = rsx! { <ui::card onClick={|_, _, _| {}} /> };
    let base: gpui::Stateful<gpui::Div> =
        rsx! { <Custom base={div().flex()} onHover={|_, _, _| {}} /> };
    for element in [native, path, base] {
        assert!(Element::id(&element).is_some());
    }
    let explicit = rsx! { <ui::card id="explicit" onClick={|_, _, _| {}} /> };
    assert_eq!(Element::id(&explicit), Some(ElementId::from("explicit")));
}

#[test]
fn third_party_buttons_keep_their_inherent_event_setters() {
    let _: Button = rsx! {
        <Button base={Button::new("save")} onClick={|_, window, _| window.refresh()} />
    };
    // A base expression can also replace an HTML tag's constructor.
    let _: Button = rsx! { <button base={Button::new("save")} onClick={|_, _, _| {}} /> };
}

#[test]
fn component_callback_props_compile_without_an_id_setter() {
    let _ = rsx! {
        <Card onClick={|_, window, _| window.refresh()}
            onHover={|hovered, _, _| { let _: &bool = hovered; }} />
    }
    .build();
}

#[test]
fn keyed_component_tags_support_elements_and_builders() {
    let calls = Cell::new(0);
    let element = rsx! {
        <ui::card key={{ calls.set(calls.get() + 1); 42u64 }} onClick={|_, _, _| {}} />
    };
    assert!(matches!(
        Element::id(&element),
        Some(ElementId::NamedInteger(_, 42))
    ));
    assert_eq!(calls.get(), 1);
    let _ = rsx! {
        <div>{for key in 0..2u64 {
            <ui::card key={key} onClick={|_, _, _| {}} />
            <Button base={Button::new(("save", key))} key={key} onClick={|_, _, _| {}} />
            <Card key={key} onClick={|_, _, _| {}} onHover={|_, _, _| {}} />
        }}</div>
    };
}

fn generic_card<E: InteractiveElement>(element: E) -> impl IntoElement
where
    gpui::Stateful<E>: StatefulInteractiveElement + IntoElement,
{
    rsx! { <Custom base={element} id="generic" onClick={|_, _, _| {}} /> }
}

#[test]
fn generic_elements_can_use_explicit_ids() {
    let _ = generic_card(div());
}

fn generic_element_with_auto_id<E: Element + InteractiveElement>(element: E) -> gpui::Stateful<E> {
    rsx! { <Custom base={element} onClick={|_, _, _| {}} /> }
}

#[test]
fn generic_elements_with_the_required_bounds_receive_auto_ids() {
    assert!(Element::id(&generic_element_with_auto_id(div())).is_some());
}

#[test]
fn key_expressions_keep_question_mark_control_flow() {
    fn render(key: Option<u64>) -> Option<gpui::Stateful<gpui::Div>> {
        Some(rsx! { <ui::card key={key?} onClick={|_, _, _| {}} /> })
    }
    assert!(matches!(
        Element::id(&render(Some(42)).unwrap()),
        Some(ElementId::NamedInteger(_, 42))
    ));
    assert!(render(None).is_none());
}
