### The `class` Attribute — Static (Compile-time, Recommended)

A Tailwind-inspired subset that expands to method calls at compile time.
It maps directly to GPUI APIs and is not a full Tailwind CSS engine.

```
# #![allow(unused)]
# use gpui::prelude::*;
# use gpui::*;
# use gpui_rsx::rsx;
# #[derive(Clone)]
# struct Item { id: usize, name: SharedString }
# struct Task { id: usize, title: SharedString }
# struct Button;
# impl Button {
#     fn new(_id: &str) -> Self { Button }
#     fn label(self, _label: &str) -> Self { self }
#     fn small(self) -> Self { self }
# }
# mod ui {
#     pub struct TaskCard;
#     impl TaskCard {
#         pub fn new(_id: usize) -> Self { TaskCard }
#         pub fn title(self, _title: gpui::SharedString) -> Self { self }
#     }
# }
# struct View { count: usize, items: Vec<Item>, show: bool }
# impl View { fn render_child(&self) -> impl IntoElement { div() } }
# impl Render for View {
# fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
# let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
# let custom_color: Option<u32> = Some(0x3b82f6);
# let custom_width: Option<f32> = Some(120.0);
# let spacing = 4;
# let task = Task { id: 7, title: "Write docs".into() };
# let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
# let _doc = {
rsx! { <div class="flex flex-col gap-4 p-4 bg-blue-500 text-white rounded-md" /> }
// → div().flex().flex_col().gap(px(4.0)).p(px(4.0)).bg(rgb(0x3b82f6))
//         .text_color(rgb(0xffffff)).rounded_md()
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

#### Supported class patterns

**Layout:** `flex`, `flex-col`, `flex-row`, `flex-1`, `flex-wrap`, `flex-none`,
`min-w-0`, `min-h-0`, `block`, `grid`, `hidden`, `absolute`, `relative`

**Alignment:** `items-center`, `items-start`, `items-end`, `justify-center`,
`justify-between`, `justify-start`, `justify-end`, `justify-around`,
`content-center`, `content-between`, …

**Spacing** (numeric value → `px(n.0)`):
`gap-4` → `.gap(px(4.0))`, `p-4`, `px-4`, `py-4`, `pt-4`, `pb-4`, `pl-4`, `pr-4`,
`m-4`, `mx-4`, `my-4`, `mt-4`, `mb-4`, arbitrary lengths such as `gap-[14px]`
and `mx-[1.25rem]`

**Sizing:** `w-full`, `h-full`, `size-full`, `w-64`, `h-32`, `w-[280px]`,
`w-[18rem]`, `w-[37.5%]`, `w-6/24`

**Text:** `text-xs`, `text-sm`, `text-base`, `text-lg`, `text-xl`, `text-2xl`,
`text-3xl`, `font-thin` through `font-black`, `italic`, `underline`, `truncate`,
`text-left`, `text-center`

**Border:** `border` → `.border_1()`, `border-2`, `rounded-sm`, `rounded-md`,
`rounded-lg`, `rounded-xl`, `rounded-full`, `rounded-none`

**Colors (full Tailwind palette):**
`text-red-500` → `.text_color(rgb(0xef4444))`,
`bg-blue-600` → `.bg(rgb(0x2563eb))`,
`border-green-500` → `.border_color(rgb(0x22c55e))`

Supported families: `slate`, `gray`, `zinc`, `neutral`, `stone`, `red`, `orange`,
`amber`, `yellow`, `lime`, `green`, `emerald`, `teal`, `cyan`, `sky`, `blue`,
`indigo`, `violet`, `purple`, `fuchsia`, `pink`, `rose` (shades 50–950) +
`black`, `white`

**Arbitrary colors:**
`bg-[#ff0000]` → `.bg(rgb(0xff0000))`,
`text-[#f00]` → `.text_color(rgb(0xff0000))`,
`border-[#11223344]` → `.border_color(rgba(0x11223344))`,
`bg-[rgb(15,23,42)]`, `text-[rgba(15,23,42,0.8)]`

Any other zero-argument GPUI style method is also accepted as a class: the supported
set is generated from the GPUI sources (`rounded-t-md` → `.rounded_t_md()`,
`border-x-2` → `.border_x_2()`, `mx-auto` → `.mx_auto()`).

#### State variants

Tailwind state prefixes compile to GPUI's style-refinement hooks:

```
# use gpui::prelude::*;
# use gpui::*;
# use gpui_rsx::{rsx, rsx_tw};
let _el = rsx! {
    <div class="group">
        <button class="bg-blue-500 hover:bg-blue-600 focus:border-2 active:opacity-75" />
        <span class="text-zinc-400 group-hover:text-white">{"label"}</span>
    </div>
};
// button → div().id(…).bg(…).hover(|s| s.bg(…)).focus(|s| s.border_2()).active(|s| s.opacity(0.75))
// span   → div().text_color(…).group_hover("", |s| s.text_color(…))
```

| Prefix | GPUI call | Needs an element ID |
|--------|-----------|---------------------|
| `hover:` | `.hover(…)` | no |
| `focus:` / `focus-visible:` | `.focus(…)` / `.focus_visible(…)` | no |
| `active:` | `.active(…)` | yes (auto-injected) |
| `group-hover:` / `group-hover/name:` | `.group_hover("" / "name", …)` | no |
| `group-active:` / `group-active/name:` | `.group_active(…)` | yes |

`group` / `group/name` mark the ancestor (`.group("")` / `.group("name")`). Prefixes GPUI
cannot express — `dark:`, responsive `sm:`…`2xl:`, `disabled:` — produce a compile-time
warning (an error in [`rsx_strict!`]) explaining the alternative.

#### Unknown classes

No class is dropped silently:

| Class | `rsx!` | `rsx_strict!` |
|-------|--------|---------------|
| supported | expanded | expanded |
| unknown identifier (`h-flex`) | passed through as `.h_flex()` (e.g. gpui-component) | error with suggestion |
| likely typo (`flex-colum`) | passed through **and** warning: did you mean `flex-col`? | error with suggestion |
| inexpressible (`md:flex`, `dark:bg-black`) | warning at the class literal | error |
| invalid value (`w-[abc]`, `opacity-150`) | error | error |

Warnings use a `#[deprecated]` item, so they show up as regular rustc warnings on
stable and respect `#![deny(deprecated)]`. Tools can use the core crate's
`class_diagnostics` API to obtain the same diagnostics without compiling an application.

#### Spacing scale (`rsx_tw!` / `tailwind-scale`)

By default numeric spacing and sizing classes are pixels (`gap-4` → `px(4.0)`). Tailwind
uses 4px units; [`rsx_tw!`], the `tailwind-scale` Cargo feature, or
`spacing-scale = "tailwind"` in `rsx.toml` switch to that scale:

```
# use gpui::prelude::*;
# use gpui::*;
# use gpui_rsx::{rsx, rsx_tw};
let _el = rsx_tw! { <div class="gap-4 p-2.5 w-64 gap-x-[14px]" /> };
// → div().gap(px(16.0)).p(px(10.0)).w(px(256.0)).gap_x(px(14.0))
```

| Class | default | Tailwind scale |
|-------|---------|----------------|
| `gap-4` | `px(4.0)` | `px(16.0)` |
| `p-2.5` | `px(2.5)` | `px(10.0)` |
| `w-64` | `px(64.0)` | `px(256.0)` |
| `gap-[14px]` | `px(14.0)` | `px(14.0)` |

Prefer `rsx.toml` over the Cargo feature in libraries: Cargo features are unified across
the dependency graph, `rsx.toml` applies to one crate only. To preserve an existing
numeric size when switching, use an arbitrary pixel value (`gap-4` → `gap-[4px]`)
or divide the numeric value by four (`gap-4` → `gap-1`).

#### Design tokens and themes (`rsx.toml`)

An `rsx.toml` next to `Cargo.toml` defines semantic classes:

```toml
[options]
preset = "gpui-component"   # bg-primary → cx.theme().colors.primary, text-muted-foreground, …

[colors]
brand = "#2563eb"                              # bg-brand → .bg(rgb(0x2563eb))
surface = { expr = "cx.theme().background" }   # read at render time → live theme switching

[radius]
card = "12px"                                  # rounded-card → .rounded(px(12.0))

[spacing]
gutter = "18px"                                # p-gutter, gap-gutter, w-gutter, …
```

Expression tokens are evaluated where the RSX is written, so `cx` must be in scope.
Tokens are compile-time only: they are not available in dynamic `class={expr}` strings.

### The `class` Attribute — Dynamic (Runtime)

When `class` receives an expression, a runtime matcher is generated.
Supported classes: common layout/spacing/typography utilities, the full Tailwind
color palette, arbitrary colors (`bg-[#ff0000]`, `text-[#f00]`,
`bg-[rgba(15,23,42,0.8)]`), arbitrary lengths (`w-[280px]`, `gap-[14px]`),
fraction sizing (`w-6/24`), and numeric values for spacing/sizing/opacity via prefix fallback.

Classes the runtime matcher cannot apply (variants, theme tokens, unknown utilities) are
reported, not silently dropped: debug builds print each one once per call site
(or, with the `dynamic-class-report` feature, a single summary at process exit). In
[`rsx_strict!`] they panic, unless the element supplies its own handler:

```
# use gpui::prelude::*;
# use gpui::*;
# use gpui_rsx::rsx_strict;
let classes = String::from("flex unknown-utility");
let _el = rsx_strict! {
    <div class={classes} onUnknownClass={|class: &str| eprintln!("ignored {class}")} />
};
```

```
# #![allow(unused)]
# use gpui::prelude::*;
# use gpui::*;
# use gpui_rsx::rsx;
# #[derive(Clone)]
# struct Item { id: usize, name: SharedString }
# struct Task { id: usize, title: SharedString }
# struct Button;
# impl Button {
#     fn new(_id: &str) -> Self { Button }
#     fn label(self, _label: &str) -> Self { self }
#     fn small(self) -> Self { self }
# }
# mod ui {
#     pub struct TaskCard;
#     impl TaskCard {
#         pub fn new(_id: usize) -> Self { TaskCard }
#         pub fn title(self, _title: gpui::SharedString) -> Self { self }
#     }
# }
# struct View { count: usize, items: Vec<Item>, show: bool }
# impl View { fn render_child(&self) -> impl IntoElement { div() } }
# impl Render for View {
# fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
# let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
# let custom_color: Option<u32> = Some(0x3b82f6);
# let custom_width: Option<f32> = Some(120.0);
# let spacing = 4;
# let task = Task { id: 7, title: "Write docs".into() };
# let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
# let _doc = {
let active = true;
rsx! { <div class={if active { "flex gap-4" } else { "block" }} /> };
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

Prefer static strings or the `when` attribute instead:

```
# #![allow(unused)]
# use gpui::prelude::*;
# use gpui::*;
# use gpui_rsx::rsx;
# #[derive(Clone)]
# struct Item { id: usize, name: SharedString }
# struct Task { id: usize, title: SharedString }
# struct Button;
# impl Button {
#     fn new(_id: &str) -> Self { Button }
#     fn label(self, _label: &str) -> Self { self }
#     fn small(self) -> Self { self }
# }
# mod ui {
#     pub struct TaskCard;
#     impl TaskCard {
#         pub fn new(_id: usize) -> Self { TaskCard }
#         pub fn title(self, _title: gpui::SharedString) -> Self { self }
#     }
# }
# struct View { count: usize, items: Vec<Item>, show: bool }
# impl View { fn render_child(&self) -> impl IntoElement { div() } }
# impl Render for View {
# fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
# let (active, is_active, is_selected, is_disabled, is_focused) = (true, true, false, false, true);
# let custom_color: Option<u32> = Some(0x3b82f6);
# let custom_width: Option<f32> = Some(120.0);
# let spacing = 4;
# let task = Task { id: 7, title: "Write docs".into() };
# let handler = |_: &ClickEvent, _: &mut Window, _: &mut App| {};
# let _doc = {
// static literal — compile-time, documented subset
rsx! { <div class="flex gap-4" /> };

// ✅ conditional literal — still static
let cls = if active { "flex gap-4" } else { "block" };
rsx! { <div class={cls} /> };

// ✅ when attribute — compile-time, fully flexible
rsx! { <div when={(active, |el| el.flex().gap(px(4.0)))} /> };

// Dynamic expression — runtime parser, narrower coverage than static literals
rsx! { <div class={format!("gap-{} bg-[#f00]", spacing)} /> };
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```
