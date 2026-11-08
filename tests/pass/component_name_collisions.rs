#![allow(non_snake_case)]

use gpui::prelude::*;
use gpui::*;
use gpui_rsx::component;

#[component]
fn TypeGeneric<__T: Clone + 'static>(t: __T) -> impl IntoElement {
    let _ = t;
    div()
}

#[component]
fn ConstGeneric<const __N: usize>(n: usize) -> impl IntoElement {
    let _ = (n, __N);
    div()
}

#[component]
fn FinalSuffix<__T1: Clone + 'static>(t: u8, t_: u8, value: __T1) -> impl IntoElement {
    let _ = (t, t_, value);
    div()
}

#[component]
fn Underscores(a_b: u8, a__b: u8, a___b: u8) -> impl IntoElement {
    let _ = (a_b, a__b, a___b);
    div()
}

#[component]
fn ReservedState<__Phantom: 'static>(
    __rsx_state: u8,
    __rsx_state_: u8,
    __rsx_state__: u8,
    __rsx_generics: u8,
) -> impl IntoElement {
    let _ = (
        __rsx_state,
        __rsx_state_,
        __rsx_state__,
        __rsx_generics,
        core::marker::PhantomData::<__Phantom>,
    );
    div()
}

#[component]
#[derive(IntoElement)]
struct StructDirect<__T: Clone + 'static, const __N: usize> {
    t: __T,
    n: [u8; __N],
    __rsx_state: u8,
    #[prop(default)]
    __rsx_state_: u8,
}

impl<__T: Clone + 'static, const __N: usize> RenderOnce for StructDirect<__T, __N> {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let _ = (self.t, self.n, self.__rsx_state, self.__rsx_state_);
        div()
    }
}

fn main() {
    let type_generic = TypeGeneric().t(1u8).build();
    assert_eq!(type_generic.t, 1);

    let const_generic = ConstGeneric::<4>().n(2usize).build();
    assert_eq!(const_generic.n, 2);

    let final_suffix = FinalSuffix::<u16>().t(3u8).t_(4u8).value(5u16).build();
    assert_eq!(
        (final_suffix.t, final_suffix.t_, final_suffix.value),
        (3, 4, 5)
    );

    let underscores = Underscores().a_b(6u8).a__b(7u8).a___b(8u8).build();
    assert_eq!(
        (underscores.a_b, underscores.a__b, underscores.a___b),
        (6, 7, 8)
    );

    let reserved = ReservedState::<u8>()
        .__rsx_state(9u8)
        .__rsx_state_(10u8)
        .__rsx_state__(11u8)
        .__rsx_generics(12u8)
        .build();
    assert_eq!(
        (
            reserved.__rsx_state,
            reserved.__rsx_state_,
            reserved.__rsx_state__,
            reserved.__rsx_generics,
        ),
        (9, 10, 11, 12)
    );

    let direct = StructDirect::<u8, 2>()
        .t(13u8)
        .n([14u8, 15])
        .__rsx_state(16u8)
        .__rsx_state_(17u8)
        .build();
    assert_eq!(direct.t, 13);
    assert_eq!(direct.n, [14, 15]);
    assert_eq!((direct.__rsx_state, direct.__rsx_state_), (16, 17));
}
