# gpui-rsx

A procedural macro that brings JSX-like syntax to [GPUI], making UI code more concise
and readable while generating **zero-overhead** native GPUI method chains at compile time.

[GPUI]: https://www.gpui.rs/

## At a Glance

```
# #![allow(unused)]
# use gpui::prelude::*;
# use gpui::*;
# use gpui_rsx::rsx_tw;
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
// Before — verbose GPUI method chain
let before = div()
    .flex()
    .flex_col()
    .gap(px(16.0))
    .p(px(16.0))
    .bg(rgb(0x3b82f6))
    .child(div().text_xl().font_weight(FontWeight::BOLD).child("Hello GPUI"))
    .child(
        div()
            .cursor_pointer()
            .id("btn")
            .on_click(cx.listener(|_, _, _window, cx| cx.notify()))
            .child("Click me"),
    );

// After — concise RSX (~50% less code)
let after = rsx_tw! {
    <div class="flex flex-col gap-4 p-4 bg-blue-500">
        <h1 class="text-xl font-bold">{"Hello GPUI"}</h1>
        <button
            cursor_pointer
            onClick={cx.listener(|_, _, _window, cx| cx.notify())}
        >
            {"Click me"}
        </button>
    </div>
};
// Both use the same spacing, layout and colors.
# };
# div()
# }
# }
# fn main() {
#     let mut view = View { count: 0, items: vec![Item { id: 1, name: "a".into() }], show: true };
#     let _ = view.render(&mut Window::default(), &mut Context::new());
# }
```

`rsx_tw!` uses Tailwind's 4px spacing units, so `gap-4` and `p-4` are 16px.
The default `rsx!` uses pixel units unless `tailwind-scale` or project configuration
selects the Tailwind scale.

## Features

| Feature | Description |
|---------|-------------|
| HTML-like tags | `<div>`, `<span>`, `<button>`, … → all map to `div()` |
| Boolean attributes | `<div flex flex_col />` → `.flex().flex_col()` |
| Value attributes | `<div gap={px(4.0)} />` → `.gap(px(4.0))` |
| `class` (static) | Tailwind-like subset, parsed at compile time |
| `class` (dynamic) | Runtime expression with colors, sizing, spacing, and common utilities |
| Full color palette | 242 Tailwind colors + arbitrary hex/RGB/RGBA (`bg-[#ff0000]`) |
| Desktop sizing | `w-[280px]`, `w-[37.5%]`, `w-6/24`, `min-w-0` |
| Fragments | `<>...</>` returns `vec![...]`; `rsx_fragment!` returns `Vec<AnyElement>` for mixed root types |
| For-loop sugar | `{for item in iter { ... }}` |
| Spread | `{...iterator}` |
| Conditional | `when` / `whenSome` attributes |
| State classes | `hover:` / `focus:` / `active:` / `group-hover:` prefixes, `hoverClass`, `focusClass`, `activeClass` |
| Components | `#[component]` typestate props builders: `<Card title="…" elevated>` |
| Design tokens | `rsx.toml` colors/radius/spacing, gpui-component theme preset |
| Spacing scale | `rsx_tw!` / `tailwind-scale` for Tailwind's 4px units |
| Styled defaults | `<h1 styled>` injects sensible tag defaults |
| camelCase mapping | `onClick` → `.on_click()`, `fontSize` → `.text_size()` |
| Custom constructors | `base={Button::new("id")}` starts a component method chain |
| `key` (loop ID) | Composite auto-ID for stateful elements in loops |
