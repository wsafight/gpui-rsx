use gpui::{SharedString, div, prelude::*};
use gpui_rsx::{component, rsx};
use std::{cell::Cell, rc::Rc};

#[allow(unknown_lints)]
#[deny(float_literal_f32_fallback)]
mod numeric_defaults {
    use super::*;

    #[component]
    fn Counter(#[prop(default = 7)] value: u32) -> impl IntoElement {
        div().child(value.to_string())
    }

    #[component]
    #[derive(gpui::IntoElement)]
    struct Defaults {
        #[prop(default = 7)]
        count: u32,
        #[prop(default = 7)]
        size: usize,
        #[prop(default = 7.0)]
        ratio: f32,
        #[prop(default = -7.0)]
        negative: f32,
        #[prop(default = 7 + 2)]
        sum: u32,
        #[prop(default = { 1 << 8 })]
        shifted: core::primitive::u32,
        #[prop(default = 7u8)]
        widened: u32,
        #[prop(default = 7 < 8)]
        compared: u32,
        #[prop(default = "ready")]
        label: SharedString,
    }

    impl RenderOnce for Defaults {
        fn render(self, _: &mut gpui::Window, _: &mut gpui::App) -> impl IntoElement {
            div().child(self.label)
        }
    }

    #[test]
    fn numeric_defaults_infer_field_types_and_preserve_conversions() {
        assert_eq!(rsx! { <Counter /> }.build().value, 7);
        let defaults = rsx! { <Defaults /> }.build();
        assert_eq!(defaults.count, 7);
        assert_eq!(defaults.size, 7);
        assert_eq!(defaults.ratio, 7.0);
        assert_eq!(defaults.negative, -7.0);
        assert_eq!(defaults.sum, 9);
        assert_eq!(defaults.shifted, 256);
        assert_eq!(defaults.widened, 7);
        assert_eq!(defaults.compared, 1);
        assert_eq!(defaults.label, SharedString::from("ready"));
        assert_eq!(rsx! { <Counter value={11u32} /> }.build().value, 11);
    }

    #[test]
    fn integer_defaults_support_signed_unsigned_and_qualified_types() {
        macro_rules! check {
            ($ty:ty, $value:expr) => {{
                #[component]
                fn Value(#[prop(default = $value)] value: $ty) -> impl IntoElement {
                    div().child(value.to_string())
                }
                assert_eq!(rsx! { <Value /> }.build().value, $value);
            }};
        }
        check!(u8, 255);
        check!(u16, 65535);
        check!(u64, 18446744073709551615);
        check!(u128, 340282366920938463463374607431768211455);
        check!(i8, -128);
        check!(i16, -32768);
        check!(i32, -2147483648);
        check!(i64, -9223372036854775808);
        check!(i128, -170141183460469231731687303715884105728);
        check!(isize, -7);
        check!(std::primitive::usize, 7);
    }

    #[test]
    fn numeric_newtypes_keep_their_into_defaults() {
        struct Number(i32);

        impl From<i32> for Number {
            fn from(value: i32) -> Self {
                Self(value)
            }
        }

        #[component]
        fn Converted(#[prop(default = 7)] value: Number) -> impl IntoElement {
            div().child(value.0.to_string())
        }

        assert_eq!(rsx! { <Converted /> }.build().value.0, 7);
    }
}

#[component]
fn Callbacks(
    boxed: Box<dyn Fn(u8)>,
    shared: Rc<dyn Fn(u8)>,
    #[prop(default)] label: Option<SharedString>,
) -> impl IntoElement {
    boxed(2);
    shared(3);
    div().child(label.unwrap_or_else(|| "empty".into()))
}

#[component]
fn Generic<T: Clone + Into<SharedString>>(value: T, callback: Rc<dyn Fn(T)>) -> impl IntoElement {
    callback(value.clone());
    div().child(value.into())
}

#[component]
#[derive(gpui::IntoElement)]
struct ArrayCard<const N: usize> {
    values: [u8; N],
}
impl<const N: usize> gpui::RenderOnce for ArrayCard<N> {
    fn render(self, _: &mut gpui::Window, _: &mut gpui::App) -> impl IntoElement {
        let _ = self.values;
        div()
    }
}

