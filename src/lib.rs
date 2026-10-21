#![doc = include_str!("docs/overview.md")]
#![doc = include_str!("docs/elements.md")]
#![doc = include_str!("docs/classes.md")]
#![doc = include_str!("docs/events-and-children.md")]
#![doc = include_str!("docs/attributes-and-identity.md")]
use gpui_rsx_core::{MacroOptions, SpacingScale, expand_component, expand_macro};
use proc_macro::TokenStream;

fn options(strict: bool) -> MacroOptions {
    MacroOptions {
        strict,
        forced_scale: None,
        default_scale: if cfg!(feature = "tailwind-scale") {
            SpacingScale::Tailwind
        } else {
            SpacingScale::Px
        },
        report_dynamic_at_exit: cfg!(feature = "dynamic-class-report"),
    }
}

/// Transforms JSX-like markup into GPUI method chains at compile time.
///
/// # Syntax
///
/// ```text
/// rsx! { <tag attr1 attr2={expr} class="…"> {children} </tag> }
/// rsx! { <tag /> }          — self-closing
/// rsx! { <> … </> }         — fragment (returns vec![…])
/// ```
///
/// # Examples
///
/// ## Basic element with static classes
///
/// ```
/// # #![allow(unused)]
/// # use gpui::prelude::*;
/// # use gpui::*;
/// # use gpui_rsx::rsx;
/// # #[derive(Clone)]
/// # struct Item { id: usize, name: SharedString }
/// # struct Task { id: usize, title: SharedString }
/// # struct Button;
/// # impl Button {
/// #     fn new(_id: &str) -> Self { Button }
/// #     fn label(self, _label: &str) -> Self { self }
/// #     fn small(self) -> Self { self }
/// # }
/// # mod ui {
/// #     pub struct TaskCard;
/// #     impl TaskCard {
/// #         pub fn new(_id: usize) -> Self { TaskCard }
/// #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
/// #     }
/// # }
/// # struct View { count: usize, items: Vec<Item>, show: bool }
/// # impl View { fn render_child(&self) -> impl IntoElement { div() } }
/// # impl Render for View {
/// # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
/// # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
/// # let custom_color: Option<u32> = Some(0x3b82f6);
/// # let custom_width: Option<f32> = Some(120.0);
/// # let spacing = 4;
/// # let task = Task { id: 7, title: "Write docs".into() };
/// # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
/// # let _doc = {
/// rsx! {
///     <div class="flex flex-col gap-4 p-4">
///         <h1 class="text-2xl font-bold">{"Hello"}</h1>
///     </div>
/// }
/// # };
/// # div()
/// # }
/// # }
/// # fn main() {
/// #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
/// #     let _ = view.render(&mut Window::default(), &mut Context::new());
/// # }
/// ```
///
/// ## Event handling (auto ID injected)
///
/// ```
/// # #![allow(unused)]
/// # use gpui::prelude::*;
/// # use gpui::*;
/// # use gpui_rsx::rsx;
/// # #[derive(Clone)]
/// # struct Item { id: usize, name: SharedString }
/// # struct Task { id: usize, title: SharedString }
/// # struct Button;
/// # impl Button {
/// #     fn new(_id: &str) -> Self { Button }
/// #     fn label(self, _label: &str) -> Self { self }
/// #     fn small(self) -> Self { self }
/// # }
/// # mod ui {
/// #     pub struct TaskCard;
/// #     impl TaskCard {
/// #         pub fn new(_id: usize) -> Self { TaskCard }
/// #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
/// #     }
/// # }
/// # struct View { count: usize, items: Vec<Item>, show: bool }
/// # impl View { fn render_child(&self) -> impl IntoElement { div() } }
/// # impl Render for View {
/// # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
/// # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
/// # let custom_color: Option<u32> = Some(0x3b82f6);
/// # let custom_width: Option<f32> = Some(120.0);
/// # let spacing = 4;
/// # let task = Task { id: 7, title: "Write docs".into() };
/// # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
/// # let _doc = {
/// rsx! {
///     <button
///         class="bg-blue-500 text-white px-4 py-2 rounded-md"
///         onClick={cx.listener(|view, _, _window, cx| {
///             view.count += 1;
///             cx.notify();
///         })}
///     >
///         {"Click me"}
///     </button>
/// }
/// # };
/// # div()
/// # }
/// # }
/// # fn main() {
/// #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
/// #     let _ = view.render(&mut Window::default(), &mut Context::new());
/// # }
/// ```
///
/// ## For-loop rendering
///
/// ```
/// # #![allow(unused)]
/// # use gpui::prelude::*;
/// # use gpui::*;
/// # use gpui_rsx::rsx;
/// # #[derive(Clone)]
/// # struct Item { id: usize, name: SharedString }
/// # struct Task { id: usize, title: SharedString }
/// # struct Button;
/// # impl Button {
/// #     fn new(_id: &str) -> Self { Button }
/// #     fn label(self, _label: &str) -> Self { self }
/// #     fn small(self) -> Self { self }
/// # }
/// # mod ui {
/// #     pub struct TaskCard;
/// #     impl TaskCard {
/// #         pub fn new(_id: usize) -> Self { TaskCard }
/// #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
/// #     }
/// # }
/// # struct View { count: usize, items: Vec<Item>, show: bool }
/// # impl View { fn render_child(&self) -> impl IntoElement { div() } }
/// # impl Render for View {
/// # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
/// # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
/// # let custom_color: Option<u32> = Some(0x3b82f6);
/// # let custom_width: Option<f32> = Some(120.0);
/// # let spacing = 4;
/// # let task = Task { id: 7, title: "Write docs".into() };
/// # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
/// # let _doc = {
/// rsx! {
///     <ul>
///         {for item in &self.items {
///             <li>{item.name.clone()}</li>
///         }}
///     </ul>
/// }
/// # };
/// # div()
/// # }
/// # }
/// # fn main() {
/// #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
/// #     let _ = view.render(&mut Window::default(), &mut Context::new());
/// # }
/// ```
///
/// ## Fragment (multiple root elements)
///
/// ```
/// # #![allow(unused)]
/// # use gpui::prelude::*;
/// # use gpui::*;
/// # use gpui_rsx::rsx;
/// # #[derive(Clone)]
/// # struct Item { id: usize, name: SharedString }
/// # struct Task { id: usize, title: SharedString }
/// # struct Button;
/// # impl Button {
/// #     fn new(_id: &str) -> Self { Button }
/// #     fn label(self, _label: &str) -> Self { self }
/// #     fn small(self) -> Self { self }
/// # }
/// # mod ui {
/// #     pub struct TaskCard;
/// #     impl TaskCard {
/// #         pub fn new(_id: usize) -> Self { TaskCard }
/// #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
/// #     }
/// # }
/// # struct View { count: usize, items: Vec<Item>, show: bool }
/// # impl View { fn render_child(&self) -> impl IntoElement { div() } }
/// # impl Render for View {
/// # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
/// # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
/// # let custom_color: Option<u32> = Some(0x3b82f6);
/// # let custom_width: Option<f32> = Some(120.0);
/// # let spacing = 4;
/// # let task = Task { id: 7, title: "Write docs".into() };
/// # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
/// # let _doc = {
/// rsx! {
///     <>
///         <div>{"First"}</div>
///         <div>{"Second"}</div>
///     </>
/// }
/// # };
/// # div()
/// # }
/// # }
/// # fn main() {
/// #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
/// #     let _ = view.render(&mut Window::default(), &mut Context::new());
/// # }
/// ```
///
/// ## Conditional styling
///
/// ```
/// # #![allow(unused)]
/// # use gpui::prelude::*;
/// # use gpui::*;
/// # use gpui_rsx::rsx;
/// # #[derive(Clone)]
/// # struct Item { id: usize, name: SharedString }
/// # struct Task { id: usize, title: SharedString }
/// # struct Button;
/// # impl Button {
/// #     fn new(_id: &str) -> Self { Button }
/// #     fn label(self, _label: &str) -> Self { self }
/// #     fn small(self) -> Self { self }
/// # }
/// # mod ui {
/// #     pub struct TaskCard;
/// #     impl TaskCard {
/// #         pub fn new(_id: usize) -> Self { TaskCard }
/// #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
/// #     }
/// # }
/// # struct View { count: usize, items: Vec<Item>, show: bool }
/// # impl View { fn render_child(&self) -> impl IntoElement { div() } }
/// # impl Render for View {
/// # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
/// # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
/// # let custom_color: Option<u32> = Some(0x3b82f6);
/// # let custom_width: Option<f32> = Some(120.0);
/// # let spacing = 4;
/// # let task = Task { id: 7, title: "Write docs".into() };
/// # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
/// # let _doc = {
/// rsx! {
///     <div
///         class="px-4 py-2 rounded-md"
///         when={(is_active, |el| el.bg(rgb(0x3b82f6)))}
///         whenSome={(custom_width, |el, w| el.w(px(w)))}
///     >
///         {"Content"}
///     </div>
/// }
/// # };
/// # div()
/// # }
/// # }
/// # fn main() {
/// #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
/// #     let _ = view.render(&mut Window::default(), &mut Context::new());
/// # }
/// ```
///
/// ## Styled defaults
///
/// ```
/// # #![allow(unused)]
/// # use gpui::prelude::*;
/// # use gpui::*;
/// # use gpui_rsx::rsx;
/// # #[derive(Clone)]
/// # struct Item { id: usize, name: SharedString }
/// # struct Task { id: usize, title: SharedString }
/// # struct Button;
/// # impl Button {
/// #     fn new(_id: &str) -> Self { Button }
/// #     fn label(self, _label: &str) -> Self { self }
/// #     fn small(self) -> Self { self }
/// # }
/// # mod ui {
/// #     pub struct TaskCard;
/// #     impl TaskCard {
/// #         pub fn new(_id: usize) -> Self { TaskCard }
/// #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
/// #     }
/// # }
/// # struct View { count: usize, items: Vec<Item>, show: bool }
/// # impl View { fn render_child(&self) -> impl IntoElement { div() } }
/// # impl Render for View {
/// # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
/// # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
/// # let custom_color: Option<u32> = Some(0x3b82f6);
/// # let custom_width: Option<f32> = Some(120.0);
/// # let spacing = 4;
/// # let task = Task { id: 7, title: "Write docs".into() };
/// # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
/// # let _doc = {
/// rsx! { <h1 styled>{"Title"}</h1> };
/// // → div().text_3xl().font_weight(FontWeight::BOLD).child("Title")
/// # };
/// # div()
/// # }
/// # }
/// # fn main() {
/// #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
/// #     let _ = view.render(&mut Window::default(), &mut Context::new());
/// # }
/// ```
///
/// # Notes
///
/// - **Static `class`**: parsed entirely at compile time, supports the documented Tailwind-like subset.
/// - **Dynamic `class={expr}`**: runtime matcher; full Tailwind color palette,
///   arbitrary colors, common layout/spacing/typography utilities, arbitrary
///   lengths, fraction sizing, and numeric values via prefix fallback. Unsupported classes
///   are reported in debug builds (or through `onUnknownClass`). Prefer `when` or static
///   strings for full coverage.
/// - **Auto ID**: elements with stateful event handlers receive a source-location-based
///   ID automatically. Provide an explicit `id` attribute for state-sensitive elements.
#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    expand_macro(input.into(), options(false)).into()
}

