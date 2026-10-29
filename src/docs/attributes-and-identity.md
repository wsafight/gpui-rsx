### Conditional Attributes: `when` and `whenSome`

Apply style transformations based on a runtime condition without leaving compile-time
safety:

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
rsx! {
    <button
        class="px-4 py-2 rounded-md"
        when={(is_selected, |el| el.bg(rgb(0x3b82f6)).text_color(rgb(0xffffff)))}
        when={(is_disabled, |el| el.opacity(0.5))}
        whenSome={(custom_color, |el, c| el.bg(rgb(c)))}
        whenClass={(is_focused, "border-blue-500 bg-blue-50")}
    >
        {"Button"}
    </button>
}
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

`whenClass` only accepts string literal classes and rejects stateful classes such as
`overflow-scroll`; use `when` for those explicit GPUI method calls.

State-style classes can be attached to GPUI's style-refinement hooks:

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
rsx! {
    <button
        class="px-4 py-2 rounded-md bg-blue-500 text-white"
        hoverClass="bg-blue-600"
        focusClass="border-blue-500"
        activeClass="opacity-75"
    />
}
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

These attributes accept only string literal classes. Element-only or stateful classes such as
`overflow-scroll` are rejected because GPUI passes a `StyleRefinement` into the closure.

### The `styled` Flag — Semantic Tag Defaults

Adding `styled` injects sensible default classes for the tag name, applied before
any user-provided attributes:

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
rsx! { <h1 styled>{"Title"}</h1> };
// → div().text_3xl().font_weight(FontWeight::BOLD).child("Title")

rsx! { <button styled onClick={handler}>{"OK"}</button> };
// → div().cursor_pointer().id("…").on_click(handler).child("OK")
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

| Tag | Default classes |
|-----|----------------|
| `h1` | `text-3xl font-bold` |
| `h2` | `text-2xl font-bold` |
| `h3` | `text-xl font-bold` |
| `h4` | `text-lg font-bold` |
| `h5` | `text-base font-bold` |
| `h6` | `text-sm font-bold` |
| `button`, `a` | `cursor-pointer` |
| `input`, `textarea` | `px-2 py-1` |
| `ul`, `ol` | `flex flex-col` |
| `li` | `flex items-center` |
| `p` | `text-base` |
| `label` | `text-sm` |
| `form` | `flex flex-col gap-4` |

### Attribute Mapping Reference

Most camelCase names are converted to snake_case GPUI methods. Attributes not in this
table are passed through unchanged (e.g., `bg={color}` → `.bg(color)`).

| RSX attribute | Generated GPUI code |
|---------------|---------------------|
| `opacity` | `.opacity()` |
| `visible` / `invisible` | `.visible()` / `.invisible()` |
| `width` / `height` | `.w()` / `.h()` |
| `minWidth` / `maxWidth` | `.min_w()` / `.max_w()` |
| `minHeight` / `maxHeight` | `.min_h()` / `.max_h()` |
| `gapX` / `gapY` | `.gap_x()` / `.gap_y()` |
| `flexBasis` | `.flex_basis()` |
| `flexGrow` / `flexShrink` (flags) | `.flex_grow_1()` / `.flex_shrink_1()` |
| `fontSize` | `.text_size()` |
| `lineHeight` | `.line_height()` |
| `fontWeight` | `.font_weight()` |
| `fontFamily` | `.font_family()` |
| `textAlign` | `.text_align()` |
| `textColor` | `.text_color()` |
| `backgroundColor` | `.bg()` |
| `borderColor` | `.border_color()` |
| `borderTop` / `borderBottom` | `.border_t(value)` / `.border_b(value)` |
| `borderLeft` / `borderRight` | `.border_l(value)` / `.border_r(value)` |
| `border_t` / `border_b` / `border_l` / `border_r` (flags) | `.border_t_1()` / `.border_b_1()` / `.border_l_1()` / `.border_r_1()` |
| `roundedTop` / `roundedBottom` | `.rounded_t()` / `.rounded_b()` |
| `roundedTopLeft` / `roundedTopRight` | `.rounded_tl()` / `.rounded_tr()` |
| `roundedBottomLeft` / `roundedBottomRight` | `.rounded_bl()` / `.rounded_br()` |
| `boxShadow` | `.shadow()` |
| `inset` | `.inset()` |

