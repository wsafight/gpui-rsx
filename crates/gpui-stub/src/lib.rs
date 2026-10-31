//! A minimal stand-in for [GPUI](https://www.gpui.rs/) used by `gpui-rsx` doctests.
//!
//! Every builder method is a no-op that returns `self`. The generic methods come from
//! `generated.rs` (produced by `cargo xtask gen-tables` from the real GPUI sources); methods
//! whose closures need concrete argument types for inference are written by hand below.

#![allow(clippy::new_without_default)]

mod generated;

pub use generated::{GeneratedInteractive, GeneratedStateful, GeneratedStyled};
pub use gpui_rsx_gpui_stub_macros::IntoElement;

use std::fmt;
use std::marker::PhantomData;

pub mod prelude {
    pub use crate::{
        FluentBuilder, GeneratedInteractive, GeneratedStateful, GeneratedStyled,
        InteractiveElement, IntoElement, ParentElement, Render, RenderOnce,
        StatefulInteractiveElement, Styled,
    };
}

// ---------------------------------------------------------------------------
// Values
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pixels(pub f32);
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rems(pub f32);
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DefiniteLength {
    Absolute(f32),
    Fraction(f32),
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rgba(pub u32);
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hsla {
    pub h: f32,
    pub s: f32,
    pub l: f32,
    pub a: f32,
}

impl From<Rgba> for Hsla {
    fn from(_: Rgba) -> Self {
        Hsla::default()
    }
}

pub fn px(value: f32) -> Pixels {
    Pixels(value)
}
pub fn rems(value: f32) -> Rems {
    Rems(value)
}
pub fn relative(fraction: f32) -> DefiniteLength {
    DefiniteLength::Fraction(fraction)
}
pub fn rgb(hex: u32) -> Rgba {
    Rgba((hex << 8) | 0xff)
}
pub fn rgba(hex: u32) -> Rgba {
    Rgba(hex)
}
pub fn hsla(h: f32, s: f32, l: f32, a: f32) -> Hsla {
    Hsla { h, s, l, a }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontWeight(pub f32);

impl FontWeight {
    pub const THIN: FontWeight = FontWeight(100.0);
    pub const EXTRA_LIGHT: FontWeight = FontWeight(200.0);
    pub const LIGHT: FontWeight = FontWeight(300.0);
    pub const NORMAL: FontWeight = FontWeight(400.0);
    pub const MEDIUM: FontWeight = FontWeight(500.0);
    pub const SEMIBOLD: FontWeight = FontWeight(600.0);
    pub const BOLD: FontWeight = FontWeight(700.0);
    pub const EXTRA_BOLD: FontWeight = FontWeight(800.0);
    pub const BLACK: FontWeight = FontWeight(900.0);
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SharedString(String);

impl From<&str> for SharedString {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
impl From<String> for SharedString {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl fmt::Display for SharedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ElementId(String);

impl From<&str> for ElementId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
impl From<SharedString> for ElementId {
    fn from(value: SharedString) -> Self {
        Self(value.0)
    }
}
impl From<usize> for ElementId {
    fn from(value: usize) -> Self {
        Self(value.to_string())
    }
}

impl From<(&'static str, u64)> for ElementId {
    fn from((prefix, value): (&'static str, u64)) -> Self {
        Self(format!("{prefix}{value}"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Default)]
pub struct ClickEvent;
#[derive(Debug, Default)]
pub struct MouseDownEvent;
#[derive(Debug, Default)]
pub struct MouseUpEvent;
#[derive(Debug, Default)]
pub struct KeyDownEvent;

// ---------------------------------------------------------------------------
// App / window / views
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct App;
#[derive(Default)]
pub struct Window;

pub struct Context<'a, V> {
    app: App,
    _view: PhantomData<&'a mut V>,
}

impl<V> Context<'_, V> {
    /// Creates a detached context, useful for tests and doctests.
    pub fn new() -> Self {
        Self {
            app: App,
            _view: PhantomData,
        }
    }

    pub fn listener<E: ?Sized>(
        &self,
        f: impl Fn(&mut V, &E, &mut Window, &mut Context<V>) + 'static,
    ) -> impl Fn(&E, &mut Window, &mut App) + 'static {
        move |_e: &E, _window: &mut Window, _cx: &mut App| {
            let _ = &f;
        }
    }

    pub fn notify(&mut self) {}
}

impl<V> std::ops::Deref for Context<'_, V> {
    type Target = App;
    fn deref(&self) -> &App {
        &self.app
    }
}

impl<V> std::ops::DerefMut for Context<'_, V> {
    fn deref_mut(&mut self) -> &mut App {
        &mut self.app
    }
}

pub struct AnyView;

pub trait Render: 'static + Sized {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement;
}

pub trait RenderOnce: 'static {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement;
}

// ---------------------------------------------------------------------------
// Elements
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct Div;
pub struct Stateful<E> {
    element: E,
    id: ElementId,
}
#[derive(Default)]
pub struct Svg;
#[derive(Default)]
pub struct Img;
pub struct Canvas<T>(PhantomData<T>);
#[derive(Default, Debug)]
pub struct StyleRefinement;
pub struct AnyElement;
pub struct Bounds;

/// Rendering elements, distinct from `RenderOnce` component builders.
pub trait Element: Sized {
    fn id(&self) -> Option<ElementId> {
        None
    }
}

impl Element for Div {}
impl Element for Svg {}
impl Element for Img {}
impl<T> Element for Canvas<T> {}
impl<E: Element> Element for Stateful<E> {
    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }
}