/// Transforms RSX into GPUI method chains and rejects unsupported static classes.
///
/// Static unknown classes emit a compile error (with a "did you mean" suggestion) instead of
/// being passed through to a GPUI method name. Dynamic unknown classes panic when evaluated
/// unless the element provides `onUnknownClass={|class| ...}`.
#[proc_macro]
pub fn rsx_strict(input: TokenStream) -> TokenStream {
    expand_macro(input.into(), options(true)).into()
}

/// Transforms RSX using the default permissive class handling.
///
/// This is equivalent to [`rsx!`], and exists as an explicit opt-in when codebases
/// also use [`rsx_strict!`].
#[proc_macro]
pub fn rsx_permissive(input: TokenStream) -> TokenStream {
    expand_macro(input.into(), options(false)).into()
}

/// [`rsx!`] with Tailwind's spacing scale: numeric spacing and sizing classes are multiplied
/// by 4px (`gap-4` → `px(16.0)`, `p-2.5` → `px(10.0)`). Arbitrary values such as
/// `gap-[14px]` are unaffected.
///
/// The same behavior can be enabled crate-wide with the `tailwind-scale` Cargo feature or
/// `spacing-scale = "tailwind"` in `rsx.toml`.
#[proc_macro]
pub fn rsx_tw(input: TokenStream) -> TokenStream {
    let options = MacroOptions {
        forced_scale: Some(SpacingScale::Tailwind),
        ..options(false)
    };
    expand_macro(input.into(), options).into()
}