## Auto ID Injection

Elements that require a stateful identity (`onClick`, `onHover`, `onDrag`, `tooltip`,
`focusable`, `overflowScroll`, `trackScroll`, and static `overflow-scroll` classes)
automatically receive a deterministic `.id()` derived from the element's source
location. The ID is chosen by the following priority:

1. **Explicit `id`** — always wins, `key` is ignored.
2. **`key` present** (and element is stateful) — composite ID. Literal keys use
   `concat!(...)`; concrete integer values up to 64 bits use a named-integer tuple without
   string allocation. Other Display values use the borrowed formatting fallback.
3. **No `key`** (and element is stateful) — pure compile-time source-location ID:
   `concat!(file!(), "::", "__rsx_{tag}_L{line}C{col}")`
4. **Not stateful** — no `.id()` injected; `key` is silently ignored.

Component tags and `base` expressions receive an automatic ID only when their type
implements both `gpui::InteractiveElement` and `gpui::Element`. Functions returning
`Div` keep auto IDs, while `#[component]` builders and gpui-component's `Button` keep
their own event setters. Key expressions are still evaluated once at the call site.
Generic or opaque types must expose both bounds for automatic IDs. Explicit IDs
always call `.id(...)` directly and require that method to be available.
Loop checks remain syntax-based, so component
event props still require `key` or `id` inside `{for ...}`.

```text
// Format (no key): concat!(file!(), "::", "__rsx_{tag}_L{line}C{col}")
// Example:         "src/views/counter.rs::__rsx_button_L42C8"
//
// Format (literal key): concat!(file!(), "::__rsx_{tag}_L{line}C{col}_", "42")
// Format (dynamic integer key): (concat!(file!(), "::__rsx_{tag}_L{line}C{col}_"), key as u64)
// Other Display values: format!(concat!(file!(), "::__rsx_{tag}_L{line}C{col}_{}"), &key)
// Example:              "src/views/list.rs::__rsx_li_L55C12_42"
```

The source-location ID is stable across incremental rebuilds as long as the
element's position in the file does not change. For IDs that must survive
refactors, use the explicit `id` attribute.

> **Note on style attributes:** `hover`, `focus`, `group`, and `groupHover` are
> *`Styled` trait* methods and do **not** trigger auto ID injection. `active`,
> `activeClass`, and `groupActive` map to stateful GPUI hooks in the current target
> and do require an ID.

### Loop safety

Elements inside `{for ...}` loops share the same source location and would receive
identical auto IDs across iterations. **A compile error is emitted** in this case.
Add `key={expr}` (any `Display` type) to produce a unique ID per iteration:

```compile_fail
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
// ❌ compile error — all <li> would share the same auto ID
rsx! {
    <ul>
        {for item in &self.items { <li onClick={handler}>{item.name.clone()}</li> }}
    </ul>
}
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
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
// ✅ key produces a unique ID per iteration
rsx! {
    <ul>
        {for item in &self.items {
            <li key={item.id} onClick={handler}>{item.name.clone()}</li>
        }}
    </ul>
}
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

## Performance

- **Compile-time** — All class parsing, color lookup, and attribute mapping happen
  during macro expansion. The generated code is identical to hand-written GPUI.
- **Fast lookups** — Colors and attribute aliases use `match` tables; the generated GPUI
  method table is sorted and binary-searched.
- **Zero allocation** — Static classes generate no heap allocation. Dynamic classes
  use `AsRef<str>` (zero-copy for `&str` inputs).
- **Binary size** — Dynamic class helpers use `#[inline(never)]` + LLVM ICF to avoid
  code bloat when multiple `class={expr}` appear in the same component.

## Further Reading

- [Architecture guide (ARCHITECTURE.md)](https://github.com/wsafight/gpui-rsx/blob/main/ARCHITECTURE.md) —
  module design, data flow, extension points
- [Getting started](https://wsafight.github.io/gpui-rsx/getting-started/)
- [API reference](https://wsafight.github.io/gpui-rsx/reference/api/)
- [Best practices](https://wsafight.github.io/gpui-rsx/guides/best-practices/)
- [Changelog](https://github.com/wsafight/gpui-rsx/blob/main/CHANGELOG.md)
