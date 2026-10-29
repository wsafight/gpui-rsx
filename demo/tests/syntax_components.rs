use gpui::{AnyElement, App, IntoElement, RenderOnce, SharedString, Window, div, prelude::*};
use gpui_rsx::{component, rsx};
use std::{cell::Cell, rc::Rc};

#[component]
#[derive(gpui::IntoElement)]
struct Props {
    title: SharedString,
    count: usize,
    #[prop(default)]
    enabled: bool,
    #[prop(default = true)]
    expanded: bool,
    #[prop(default = "fallback")]
    caption: SharedString,
    #[prop(default)]
    hint: Option<SharedString>,
    #[children]
    children: Vec<AnyElement>,
}
impl RenderOnce for Props {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().child(self.title).children(self.children)
    }
}

#[test]
fn required_literal_props_and_defaults_keep_their_values() {
    let props = rsx! { <Props title="Inbox" count=3usize /> }.build();
    assert_eq!(props.title, "Inbox");
    assert_eq!(props.count, 3);
    assert!(!props.enabled);
    assert!(props.expanded);
    assert_eq!(props.caption, "fallback");
    assert!(props.hint.is_none());
    assert!(props.children.is_empty());
}

#[test]
fn boolean_flags_and_camel_case_if_setters_override_defaults() {
    let props = rsx! { <Props title="Inbox" count={1usize + 2} enabled expandedIf={false} caption="custom" /> }.build();
    assert!(props.enabled);
    assert!(!props.expanded);
    assert_eq!(props.caption, "custom");
    for enabled in [false, true] {
        let props = rsx! { <Props title="Inbox" count=0usize enabledIf={enabled} /> }.build();
        assert_eq!(props.enabled, enabled);
    }
}

#[test]
fn option_props_accept_inner_some_and_none() {
    let inner = rsx! { <Props title="A" count=0usize hint={SharedString::from("hint")} /> }.build();
    let some =
        rsx! { <Props title="A" count=0usize hint={Some(SharedString::from("hint"))} /> }.build();
    let none = rsx! { <Props title="A" count=0usize hint={None} /> }.build();
    assert_eq!(inner.hint, some.hint);
    assert_eq!(inner.hint.as_deref(), Some("hint"));
    assert!(none.hint.is_none());
}

#[test]
fn component_children_collect_fragments_loops_spread_and_text() {
    let extra = rsx! { <><div/><div/></> };
    let props = rsx! {
        <Props title="A" count=0usize>
            "literal"
            {String::from("expression")}
            <><span/><></><span/></>
            {...extra}
            {for n in 0..3 { <span>{n.to_string()}</span> }}
        </Props>
    }
    .build();
    assert_eq!(props.children.len(), 9);
}

trait OnValue: Fn(u8) -> u8 {}
impl<F: Fn(u8) -> u8> OnValue for F {}
type CallbackAlias = Rc<dyn Fn(u8) -> u8>;

#[component]
#[derive(gpui::IntoElement)]
struct Callbacks {
    boxed: Box<dyn Fn(u8) -> u8>,
    shared: Rc<dyn Fn(u8) -> u8>,
    mutable: Box<dyn FnMut(u8) -> u8>,
    once: Box<dyn FnOnce(u8) -> u8>,
    #[prop(callback)]
    custom: Rc<dyn OnValue>,
    alias: CallbackAlias,
    #[prop(default = Rc::new(|value| value + 1))]
    defaulted: Rc<dyn Fn(u8) -> u8>,
}
impl RenderOnce for Callbacks {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
    }
}

#[test]
fn callback_types_infer_arguments_and_execute_with_captured_state() {
    let state = Rc::new(Cell::new(0));
    let sink = state.clone();
    let alias: CallbackAlias = Rc::new(|value| value + 10);
    let mut sum = 0;
    let mut props = rsx! {
        <Callbacks boxed={|n| n + 1} shared={move |n| { sink.set(n); n + 2 }}
            mutable={move |n| { sum += n; sum }}
            once={|n| n + 4} custom={|n| n + 5} alias={alias.clone()} />
    }
    .build();
    assert_eq!((props.boxed)(2), 3);
    assert_eq!((props.shared)(2), 4);
    assert_eq!(state.get(), 2);
    assert_eq!((props.mutable)(2), 2);
    assert_eq!((props.mutable)(3), 5);
    assert_eq!((props.custom)(2), 7);
    assert_eq!((props.alias)(2), 12);
    assert_eq!((props.defaulted)(2), 3);
    assert_eq!((props.once)(2), 6);
}

#[component]
#[derive(gpui::IntoElement)]
struct Generic<T = String, const N: usize = 2>
where
    T: Into<SharedString> + Clone + 'static,
{
    value: T,
    values: [u8; N],
}
impl<T: Into<SharedString> + Clone + 'static, const N: usize> RenderOnce for Generic<T, N> {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().child(self.value.into())
    }
}

#[test]
fn generic_defaults_const_parameters_and_where_clauses_are_preserved() {
    let props = rsx! { <Generic base={Generic::<String, 3>()} value={String::from("value")} values={[1, 2, 3]} /> }.build();
    assert_eq!(props.value, "value");
    assert_eq!(props.values, [1, 2, 3]);
    let props = Generic()
        .value(String::from("inferred"))
        .values([7u8, 8])
        .build();
    assert_eq!(props.values, [7, 8]);
}

mod ui {
    use super::*;
    #[component]
    pub(super) fn Label(label: &'static str) -> impl IntoElement {
        div().child(label)
    }
}

#[test]
fn path_tags_and_static_reference_function_props_are_accepted() {
    let _: AnyElement = rsx! { <ui::Label label="static label" /> }.into_any_element();
}

#[component]
fn ConstFunction<const N: usize>(values: [u8; N]) -> impl IntoElement {
    div().child(values.len().to_string())
}

#[test]
fn function_component_const_parameters_work_with_explicit_base_constructors() {
    let props = rsx! { <ConstFunction base={ConstFunction::<3>()} values={[1, 2, 3]} /> }.build();
    assert_eq!(props.values, [1, 2, 3]);
    let _: AnyElement = props.into_any_element();
}

#[test]
fn local_function_components_can_use_types_from_the_enclosing_function() {
    struct Local;
    #[component]
    fn Inner(value: Rc<dyn Fn() -> Local>) -> impl IntoElement {
        let _: Local = value();
        div()
    }
    let _: AnyElement = rsx! { <Inner value={|| Local} /> }.into_any_element();
}

#[test]
fn repeated_props_and_direct_child_builders_keep_the_last_values() {
    let props = Props()
        .title("first")
        .title("last")
        .count(1usize)
        .enabled()
        .enabled_if(false)
        .child(div())
        .children([div(), div()])
        .build();
    assert_eq!(props.title, "last");
    assert!(!props.enabled);
    assert_eq!(props.children.len(), 3);
}