/// Returns a string preview of the generated GPUI method chain.
///
/// This macro is intended for debugging generated code. It does not type-check the
/// generated GPUI expression because it returns a `&'static str`.
#[proc_macro]
pub fn rsx_expand(input: TokenStream) -> TokenStream {
    let preview = expand_macro(input.into(), options(false)).to_string();
    quote::quote! { #preview }.into()
}

/// Turns a struct with named fields into an RSX component with a typestate props builder.
///
/// - plain fields are **required** props; forgetting one is a compile error naming the prop
/// - `#[prop(default)]` / `#[prop(default = expr)]` fields are optional; `bool` props become
///   flags (`<Card elevated />`) plus a `fooIf={cond}` form
/// - one `#[children]` field (a `Vec<AnyElement>`) collects RSX children
///
/// The macro adds a function with the component's name (`Card()`), which is what
/// `<Card ...>` expands to, and implements `IntoElement` for the finished builder by
/// delegating to the component's own `IntoElement` (usually `#[derive(IntoElement)]` +
/// `RenderOnce`). `IntoElement` must be in scope (`use gpui::prelude::*`).
///
/// ```
/// use gpui::prelude::*;
/// use gpui::*;
/// use gpui_rsx::{component, rsx};
///
/// #[component]
/// #[derive(IntoElement)]
/// struct Card {
///     title: SharedString,
///     #[prop(default)]
///     elevated: bool,
///     #[children]
///     children: Vec<AnyElement>,
/// }
///
/// impl RenderOnce for Card {
///     fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
///         rsx! {
///             <div class="flex flex-col gap-2 p-4 rounded-lg" whenClass={(self.elevated, "shadow-md")}>
///                 <h2 class="text-lg font-semibold">{self.title}</h2>
///                 {...self.children}
///             </div>
///         }
///     }
/// }
///
/// let card = rsx! {
///     <Card title="Inbox" elevated>
///         <span>{"3 unread"}</span>
///     </Card>
/// };
/// # let _ = card.into_any_element();
/// ```
///
/// Leaving out `title` fails to compile with
/// ``missing required prop `title` on `<Card>` ``.
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_component(attr.into(), item.into()).into()
}
