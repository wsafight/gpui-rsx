//! Real GPUI derives must accept the generated generic and callback builders.
use gpui::{SharedString, div, prelude::*};
use gpui_rsx::{component, rsx};
use std::rc::Rc;

#[component]
fn Generic<T>(
    value: T,
    callback: Rc<dyn Fn(T)>,
    #[prop(default)] label: Option<SharedString>,
) -> impl IntoElement
where
    T: Clone + Into<SharedString>,
{
    callback(value.clone());
    div().child(label.unwrap_or_else(|| value.into()))
}

/// Trait alias for GPUI's click handler; `#[prop(callback)]` keeps closure inference.
trait OnPress: Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) {}
impl<F: Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App)> OnPress for F {}

#[component]
fn Pressable(#[prop(callback)] on_press: Rc<dyn OnPress>) -> impl IntoElement {
    div()
        .id("pressable")
        .on_click(move |event, window, cx| on_press(event, window, cx))
}

#[component]
#[derive(gpui::IntoElement)]
struct ArrayCard<const N: usize> {
    values: [u8; N],
}
impl<const N: usize> gpui::RenderOnce for ArrayCard<N> {
    fn render(self, _: &mut gpui::Window, _: &mut gpui::App) -> impl IntoElement {
        div().child(self.values.len().to_string())
    }
}

#[test]
fn generic_and_callback_builders_implement_real_into_element() {
    let _: gpui::AnyElement = rsx! { <Generic value={String::from("value")} callback={|_: String| {}} label={SharedString::from("label")} /> }.into_any_element();
    let callback: Rc<dyn Fn(String)> = Rc::new(|_| {});
    let _: gpui::AnyElement = Generic()
        .value(String::from("value"))
        .callback(move |value| callback(value))
        .label(None)
        .into_any_element();
    let _: gpui::AnyElement =
        rsx! { <ArrayCard base={ArrayCard::<3>()} values={[1,2,3]} /> }.into_any_element();
    let _: gpui::AnyElement =
        rsx! { <Pressable on_press={|_, window, _| window.refresh()} /> }.into_any_element();
}

#[component]
fn Empty<T: 'static>() -> impl IntoElement {
    let _ = std::marker::PhantomData::<T>;
    div()
}

#[test]
fn generics_used_only_in_the_function_body_can_be_explicit() {
    let _: gpui::AnyElement = rsx! { <Empty base={Empty::<String>()} /> }.into_any_element();
}

mod defaults {
    // Keep the written callback type to verify unsizing and argument inference in real GPUI.
    #![allow(clippy::type_complexity)]
    use super::*;

    #[component]
    fn Defaults(
        #[prop(default = Box::new(|_, _, _| {}))] on_click: Box<
            dyn Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App),
        >,
        #[prop(callback, default = Rc::new(|_, _, _| {}))] on_press: Rc<dyn OnPress>,
        #[prop(default = Rc::new(|text| text.len()))] measure: Rc<dyn Fn(&str) -> usize>,
        #[prop(default = "default")] label: SharedString,
    ) -> impl IntoElement {
        assert_eq!(measure(&label), label.len());
        div()
            .id("default-callbacks")
            .on_click(move |event, window, cx| {
                on_click(event, window, cx);
                on_press(event, window, cx);
            })
    }

    #[test]
    fn callback_defaults_coerce_and_remain_overridable() {
        let card = rsx! { <Defaults /> }.build();
        assert_eq!((card.measure)("abc"), 3);
        let _: gpui::AnyElement = card.into_any_element();
        let _: gpui::AnyElement = rsx! {
            <Defaults onClick={|_, window, _| window.refresh()} />
        }
        .into_any_element();
    }
}
