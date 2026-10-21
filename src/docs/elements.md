## Syntax Reference

### Basic Elements

Any HTML tag maps to `div()`. Self-closing and pair tags are both valid:

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
rsx! { <div /> };                    // div()
rsx! { <span></span> };              // div()
rsx! { <button>{"OK"}</button> };    // div().child("OK")
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

### Fragment

Return multiple root elements without a wrapper. Expands to `vec![...]`:

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
    <>
        <div>{"First"}</div>
        <div>{"Second"}</div>
    </>
}
// → vec![div().child("First"), div().child("Second")]
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

### Boolean Attributes (Flags)

Bare attribute names become zero-argument method calls:

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
rsx! { <div flex flex_col items_center /> }
// → div().flex().flex_col().items_center()
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

### Value Attributes

`name={expr}` passes the expression as the method argument:

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
rsx! { <div gap={px(16.0)} bg={rgb(0x3b82f6)} opacity={0.8} /> }
// → div().gap(px(16.0)).bg(rgb(0x3b82f6)).opacity(0.8)
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

### Custom Constructor with `base`

`base={expr}` replaces the constructor inferred from the tag name, then the remaining
attributes continue as a normal method chain:

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
rsx! { <Button base={Button::new("save")} label={"Save"} small /> }
// → Button::new("save").label("Save").small()
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

Path-qualified tags are supported for module-scoped components:

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
rsx! { <ui::TaskCard base={ui::TaskCard::new(task.id)} title={task.title.clone()} /> }
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```
