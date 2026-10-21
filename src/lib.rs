//! # gpui-rsx
//!
//! A procedural macro that brings JSX-like syntax to [GPUI], making UI code more concise
//! and readable while generating **zero-overhead** native GPUI method chains at compile time.
//!
//! [GPUI]: https://www.gpui.rs/
//!
//! ## At a Glance
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! // Before — verbose GPUI method chain
//! let before = div()
//!     .flex()
//!     .flex_col()
//!     .gap(px(16.0))
//!     .p(px(16.0))
//!     .bg(rgb(0x3b82f6))
//!     .child(div().text_xl().font_weight(FontWeight::BOLD).child("Hello GPUI"))
//!     .child(
//!         div()
//!             .cursor_pointer()
//!             .id("btn")
//!             .on_click(cx.listener(|_, _, _window, cx| cx.notify()))
//!             .child("Click me"),
//!     );
//!
//! // After — concise RSX (~50% less code)
//! let after = rsx! {
//!     <div class="flex flex-col gap-4 p-4 bg-blue-500">
//!         <h1 class="text-xl font-bold">{"Hello GPUI"}</h1>
//!         <button
//!             cursor_pointer
//!             onClick={cx.listener(|_, _, _window, cx| cx.notify())}
//!         >
//!             {"Click me"}
//!         </button>
//!     </div>
//! };
//! // Both produce the same method chain.
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! The two snippets produce **identical** compiled output.
//!
//! ## Features
//!
//! | Feature | Description |
//! |---------|-------------|
//! | HTML-like tags | `<div>`, `<span>`, `<button>`, … → all map to `div()` |
//! | Boolean attributes | `<div flex flex_col />` → `.flex().flex_col()` |
//! | Value attributes | `<div gap={px(4.0)} />` → `.gap(px(4.0))` |
//! | `class` (static) | Tailwind-like subset, parsed at compile time |
//! | `class` (dynamic) | Runtime expression with colors, sizing, spacing, and common utilities |
//! | Full color palette | 242 Tailwind colors + arbitrary hex/RGB/RGBA (`bg-[#ff0000]`) |
//! | Desktop sizing | `w-[280px]`, `w-[37.5%]`, `w-6/24`, `min-w-0` |
//! | Fragments | `<>...</>` — returns `vec![...]` |
//! | For-loop sugar | `{for item in iter { ... }}` |
//! | Spread | `{...iterator}` |
//! | Conditional | `when` / `whenSome` attributes |
//! | State classes | `hover:` / `focus:` / `active:` / `group-hover:` prefixes, `hoverClass`, `focusClass`, `activeClass` |
//! | Components | `#[component]` typestate props builders: `<Card title="…" elevated>` |
//! | Design tokens | `rsx.toml` colors/radius/spacing, gpui-component theme preset |
//! | Spacing scale | `rsx_tw!` / `tailwind-scale` for Tailwind's 4px units |
//! | Styled defaults | `<h1 styled>` injects sensible tag defaults |
//! | camelCase mapping | `onClick` → `.on_click()`, `fontSize` → `.text_size()` |
//! | Custom constructors | `base={Button::new("id")}` starts a component method chain |
//! | `key` (loop ID) | Composite auto-ID for stateful elements in loops |
//!
//! ## Syntax Reference
//!
//! ### Basic Elements
//!
//! Any HTML tag maps to `div()`. Self-closing and pair tags are both valid:
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! { <div /> };                    // div()
//! rsx! { <span></span> };              // div()
//! rsx! { <button>{"OK"}</button> };    // div().child("OK")
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! ### Fragment
//!
//! Return multiple root elements without a wrapper. Expands to `vec![...]`:
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! {
//!     <>
//!         <div>{"First"}</div>
//!         <div>{"Second"}</div>
//!     </>
//! }
//! // → vec![div().child("First"), div().child("Second")]
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! ### Boolean Attributes (Flags)
//!
//! Bare attribute names become zero-argument method calls:
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! { <div flex flex_col items_center /> }
//! // → div().flex().flex_col().items_center()
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! ### Value Attributes
//!
//! `name={expr}` passes the expression as the method argument:
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! { <div gap={px(16.0)} bg={rgb(0x3b82f6)} opacity={0.8} /> }
//! // → div().gap(px(16.0)).bg(rgb(0x3b82f6)).opacity(0.8)
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! ### Custom Constructor with `base`
//!
//! `base={expr}` replaces the constructor inferred from the tag name, then the remaining
//! attributes continue as a normal method chain:
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! { <Button base={Button::new("save")} label={"Save"} small /> }
//! // → Button::new("save").label("Save").small()
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! Path-qualified tags are supported for module-scoped components:
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! { <ui::TaskCard base={ui::TaskCard::new(task.id)} title={task.title.clone()} /> }
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! ### The `class` Attribute — Static (Compile-time, Recommended)
//!
//! A Tailwind-inspired subset that expands to method calls at compile time.
//! It maps directly to GPUI APIs and is not a full Tailwind CSS engine.
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! { <div class="flex flex-col gap-4 p-4 bg-blue-500 text-white rounded-md" /> }
//! // → div().flex().flex_col().gap(px(4.0)).p(px(4.0)).bg(rgb(0x3b82f6))
//! //         .text_color(rgb(0xffffff)).rounded_md()
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! #### Supported class patterns
//!
//! **Layout:** `flex`, `flex-col`, `flex-row`, `flex-1`, `flex-wrap`, `flex-none`,
//! `min-w-0`, `min-h-0`, `block`, `grid`, `hidden`, `absolute`, `relative`
//!
//! **Alignment:** `items-center`, `items-start`, `items-end`, `justify-center`,
//! `justify-between`, `justify-start`, `justify-end`, `justify-around`,
//! `content-center`, `content-between`, …
//!
//! **Spacing** (numeric value → `px(n.0)`):
//! `gap-4` → `.gap(px(4.0))`, `p-4`, `px-4`, `py-4`, `pt-4`, `pb-4`, `pl-4`, `pr-4`,
//! `m-4`, `mx-4`, `my-4`, `mt-4`, `mb-4`, arbitrary lengths such as `gap-[14px]`
//! and `mx-[1.25rem]`
//!
//! **Sizing:** `w-full`, `h-full`, `size-full`, `w-64`, `h-32`, `w-[280px]`,
//! `w-[18rem]`, `w-[37.5%]`, `w-6/24`
//!
//! **Text:** `text-xs`, `text-sm`, `text-base`, `text-lg`, `text-xl`, `text-2xl`,
//! `text-3xl`, `font-thin` through `font-black`, `italic`, `underline`, `truncate`,
//! `text-left`, `text-center`
//!
//! **Border:** `border` → `.border_1()`, `border-2`, `rounded-sm`, `rounded-md`,
//! `rounded-lg`, `rounded-xl`, `rounded-full`, `rounded-none`
//!
//! **Colors (full Tailwind palette):**
//! `text-red-500` → `.text_color(rgb(0xef4444))`,
//! `bg-blue-600` → `.bg(rgb(0x2563eb))`,
//! `border-green-500` → `.border_color(rgb(0x22c55e))`
//!
//! Supported families: `slate`, `gray`, `zinc`, `neutral`, `stone`, `red`, `orange`,
//! `amber`, `yellow`, `lime`, `green`, `emerald`, `teal`, `cyan`, `sky`, `blue`,
//! `indigo`, `violet`, `purple`, `fuchsia`, `pink`, `rose` (shades 50–950) +
//! `black`, `white`
//!
//! **Arbitrary colors:**
//! `bg-[#ff0000]` → `.bg(rgb(0xff0000))`,
//! `text-[#f00]` → `.text_color(rgb(0xff0000))`,
//! `border-[#11223344]` → `.border_color(rgba(0x11223344))`,
//! `bg-[rgb(15,23,42)]`, `text-[rgba(15,23,42,0.8)]`
//!
//! Any other zero-argument GPUI style method is also accepted as a class: the supported
//! set is generated from the GPUI sources (`rounded-t-md` → `.rounded_t_md()`,
//! `border-x-2` → `.border_x_2()`, `mx-auto` → `.mx_auto()`).
//!
//! #### State variants
//!
//! Tailwind state prefixes compile to GPUI's style-refinement hooks:
//!
//! ```
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::{rsx, rsx_tw};
//! let _el = rsx! {
//!     <div class="group">
//!         <button class="bg-blue-500 hover:bg-blue-600 focus:border-2 active:opacity-75" />
//!         <span class="text-zinc-400 group-hover:text-white">{"label"}</span>
//!     </div>
//! };
//! // button → div().id(…).bg(…).hover(|s| s.bg(…)).focus(|s| s.border_2()).active(|s| s.opacity(0.75))
//! // span   → div().text_color(…).group_hover("", |s| s.text_color(…))
//! ```
//!
//! | Prefix | GPUI call | Needs an element ID |
//! |--------|-----------|---------------------|
//! | `hover:` | `.hover(…)` | no |
//! | `focus:` / `focus-visible:` | `.focus(…)` / `.focus_visible(…)` | no |
//! | `active:` | `.active(…)` | yes (auto-injected) |
//! | `group-hover:` / `group-hover/name:` | `.group_hover("" / "name", …)` | no |
//! | `group-active:` / `group-active/name:` | `.group_active(…)` | yes |
//!
//! `group` / `group/name` mark the ancestor (`.group("")` / `.group("name")`). Prefixes GPUI
//! cannot express — `dark:`, responsive `sm:`…`2xl:`, `disabled:` — produce a compile-time
//! warning (an error in [`rsx_strict!`]) explaining the alternative.
//!
//! #### Unknown classes
//!
//! No class is dropped silently:
//!
//! | Class | `rsx!` | `rsx_strict!` |
//! |-------|--------|---------------|
//! | supported | expanded | expanded |
//! | unknown identifier (`h-flex`) | passed through as `.h_flex()` (e.g. gpui-component) | error with suggestion |
//! | likely typo (`flex-colum`) | passed through **and** warning: did you mean `flex-col`? | error with suggestion |
//! | inexpressible (`md:flex`, `dark:bg-black`) | warning at the class literal | error |
//! | invalid value (`w-[abc]`, `opacity-150`) | error | error |
//!
//! Warnings use a `#[deprecated]` item, so they show up as regular rustc warnings on
//! stable and respect `#![deny(deprecated)]`. Tools can use the core crate's
//! `class_diagnostics` API to obtain the same diagnostics without compiling an application.
//!
//! #### Spacing scale (`rsx_tw!` / `tailwind-scale`)
//!
//! By default numeric spacing and sizing classes are pixels (`gap-4` → `px(4.0)`). Tailwind
//! uses 4px units; [`rsx_tw!`], the `tailwind-scale` Cargo feature, or
//! `spacing-scale = "tailwind"` in `rsx.toml` switch to that scale:
//!
//! ```
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::{rsx, rsx_tw};
//! let _el = rsx_tw! { <div class="gap-4 p-2.5 w-64 gap-x-[14px]" /> };
//! // → div().gap(px(16.0)).p(px(10.0)).w(px(256.0)).gap_x(px(14.0))
//! ```
//!
//! | Class | default | Tailwind scale |
//! |-------|---------|----------------|
//! | `gap-4` | `px(4.0)` | `px(16.0)` |
//! | `p-2.5` | `px(2.5)` | `px(10.0)` |
//! | `w-64` | `px(64.0)` | `px(256.0)` |
//! | `gap-[14px]` | `px(14.0)` | `px(14.0)` |
//!
//! Prefer `rsx.toml` over the Cargo feature in libraries: Cargo features are unified across
//! the dependency graph, `rsx.toml` applies to one crate only. To preserve an existing
//! numeric size when switching, use an arbitrary pixel value (`gap-4` → `gap-[4px]`)
//! or divide the numeric value by four (`gap-4` → `gap-1`).
//!
//! #### Design tokens and themes (`rsx.toml`)
//!
//! An `rsx.toml` next to `Cargo.toml` defines semantic classes:
//!
//! ```toml
//! [options]
//! preset = "gpui-component"   # bg-primary → cx.theme().colors.primary, text-muted-foreground, …
//!
//! [colors]
//! brand = "#2563eb"                              # bg-brand → .bg(rgb(0x2563eb))
//! surface = { expr = "cx.theme().background" }   # read at render time → live theme switching
//!
//! [radius]
//! card = "12px"                                  # rounded-card → .rounded(px(12.0))
//!
//! [spacing]
//! gutter = "18px"                                # p-gutter, gap-gutter, w-gutter, …
//! ```
//!
//! Expression tokens are evaluated where the RSX is written, so `cx` must be in scope.
//! Tokens are compile-time only: they are not available in dynamic `class={expr}` strings.
//!
//! ### The `class` Attribute — Dynamic (Runtime)
//!
//! When `class` receives an expression, a runtime matcher is generated.
//! Supported classes: common layout/spacing/typography utilities, the full Tailwind
//! color palette, arbitrary colors (`bg-[#ff0000]`, `text-[#f00]`,
//! `bg-[rgba(15,23,42,0.8)]`), arbitrary lengths (`w-[280px]`, `gap-[14px]`),
//! fraction sizing (`w-6/24`), and numeric values for spacing/sizing/opacity via prefix fallback.
//!
//! Classes the runtime matcher cannot apply (variants, theme tokens, unknown utilities) are
//! reported, not silently dropped: debug builds print each one once per call site
//! (or, with the `dynamic-class-report` feature, a single summary at process exit). In
//! [`rsx_strict!`] they panic, unless the element supplies its own handler:
//!
//! ```
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx_strict;
//! let classes = String::from("flex unknown-utility");
//! let _el = rsx_strict! {
//!     <div class={classes} onUnknownClass={|class: &str| eprintln!("ignored {class}")} />
//! };
//! ```
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! let active = true;
//! rsx! { <div class={if active { "flex gap-4" } else { "block" }} /> };
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! Prefer static strings or the `when` attribute instead:
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! // static literal — compile-time, documented subset
//! rsx! { <div class="flex gap-4" /> };
//!
//! // ✅ conditional literal — still static
//! let cls = if active { "flex gap-4" } else { "block" };
//! rsx! { <div class={cls} /> };
//!
//! // ✅ when attribute — compile-time, fully flexible
//! rsx! { <div when={(active, |el| el.flex().gap(px(4.0)))} /> };
//!
//! // Dynamic expression — runtime parser, narrower coverage than static literals
//! rsx! { <div class={format!("gap-{} bg-[#f00]", spacing)} /> };
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! ### Event Handling
//!
//! Event attributes (camelCase or snake_case) are mapped to GPUI listeners.
//! Elements with event handlers automatically receive a deterministic `.id()`.
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! {
//!     <button onClick={cx.listener(|view, _, _window, cx| {
//!         view.count += 1;
//!         cx.notify();
//!     })}>
//!         {"Increment"}
//!     </button>
//! }
//! // → div().id("src/main.rs::__rsx_button_L42C8").on_click(cx.listener(...)).child("Increment")
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! #### `key` — Unique IDs for stateful elements in loops
//!
//! `key={expr}` is a **macro-level** attribute consumed at compile time; it never
//! becomes a `.key()` method call on the GPUI element.
//!
//! **`key` only takes effect when the element already needs an `.id()`** (i.e. it
//! carries `onClick`, `onHover`, `onDrag`, `tooltip`, `focusable`, `overflowScroll`,
//! `trackScroll`, or another attribute that requires a stateful element).
//! On elements without any stateful attributes, `key` is silently ignored and no
//! `.id()` is injected — the element stays a plain `Div`.
//!
//! | Situation | Result |
//! |-----------|--------|
//! | stateful attrs + `key` | composite ID: auto-prefix + key (runtime) |
//! | stateful attrs, no `key` | pure source-location auto ID (compile-time) |
//! | no stateful attrs + `key` | **`key` ignored**, no `.id()` injected |
//! | explicit `id` | always used, `key` has no effect |
//!
//! **Loop safety:** Inside a `{for ...}` loop, every iteration shares the same source
//! location. The macro **emits a compile error** when a stateful element is found inside
//! a loop without an explicit `id` or `key`:
//!
//! ```compile_fail
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! // ❌ compile error — all <li> would share the same auto ID
//! rsx! {
//!     <ul>
//!         {for item in &self.items { <li onClick={handler}>{item.name.clone()}</li> }}
//!     </ul>
//! }
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! // ✅ key produces a unique ID per iteration
//! rsx! {
//!     <ul>
//!         {for item in &self.items {
//!             <li key={item.id} onClick={handler}>{item.name.clone()}</li>
//!         }}
//!     </ul>
//! }
//! // → div().id(format!("src/main.rs::__rsx_li_L42C8_{}", item.id)).on_click(handler)…
//! //   e.g. "src/main.rs::__rsx_li_L42C8_1", "src/main.rs::__rsx_li_L42C8_2", …
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! `key` accepts any type that implements `Display` (integers, `&str`, UUIDs, …).
//! For a fully stable custom ID that survives refactors, use the `id` attribute instead.
//!
//! | Attribute | GPUI method |
//! |-----------|-------------|
//! | `onClick` / `on_click` | `.on_click(h)` |
//! | `onMouseDown` / `on_mouse_down` | `.on_mouse_down(button, h)` |
//! | `onMouseUp` / `on_mouse_up` | `.on_mouse_up(button, h)` |
//! | `onMouseMove` / `on_mouse_move` | `.on_mouse_move(h)` |
//! | `onMouseDownOut` / `on_mouse_down_out` | `.on_mouse_down_out(h)` |
//! | `onMouseUpOut` / `on_mouse_up_out` | `.on_mouse_up_out(button, h)` |
//! | `onAnyMouseDown` / `on_any_mouse_down` | `.on_any_mouse_down(h)` |
//! | `onAnyMouseUp` / `on_any_mouse_up` | `.on_any_mouse_up(h)` |
//! | `onKeyDown` / `on_key_down` | `.on_key_down(h)` |
//! | `onKeyUp` / `on_key_up` | `.on_key_up(h)` |
//! | `onModifiersChanged` / `on_modifiers_changed` | `.on_modifiers_changed(h)` |
//! | `onHover` / `on_hover` | `.on_hover(h)` |
//! | `onScrollWheel` / `on_scroll_wheel` | `.on_scroll_wheel(h)` |
//! | `onDrag` / `on_drag` | `.on_drag(value, constructor)` |
//! | `onDragMove` / `on_drag_move` | `.on_drag_move(h)` |
//! | `onDrop` / `on_drop` | `.on_drop(h)` |
//! | `onAction` / `on_action` | `.on_action(h)` |
//! | `onBoxedAction` / `on_boxed_action` | `.on_boxed_action(action, h)` |
//! | `captureAnyMouseDown` / `capture_any_mouse_down` | `.capture_any_mouse_down(h)` |
//! | `captureAnyMouseUp` / `capture_any_mouse_up` | `.capture_any_mouse_up(h)` |
//! | `captureKeyDown` / `capture_key_down` | `.capture_key_down(h)` |
//! | `captureKeyUp` / `capture_key_up` | `.capture_key_up(h)` |
//! | `captureAction` / `capture_action` | `.capture_action(h)` |
//!
//! Multi-argument GPUI methods use tuple syntax in RSX, e.g.
//! `onMouseDown={(MouseButton::Left, handler)}`.
//!
//! ### Expressions and Children
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! {
//!     <div>
//!         {format!("Count: {}", self.count)}   // any expression
//!         {self.render_child()}                 // method returning IntoElement
//!         {if self.show {
//!             rsx! { <span>{"Visible"}</span> }
//!         } else {
//!             rsx! { <span>{"Hidden"}</span> }
//!         }}
//!     </div>
//! }
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! Two or more consecutive expressions are batched into a single `.children([...])`
//! call (stack-allocated array, zero heap allocation):
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! { <div>{"a"}{"b"}{"c"}</div> }
//! // → div().children(["a", "b", "c"])
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! ### For-loop Syntax Sugar
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! {
//!     <ul>
//!         {for item in &self.items {
//!             <li>{item.name.clone()}</li>
//!         }}
//!     </ul>
//! }
//! // → div().children((&self.items).into_iter().map(|item| {
//! //       div().child(item.name.clone())
//! //   }))
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! ### Spread Syntax
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! {
//!     <div>
//!         {...self.items.iter().map(|i| rsx! { <span>{i.name.clone()}</span> })}
//!     </div>
//! }
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! ### Conditional Attributes: `when` and `whenSome`
//!
//! Apply style transformations based on a runtime condition without leaving compile-time
//! safety:
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! {
//!     <button
//!         class="px-4 py-2 rounded-md"
//!         when={(is_selected, |el| el.bg(rgb(0x3b82f6)).text_color(rgb(0xffffff)))}
//!         when={(is_disabled, |el| el.opacity(0.5))}
//!         whenSome={(custom_color, |el, c| el.bg(rgb(c)))}
//!         whenClass={(is_focused, "border-blue-500 bg-blue-50")}
//!     >
//!         {"Button"}
//!     </button>
//! }
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! `whenClass` only accepts string literal classes and rejects stateful classes such as
//! `overflow-scroll`; use `when` for those explicit GPUI method calls.
//!
//! State-style classes can be attached to GPUI's style-refinement hooks:
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! {
//!     <button
//!         class="px-4 py-2 rounded-md bg-blue-500 text-white"
//!         hoverClass="bg-blue-600"
//!         focusClass="border-blue-500"
//!         activeClass="opacity-75"
//!     />
//! }
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! These attributes accept only string literal classes. Element-only or stateful classes such as
//! `overflow-scroll` are rejected because GPUI passes a `StyleRefinement` into the closure.
//!
//! ### The `styled` Flag — Semantic Tag Defaults
//!
//! Adding `styled` injects sensible default classes for the tag name, applied before
//! any user-provided attributes:
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! rsx! { <h1 styled>{"Title"}</h1> };
//! // → div().text_3xl().font_weight(FontWeight::BOLD).child("Title")
//!
//! rsx! { <button styled onClick={handler}>{"OK"}</button> };
//! // → div().cursor_pointer().id("…").on_click(handler).child("OK")
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! | Tag | Default classes |
//! |-----|----------------|
//! | `h1` | `text-3xl font-bold` |
//! | `h2` | `text-2xl font-bold` |
//! | `h3` | `text-xl font-bold` |
//! | `h4` | `text-lg font-bold` |
//! | `h5` | `text-base font-bold` |
//! | `h6` | `text-sm font-bold` |
//! | `button`, `a` | `cursor-pointer` |
//! | `input`, `textarea` | `px-2 py-1` |
//! | `ul`, `ol` | `flex flex-col` |
//! | `li` | `flex items-center` |
//! | `p` | `text-base` |
//! | `label` | `text-sm` |
//! | `form` | `flex flex-col gap-4` |
//!
//! ### Attribute Mapping Reference
//!
//! Most camelCase names are converted to snake_case GPUI methods. Attributes not in this
//! table are passed through unchanged (e.g., `bg={color}` → `.bg(color)`).
//!
//! | RSX attribute | Generated GPUI code |
//! |---------------|---------------------|
//! | `opacity` | `.opacity()` |
//! | `visible` / `invisible` | `.visible()` / `.invisible()` |
//! | `width` / `height` | `.w()` / `.h()` |
//! | `minWidth` / `maxWidth` | `.min_w()` / `.max_w()` |
//! | `minHeight` / `maxHeight` | `.min_h()` / `.max_h()` |
//! | `gapX` / `gapY` | `.gap_x()` / `.gap_y()` |
//! | `flexBasis` | `.flex_basis()` |
//! | `flexGrow` / `flexShrink` (flags) | `.flex_grow_1()` / `.flex_shrink_1()` |
//! | `fontSize` | `.text_size()` |
//! | `lineHeight` | `.line_height()` |
//! | `fontWeight` | `.font_weight()` |
//! | `fontFamily` | `.font_family()` |
//! | `textAlign` | `.text_align()` |
//! | `textColor` | `.text_color()` |
//! | `backgroundColor` | `.bg()` |
//! | `borderColor` | `.border_color()` |
//! | `borderTop` / `borderBottom` | `.border_t(value)` / `.border_b(value)` |
//! | `borderLeft` / `borderRight` | `.border_l(value)` / `.border_r(value)` |
//! | `border_t` / `border_b` / `border_l` / `border_r` (flags) | `.border_t_1()` / `.border_b_1()` / `.border_l_1()` / `.border_r_1()` |
//! | `roundedTop` / `roundedBottom` | `.rounded_t()` / `.rounded_b()` |
//! | `roundedTopLeft` / `roundedTopRight` | `.rounded_tl()` / `.rounded_tr()` |
//! | `roundedBottomLeft` / `roundedBottomRight` | `.rounded_bl()` / `.rounded_br()` |
//! | `boxShadow` | `.shadow()` |
//! | `inset` | `.inset()` |
//!
//! ## Auto ID Injection
//!
//! Elements that require a stateful identity (`onClick`, `onHover`, `onDrag`, `tooltip`,
//! `focusable`, `overflowScroll`, `trackScroll`, and static `overflow-scroll` classes)
//! automatically receive a deterministic `.id()` derived from the element's source
//! location. The ID is chosen by the following priority:
//!
//! 1. **Explicit `id`** — always wins, `key` is ignored.
//! 2. **`key` present** (and element is stateful) — composite ID. Literal keys use
//!    `concat!(...)`; dynamic keys use `format!(concat!(file!(), "::{prefix}_{}"), key_expr)`.
//! 3. **No `key`** (and element is stateful) — pure compile-time source-location ID:
//!    `concat!(file!(), "::", "__rsx_{tag}_L{line}C{col}")`
//! 4. **Not stateful** — no `.id()` injected; `key` is silently ignored.
//!
//! ```text
//! // Format (no key): concat!(file!(), "::", "__rsx_{tag}_L{line}C{col}")
//! // Example:         "src/views/counter.rs::__rsx_button_L42C8"
//! //
//! // Format (literal key): concat!(file!(), "::__rsx_{tag}_L{line}C{col}_", "42")
//! // Format (dynamic key): format!(concat!(file!(), "::__rsx_{tag}_L{line}C{col}_{}"), key)
//! // Example:              "src/views/list.rs::__rsx_li_L55C12_42"
//! ```
//!
//! The source-location ID is stable across incremental rebuilds as long as the
//! element's position in the file does not change. For IDs that must survive
//! refactors, use the explicit `id` attribute.
//!
//! > **Note on style attributes:** `hover`, `focus`, `group`, and `groupHover` are
//! > *`Styled` trait* methods and do **not** trigger auto ID injection. `active`,
//! > `activeClass`, and `groupActive` map to stateful GPUI hooks in the current target
//! > and do require an ID.
//!
//! ### Loop safety
//!
//! Elements inside `{for ...}` loops share the same source location and would receive
//! identical auto IDs across iterations. **A compile error is emitted** in this case.
//! Add `key={expr}` (any `Display` type) to produce a unique ID per iteration:
//!
//! ```compile_fail
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! // ❌ compile error — all <li> would share the same auto ID
//! rsx! {
//!     <ul>
//!         {for item in &self.items { <li onClick={handler}>{item.name.clone()}</li> }}
//!     </ul>
//! }
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! ```
//! # #![allow(unused)]
//! # use gpui::prelude::*;
//! # use gpui::*;
//! # use gpui_rsx::rsx;
//! # #[derive(Clone)]
//! # struct Item { id: usize, name: SharedString }
//! # struct Task { id: usize, title: SharedString }
//! # struct Button;
//! # impl Button {
//! #     fn new(_id: &str) -> Self { Button }
//! #     fn label(self, _label: &str) -> Self { self }
//! #     fn small(self) -> Self { self }
//! # }
//! # mod ui {
//! #     pub struct TaskCard;
//! #     impl TaskCard {
//! #         pub fn new(_id: usize) -> Self { TaskCard }
//! #         pub fn title(self, _title: gpui::SharedString) -> Self { self }
//! #     }
//! # }
//! # struct View { count: usize, items: Vec<Item>, show: bool }
//! # impl View { fn render_child(&self) -> impl IntoElement { div() } }
//! # impl Render for View {
//! # fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
//! # let custom_color: Option<u32> = Some(0x3b82f6);
//! # let custom_width: Option<f32> = Some(120.0);
//! # let spacing = 4;
//! # let task = Task { id: 7, title: "Write docs".into() };
//! # let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
//! # let _doc = {
//! // ✅ key produces a unique ID per iteration
//! rsx! {
//!     <ul>
//!         {for item in &self.items {
//!             <li key={item.id} onClick={handler}>{item.name.clone()}</li>
//!         }}
//!     </ul>
//! }
//! # };
//! # div()
//! # }
//! # }
//! # fn main() {
//! #     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
//! #     let _ = view.render(&mut Window::default(), &mut Context::new());
//! # }
//! ```
//!
//! ## Performance
//!
//! - **Compile-time** — All class parsing, color lookup, and attribute mapping happen
//!   during macro expansion. The generated code is identical to hand-written GPUI.
//! - **Fast lookups** — Colors and attribute aliases use `match` tables; the generated GPUI
//!   method table is sorted and binary-searched.
//! - **Zero allocation** — Static classes generate no heap allocation. Dynamic classes
//!   use `AsRef<str>` (zero-copy for `&str` inputs).
//! - **Binary size** — Dynamic class helpers use `#[inline(never)]` + LLVM ICF to avoid
//!   code bloat when multiple `class={expr}` appear in the same component.
//!
//! ## Further Reading
//!
//! - [Architecture guide (ARCHITECTURE.md)](https://github.com/wsafight/gpui-rsx/blob/main/ARCHITECTURE.md) —
//!   module design, data flow, extension points
//! - [Getting started](https://wsafight.github.io/gpui-rsx/getting-started/)
//! - [API reference](https://wsafight.github.io/gpui-rsx/reference/api/)
//! - [Best practices](https://wsafight.github.io/gpui-rsx/guides/best-practices/)
//! - [Changelog](https://github.com/wsafight/gpui-rsx/blob/main/CHANGELOG.md)

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