#[test]
fn closures_and_existing_rc_callbacks_both_work_without_wrappers() {
    let sum = Rc::new(Cell::new(0));
    let a = sum.clone();
    let b = sum.clone();
    let card = Callbacks()
        .boxed(move |n| a.set(a.get() + n))
        .shared(move |n| b.set(b.get() + n))
        .label(SharedString::from("label"))
        .build();
    let _ = card.render(&mut gpui::Window, &mut gpui::App);
    assert_eq!(sum.get(), 5);
    let a = sum.clone();
    let shared: Rc<dyn Fn(u8)> = Rc::new(move |n| a.set(a.get() + n));
    // Existing shared callbacks are forwarded through a closure, like GPUI's own setters.
    let forward = shared.clone();
    let card = Callbacks()
        .boxed(|_| {})
        .shared(move |n| forward(n))
        .label(None)
        .build();
    let _ = card.render(&mut gpui::Window, &mut gpui::App);
    assert_eq!(sum.get(), 8);
}

/// Trait alias for a GPUI handler, keeping the prop type short without a `type` alias.
trait OnPress: Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) {}
impl<F: Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App)> OnPress for F {}

fn ignore_press() -> Box<dyn OnPress> {
    Box::new(|_: &gpui::ClickEvent, _: &mut gpui::Window, _: &mut gpui::App| {})
}

#[component]
fn Handlers(
    #[prop(callback)] on_press: Rc<dyn OnPress>,
    #[prop(callback, default = ignore_press())] on_release: Box<dyn OnPress>,
    measure: Rc<dyn Fn(&str) -> usize>,
) -> impl IntoElement {
    on_press(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App);
    on_release(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App);
    div().child(SharedString::from(measure("abc").to_string()))
}

#[test]
fn shared_callbacks_infer_closure_arguments_and_lifetimes() {
    let pressed = Rc::new(Cell::new(0));
    let (press, release) = (pressed.clone(), pressed.clone());
    let card = rsx! {
        <Handlers on_press={move |_, _, _| press.set(press.get() + 1)} measure={|s| s.len()} />
    };
    let _ = card.build().render(&mut gpui::Window, &mut gpui::App);
    let card = rsx! {
        <Handlers on_press={|_, _, _| {}} on_release={move |_, _, _| release.set(release.get() + 2)}
            measure={|s| s.len()} />
    };
    let _ = card.build().render(&mut gpui::Window, &mut gpui::App);
    assert_eq!(pressed.get(), 3);
}

#[test]
fn components_declared_inside_functions_accept_shared_callbacks() {
    struct Local(u8);

    #[component]
    fn Inner(value: Rc<dyn Fn() -> Local>) -> impl IntoElement {
        let _ = value().0;
        div()
    }

    let _ = rsx! { <Inner value={|| Local(1)} /> }.build();
}

#[test]
fn generic_functions_infer_types_and_const_structs_support_explicit_constructors() {
    let card = rsx! { <Generic value={String::from("value")} callback={|value: String| assert_eq!(value, "value")} /> };
    let _ = card.build().render(&mut gpui::Window, &mut gpui::App);
    let card = rsx! { <ArrayCard base={ArrayCard::<3>()} values={[1,2,3]} /> };
    let _ = card.build().render(&mut gpui::Window, &mut gpui::App);
}

#[component]
fn Empty<T: 'static>() -> impl IntoElement {
    let _ = std::marker::PhantomData::<T>;
    div()
}
#[test]
fn generic_parameters_need_not_appear_in_public_props() {
    let _ = rsx! { <Empty base={Empty::<String>()} /> }.build();
}

mod defaults {
    // Written-out callback types exercise coercion and closure argument inference.
    #![allow(clippy::type_complexity)]
    use super::*;

    #[component]
    fn Defaults(
        #[prop(default = Box::new(|_, _, _| {}))] on_click: Box<
            dyn Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App),
        >,
        #[prop(callback, default = Box::new(|_, _, _| {}))] on_press: Box<dyn OnPress>,
        #[prop(default = Rc::new(|text| text.len()))] measure: Rc<dyn Fn(&str) -> usize>,
        #[prop(default = "default")] label: String,
    ) -> impl IntoElement {
        on_click(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App);
        on_press(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App);
        assert_eq!(measure(&label), label.len());
        div()
    }

    #[test]
    fn callback_defaults_unsize_and_infer_closure_arguments() {
        let _ = rsx! { <Defaults /> }
            .build()
            .render(&mut gpui::Window, &mut gpui::App);
        let calls = Rc::new(Cell::new(0));
        let sink = calls.clone();
        let _ = rsx! { <Defaults onClick={move |_, _, _| sink.set(sink.get() + 1)} /> }
            .build()
            .render(&mut gpui::Window, &mut gpui::App);
        assert_eq!(calls.get(), 1);
    }
}
