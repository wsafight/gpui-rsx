### Event Handling

Event attributes (camelCase or snake_case) are mapped to GPUI listeners.
Elements with event handlers automatically receive a deterministic `.id()`.

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
    <button onClick={cx.listener(|view, _, _window, cx| {
        view.count += 1;
        cx.notify();
    })}>
        {"Increment"}
    </button>
}
// → div().id("src/main.rs::__rsx_button_L42C8").on_click(cx.listener(...)).child("Increment")
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

#### `key` — Unique IDs for stateful elements in loops

`key={expr}` is a **macro-level** attribute consumed at compile time; it never
becomes a `.key()` method call on the GPUI element.

**`key` only takes effect when the element already needs an `.id()`** (i.e. it
carries `onClick`, `onHover`, `onDrag`, `tooltip`, `focusable`, `overflowScroll`,
`trackScroll`, or another attribute that requires a stateful element).
On elements without any stateful attributes, `key` is silently ignored and no
`.id()` is injected — the element stays a plain `Div`.

| Situation | Result |
|-----------|--------|
| stateful attrs + `key` | composite ID: auto-prefix + key (runtime) |
| stateful attrs, no `key` | pure source-location auto ID (compile-time) |
| no stateful attrs + `key` | **`key` ignored**, no `.id()` injected |
| explicit `id` | always used, `key` has no effect |

**Loop safety:** Inside a `{for ...}` loop, every iteration shares the same source
location. The macro **emits a compile error** when a stateful element is found inside
a loop without an explicit `id` or `key`:

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
// → div().id(format!("src/main.rs::__rsx_li_L42C8_{}", item.id)).on_click(handler)…
//   e.g. "src/main.rs::__rsx_li_L42C8_1", "src/main.rs::__rsx_li_L42C8_2", …
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

`key` accepts any type that implements `Display` (integers, `&str`, UUIDs, …).
For a fully stable custom ID that survives refactors, use the `id` attribute instead.

| Attribute | GPUI method |
|-----------|-------------|
| `onClick` / `on_click` | `.on_click(h)` |
| `onMouseDown` / `on_mouse_down` | `.on_mouse_down(button, h)` |
| `onMouseUp` / `on_mouse_up` | `.on_mouse_up(button, h)` |
| `onMouseMove` / `on_mouse_move` | `.on_mouse_move(h)` |
| `onMouseDownOut` / `on_mouse_down_out` | `.on_mouse_down_out(h)` |
| `onMouseUpOut` / `on_mouse_up_out` | `.on_mouse_up_out(button, h)` |
| `onAnyMouseDown` / `on_any_mouse_down` | `.on_any_mouse_down(h)` |
| `onAnyMouseUp` / `on_any_mouse_up` | `.on_any_mouse_up(h)` |
| `onKeyDown` / `on_key_down` | `.on_key_down(h)` |
| `onKeyUp` / `on_key_up` | `.on_key_up(h)` |
| `onModifiersChanged` / `on_modifiers_changed` | `.on_modifiers_changed(h)` |
| `onHover` / `on_hover` | `.on_hover(h)` |
| `onScrollWheel` / `on_scroll_wheel` | `.on_scroll_wheel(h)` |
| `onDrag` / `on_drag` | `.on_drag(value, constructor)` |
| `onDragMove` / `on_drag_move` | `.on_drag_move(h)` |
| `onDrop` / `on_drop` | `.on_drop(h)` |
| `onAction` / `on_action` | `.on_action(h)` |
| `onBoxedAction` / `on_boxed_action` | `.on_boxed_action(action, h)` |
| `captureAnyMouseDown` / `capture_any_mouse_down` | `.capture_any_mouse_down(h)` |
| `captureAnyMouseUp` / `capture_any_mouse_up` | `.capture_any_mouse_up(h)` |
| `captureKeyDown` / `capture_key_down` | `.capture_key_down(h)` |
| `captureKeyUp` / `capture_key_up` | `.capture_key_up(h)` |
| `captureAction` / `capture_action` | `.capture_action(h)` |

Multi-argument GPUI methods use tuple syntax in RSX, e.g.
`onMouseDown={(MouseButton::Left, handler)}`.

For `InteractiveText`, `onClick={(ranges, listener)}` and
`on_click={(ranges, listener)}` expand to `.on_click(ranges, listener)`.
Ordinary `Div` listeners keep the single-argument form `onClick={listener}`.

### Expressions and Children

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
    <div>
        {format!("Count: {}", self.count)}   // any expression
        {self.render_child()}                 // method returning IntoElement
        {if self.show {
            rsx! { <span>{"Visible"}</span> }
        } else {
            rsx! { <span>{"Hidden"}</span> }
        }}
    </div>
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

Two or more consecutive expressions are batched into a single `.children([...])`
call (stack-allocated array, zero heap allocation):

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
rsx! { <div>{"a"}{"b"}{"c"}</div> }
// → div().children(["a", "b", "c"])
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

### For-loop Syntax Sugar

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
    <ul>
        {for item in &self.items {
            <li>{item.name.clone()}</li>
        }}
    </ul>
}
// → div().children((&self.items).into_iter().map(|item| {
//       div().child(item.name.clone())
//   }))
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

### Spread Syntax

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
    <div>
        {...self.items.iter().map(|i| rsx! { <span>{i.name.clone()}</span> })}
    </div>
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
