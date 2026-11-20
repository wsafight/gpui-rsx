#![doc = include_str!("docs/overview.md")]
#![doc = include_str!("docs/elements.md")]
#![doc = include_str!("docs/classes.md")]
#![doc = include_str!("docs/events-and-children.md")]
#![doc = include_str!("docs/attributes-and-identity.md")]
#![doc = include_str!("docs/listeners.md")]
#![doc = include_str!("docs/state-updates.md")]
use gpui_rsx_core::{MacroOptions, SpacingScale, Theme, expand_macro};
use proc_macro::TokenStream;

fn options(strict: bool) -> MacroOptions {
    MacroOptions::new()
        .with_strict(strict)
        .with_default_scale(if cfg!(feature = "tailwind-scale") {
            SpacingScale::Tailwind
        } else {
            SpacingScale::Px
        })
        .with_dynamic_report(cfg!(feature = "dynamic-class-report"))
}

/// Expands with the configured GPUI path. `item` is `Some(tokens)` for attribute macros: it is
/// re-emitted after a configuration error so the error is not buried under cascading ones.
fn configured_gpui(
    item: Option<proc_macro2::TokenStream>,
    expand: impl FnOnce(&syn::Path) -> proc_macro2::TokenStream,
) -> TokenStream {
    let is_item = item.is_some();
    let theme = match std::env::var_os("CARGO_MANIFEST_DIR") {
        Some(directory) => match Theme::load_from_dir(std::path::Path::new(&directory)) {
            Ok(theme) => theme,
            Err(message) => {
                return match item {
                    Some(item) => quote::quote!(compile_error!(#message); #item),
                    None => quote::quote!(compile_error!(#message)),
                }
                .into();
            }
        },
        None => None,
    };
    let gpui = theme
        .as_deref()
        .map_or_else(|| syn::parse_quote!(::gpui), Theme::gpui_syn_path);
    let tracker = theme
        .as_deref()
        .and_then(gpui_rsx_core::theme::rebuild_tracker);
    let code = expand(&gpui);
    if is_item {
        quote::quote!(#tracker #code).into()
    } else {
        quote::quote!({ #tracker #code }).into()
    }
}

/// Generates shared dynamic class helpers once, at the crate root.
///
/// Enable them with `[options] dynamic-runtime = "rsx_runtime"` in `rsx.toml`.
/// With no configuration, RSX continues to inline its runtime helper as before.
/// Libraries expose this module to their integration tests, examples and binaries.
/// Binary-only packages invoke this macro in each target using dynamic classes.
///
/// ```
/// gpui_rsx::rsx_runtime!();
/// ```
///
/// Pass a crate path when GPUI is imported under another name:
/// `gpui_rsx::rsx_runtime!(my_gpui);`.
#[proc_macro]
pub fn rsx_runtime(input: TokenStream) -> TokenStream {
    let explicit_gpui = if input.is_empty() {
        None
    } else {
        Some(syn::parse_macro_input!(input as syn::Path))
    };
    let theme = match std::env::var_os("CARGO_MANIFEST_DIR") {
        Some(directory) => match Theme::load_from_dir(std::path::Path::new(&directory)) {
            Ok(theme) => theme,
            Err(message) => return quote::quote!(compile_error!(#message);).into(),
        },
        None => None,
    };
    let gpui = explicit_gpui.unwrap_or_else(|| {
        theme
            .as_deref()
            .map_or_else(|| syn::parse_quote!(::gpui), Theme::gpui_syn_path)
    });
    let tracker = theme
        .as_deref()
        .and_then(gpui_rsx_core::theme::rebuild_tracker);
    let code = gpui_rsx_core::generate_runtime_module_with_theme(&gpui, theme.as_deref());
    quote::quote!(#tracker #code).into()
}

/// Builds a GPUI view listener from a view context and a four-argument handler.
///
/// `rsx_listener!(cx, handler)` delegates directly to `cx.listener(handler)`.
/// Add the explicit `notify` marker to notify the view after the handler returns:
/// `rsx_listener!(cx, notify, handler)`. The handler still receives the view,
/// event, window, and view context in that order.
///
/// `notify` is only a marker when a handler follows it; `rsx_listener!(cx, notify)` passes a
/// handler that is itself named `notify`.
#[proc_macro]
pub fn rsx_listener(input: TokenStream) -> TokenStream {
    configured_gpui(None, |gpui| {
        gpui_rsx_core::expand_listener_with_path(input.into(), gpui)
    })
}

/// Runs `rsx_update!(view, cx, update)` and notifies after normal return.
///
/// The `FnOnce` update receives the mutable view and its context. Several field
/// changes can share one notification. The update's result is returned unchanged.
/// Assignments made outside this macro retain GPUI's notification rules.
#[proc_macro]
pub fn rsx_update(input: TokenStream) -> TokenStream {
    configured_gpui(None, |gpui| {
        gpui_rsx_core::expand_update_with_path(input.into(), gpui)
    })
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
///   Directly inline `if` / `match` expressions with string-literal leaves are also expanded
///   statically; assigning the condition to a local variable selects the dynamic path instead.
/// - **Dynamic `class={expr}`**: runtime matcher for expressions that cannot be statically expanded;
///   full Tailwind color palette,
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

/// Returns `Vec<gpui::AnyElement>`, allowing fragments with different concrete element types.
///
/// Uses the same class handling, spacing scale and project configuration as [`rsx!`].
/// A single root returns a one-element vector; an empty fragment returns an empty vector.
///
/// ```
/// use gpui::{div, prelude::*};
/// use gpui_rsx::{rsx, rsx_fragment};
///
/// let parts = rsx_fragment! {
///     <>
///         <div />
///         <div id="stateful" />
///     </>
/// };
/// let view = rsx! { <div>{...parts}</div> };
/// ```
#[proc_macro]
pub fn rsx_fragment(input: TokenStream) -> TokenStream {
    expand_macro(input.into(), options(false).with_erased_roots(true)).into()
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
    let options = options(false).with_forced_scale(Some(SpacingScale::Tailwind));
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

#[doc = include_str!("docs/components.md")]
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: proc_macro2::TokenStream = item.into();
    configured_gpui(Some(item.clone()), |gpui| {
        gpui_rsx_core::expand_component_with_path(attr.into(), item, gpui)
    })
}