pub fn div() -> Div {
    Div
}
pub fn svg() -> Svg {
    Svg
}
pub fn img(_source: impl Into<SharedString>) -> Img {
    Img
}
pub fn canvas<T>(
    _prepaint: impl 'static + FnOnce(Bounds, &mut Window, &mut App) -> T,
    _paint: impl 'static + FnOnce(Bounds, T, &mut Window, &mut App),
) -> Canvas<T> {
    Canvas(PhantomData)
}

impl Svg {
    pub fn path(self, _path: impl Into<SharedString>) -> Self {
        self
    }
}

pub trait IntoElement: Sized {
    type Element;
    fn into_element(self) -> Self::Element;
    fn into_any_element(self) -> AnyElement {
        AnyElement
    }
}

macro_rules! self_into_element {
    ($($ty:ty),* $(,)?) => {
        $(impl IntoElement for $ty {
            type Element = Self;
            fn into_element(self) -> Self { self }
        })*
    };
}

self_into_element!(
    Div,
    Svg,
    Img,
    AnyElement,
    &'static str,
    String,
    SharedString
);

impl<T> IntoElement for Canvas<T> {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl<E: Element> IntoElement for Stateful<E> {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl<T: RenderOnce> IntoElement for T {
    type Element = AnyElement;
    fn into_element(self) -> AnyElement {
        AnyElement
    }
}

impl<T: IntoElement> IntoElement for Option<T> {
    type Element = AnyElement;
    fn into_element(self) -> AnyElement {
        AnyElement
    }
}

pub trait Styled: GeneratedStyled {
    fn style(&mut self) -> &mut StyleRefinement;
}

impl Styled for Div {
    fn style(&mut self) -> &mut StyleRefinement {
        Box::leak(Box::default())
    }
}
impl<E: Styled> Styled for Stateful<E> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.element.style()
    }
}
impl Styled for StyleRefinement {
    fn style(&mut self) -> &mut StyleRefinement {
        self
    }
}
impl Styled for Svg {
    fn style(&mut self) -> &mut StyleRefinement {
        Box::leak(Box::default())
    }
}
impl Styled for Img {
    fn style(&mut self) -> &mut StyleRefinement {
        Box::leak(Box::default())
    }
}
impl<T> Styled for Canvas<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        Box::leak(Box::default())
    }
}

pub trait InteractiveElement: GeneratedInteractive {
    fn id(self, id: impl Into<ElementId>) -> Stateful<Self> {
        Stateful {
            element: self,
            id: id.into(),
        }
    }
    fn hover(self, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
        let _ = f(StyleRefinement);
        self
    }
    fn group_hover(
        self,
        _group: impl Into<SharedString>,
        f: impl FnOnce(StyleRefinement) -> StyleRefinement,
    ) -> Self {
        let _ = f(StyleRefinement);
        self
    }
    fn focus(self, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
        let _ = f(StyleRefinement);
        self
    }
    fn focus_visible(self, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
        let _ = f(StyleRefinement);
        self
    }
    fn in_focus(self, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
        let _ = f(StyleRefinement);
        self
    }
    fn on_mouse_down(
        self,
        _button: MouseButton,
        _listener: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self
    }
    fn on_mouse_up(
        self,
        _button: MouseButton,
        _listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self
    }
    fn on_key_down(
        self,
        _listener: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self
    }
}

impl InteractiveElement for Div {}
impl<E: InteractiveElement> InteractiveElement for Stateful<E> {}
impl InteractiveElement for Svg {}
impl InteractiveElement for Img {}

pub trait StatefulInteractiveElement: InteractiveElement + GeneratedStateful {
    fn active(self, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
        let _ = f(StyleRefinement);
        self
    }
    fn group_active(
        self,
        _group: impl Into<SharedString>,
        f: impl FnOnce(StyleRefinement) -> StyleRefinement,
    ) -> Self {
        let _ = f(StyleRefinement);
        self
    }
    fn on_click(self, _listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self
    }
    fn on_hover(self, _listener: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self
    }
    fn tooltip(self, _build: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self {
        self
    }
}

impl<E: Element + InteractiveElement> StatefulInteractiveElement for Stateful<E> {}

pub trait ParentElement: Sized {
    fn extend(&mut self, _elements: impl IntoIterator<Item = AnyElement>) {}
    fn child(self, child: impl IntoElement) -> Self {
        let _ = child.into_any_element();
        self
    }
    fn children(self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        for child in children {
            let _ = child.into_any_element();
        }
        self
    }
}

impl ParentElement for Div {}
impl<E: ParentElement> ParentElement for Stateful<E> {}

pub trait FluentBuilder: Sized {
    fn map<U>(self, f: impl FnOnce(Self) -> U) -> U {
        f(self)
    }
    fn when(self, condition: bool, then: impl FnOnce(Self) -> Self) -> Self {
        if condition { then(self) } else { self }
    }
    fn when_else(
        self,
        condition: bool,
        then: impl FnOnce(Self) -> Self,
        else_fn: impl FnOnce(Self) -> Self,
    ) -> Self {
        if condition { then(self) } else { else_fn(self) }
    }
    fn when_some<T>(self, option: Option<T>, then: impl FnOnce(Self, T) -> Self) -> Self {
        match option {
            Some(value) => then(self, value),
            None => self,
        }
    }
    fn when_none<T>(self, option: &Option<T>, then: impl FnOnce(Self) -> Self) -> Self {
        if option.is_none() { then(self) } else { self }
    }
}

impl<T: IntoElement> FluentBuilder for T {}
