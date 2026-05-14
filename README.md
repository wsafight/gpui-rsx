# GPUI-RSX

English | [简体中文](./README_CN.md)

[![CI](https://github.com/wsafight/gpui-rsx/actions/workflows/ci.yml/badge.svg)](https://github.com/wsafight/gpui-rsx/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/wsafight/gpui-rsx/branch/main/graph/badge.svg)](https://codecov.io/gh/wsafight/gpui-rsx)
[![Crates.io](https://img.shields.io/crates/v/gpui-rsx.svg)](https://crates.io/crates/gpui-rsx)
[![Documentation](https://docs.rs/gpui-rsx/badge.svg)](https://docs.rs/gpui-rsx)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

A Rust procedural macro that provides JSX-like syntax for GPUI, making UI development more concise and intuitive.

## ✨ Features

- 🎨 **HTML-like Syntax** - React JSX-like development experience
- 🚀 **Zero Runtime Overhead** - Expands to native GPUI code at compile time
- 📦 **Lightweight** - Only depends on `syn`, `quote`, `proc-macro2`
- 🔧 **Flexible** - Supports expressions, conditional rendering, component composition
- 💡 **Type Safe** - Full compile-time type checking
- 🧩 **Fragment Support** - Return multiple root elements with `<>...</>`
- 🔁 **For-loop Sugar** - Iterate with `{for item in iter { ... }}`
- 🔑 **Loop-safe IDs** - `key={expr}` generates unique IDs per iteration; compile error on missing key
- 🎨 **Full Tailwind Colors** - 242 built-in colors + arbitrary hex/RGB/RGBA values
- 📐 **Desktop Layout Utilities** - Arbitrary lengths, percentages, and fraction sizing for panels and split views
- ⚡ **Dynamic Class** - Runtime class switching with colors, sizing, spacing, and numeric prefix fallback
- 🔍 **Diagnostics & Preview** - Strict/permissive macros, readable errors, and `rsx_expand!`

## 📚 Documentation

- **[Architecture Guide](./ARCHITECTURE.md)** - Detailed architecture documentation
  - Module organization and data flow
  - Code generation strategies
  - Design patterns and testing approach
  - Extension points and debugging guide
- **[Getting Started](./docs/getting-started.md)** - Step-by-step tutorial
- **[API Reference](./docs/api-reference.md)** - Complete API documentation
- **[Best Practices](./docs/best-practices.md)** - Recommended patterns
- **[Migration Guide](./docs/migration-guide.md)** - Upgrade instructions
- **[Troubleshooting](./docs/troubleshooting.md)** - Common issues and solutions

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gpui = "0.2"
gpui-rsx = "0.5"
```

## 🚀 Quick Start

### Get Started in 5 Minutes

```rust
use gpui::*;
use gpui_rsx::rsx;

struct CounterView {
    count: i32,
}

impl Render for CounterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="flex flex-col gap-4 p-4">
                <h1>{format!("Count: {}", self.count)}</h1>
                <div class="flex gap-2">
                    <button
                        bg={rgb(0x3b82f6)}
                        text_color={rgb(0xffffff)}
                        px_4
                        py_2
                        rounded_md
                        onClick={cx.listener(|view, _, _window, cx| {
                            view.count += 1;
                            cx.notify();
                        })}
                    >
                        {"Increment"}
                    </button>
                    <button
                        bg={rgb(0xef4444)}
                        text_color={rgb(0xffffff)}
                        px_4
                        py_2
                        rounded_md
                        onClick={cx.listener(|view, _, _window, cx| {
                            view.count -= 1;
                            cx.notify();
                        })}
                    >
                        {"Decrement"}
                    </button>
                </div>
            </div>
        }
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_cx| CounterView { count: 0 })
        });
    });
}
```

### Before & After

#### ❌ Traditional GPUI (Verbose)

```rust
fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        .child(
            div()
                .text_xl()
                .font_weight(FontWeight::BOLD)
                .child(format!("Count: {}", self.count))
        )
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    div()
                        .bg(rgb(0x3b82f6))
                        .text_color(rgb(0xffffff))
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .on_click(cx.listener(|view, _, _window, cx| {
                            view.count += 1;
                            cx.notify();
                        }))
                        .child("Increment")
                )
                .child(
                    div()
                        .bg(rgb(0xef4444))
                        .text_color(rgb(0xffffff))
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .on_click(cx.listener(|view, _, _window, cx| {
                            view.count -= 1;
                            cx.notify();
                        }))
                        .child("Decrement")
                )
        )
}
```

#### ✅ With GPUI-RSX (Concise)

See the Quick Start example above.

**Code Reduction: ~50%** ✨

## 📖 Syntax Guide

### 1. Basic Elements

```rust
rsx! {
    <div>{"Hello GPUI"}</div>
}
```

Expands to:
```rust
div().child("Hello GPUI")
```

### 2. Fragment (Multiple Root Elements)

When you need to return multiple elements without a wrapper:

```rust
rsx! {
    <>
        <div>{"First"}</div>
        <div>{"Second"}</div>
        <div>{"Third"}</div>
    </>
}
```

Expands to:
```rust
vec![
    div().child("First"),
    div().child("Second"),
    div().child("Third"),
]
```

### 3. Attributes

#### Boolean Attributes (Flags)

```rust
rsx! {
    <div flex flex_col />
}
```

Expands to:
```rust
div().flex().flex_col()
```

#### Value Attributes

```rust
rsx! {
    <div gap={px(16.0)} bg={rgb(0xffffff)} />
}
```

Expands to:
```rust
div().gap(px(16.0)).bg(rgb(0xffffff))
```

### 4. Class Attribute

The `class` attribute accepts a Tailwind-like string that expands into multiple GPUI method calls:

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4" />
}
```

Expands to:
```rust
div().flex().flex_col().gap(px(4.0)).p(px(4.0))
```

> **Note:** `class` accepts both static strings (compiled at build time) and dynamic expressions (parsed at runtime). GPUI-RSX implements a Tailwind-like subset, not the full Tailwind CSS engine. Static classes expand directly to GPUI builder calls; dynamic class expressions use a small runtime matcher. See [Dynamic Class](#10-dynamic-class-) for details.

#### Supported class patterns

**Layout:**
- `flex`, `flex-col`, `flex-row`, `flex-wrap`, `flex-1`, `flex-none`, `flex-auto`
- `min-w-0`, `min-h-0`, `items-center`, `items-start`, `items-end`, `items-stretch`
- `justify-center`, `justify-between`, `justify-around`, `justify-evenly`

**Spacing** (numeric values become `px(n)`):
- `gap-4` → `.gap(px(4.0))`
- `p-4`, `px-4`, `py-4`, `pt-4`, `pb-4`, `pl-4`, `pr-4`
- `m-4`, `mx-4`, `my-4`, `mt-4`, `mb-4`, `ml-4`, `mr-4`
- Arbitrary spacing: `gap-[14px]`, `gap-x-[0.75rem]`, `p-[18px]`, `mx-[1.25rem]`
- Percent spacing such as `gap-[10%]` intentionally errors because GPUI spacing uses definite lengths

**Sizing:**
- Numeric values keep project semantics: `w-64` → `.w(px(64.0))`, `h-32` → `.h(px(32.0))`
- `w-full`, `h-full`, `size-full`, `aspect-square`
- `w-px`, `h-px`, `w-auto`, `h-auto`, `w-1/2`, `h-1/3`, `size-1/2`
- Arbitrary sizing: `w-[280px]`, `w-[18rem]`, `w-[37.5%]`, `min-w-[280px]`, `max-w-[32rem]`
- Fraction sizing with arbitrary denominators: `w-6/24`, `min-w-1/3`, `size-3/4`

**Text:**
- `text-xs`, `text-sm`, `text-base`, `text-lg`, `text-xl`
- `text-2xl`, `text-3xl`
- `font-thin`, `font-extralight`, `font-light`, `font-normal`, `font-medium`, `font-semibold`, `font-bold`, `font-extrabold`, `font-black`
- `whitespace-normal`, `whitespace-nowrap`, `line-clamp-*`
- `text-ellipsis`, `text-ellipsis-start`, `truncate`, `no-underline`
- `text-decoration-solid`, `text-decoration-wavy`, `text-decoration-0/1/2/4/8`

**Alignment:**
- `content-normal`, `content-center`, `content-start`, `content-end`, `content-between`, `content-around`, `content-evenly`, `content-stretch`
- `self-start`, `self-end`, `self-flex-start`, `self-flex-end`, `self-center`, `self-baseline`, `self-stretch`

**Border:**
- `border` → `.border_1()`
- `border-2` → `.border_2()`, `border-4` → `.border_4()`
- `rounded-sm`, `rounded-md`, `rounded-lg`, `rounded-xl`, `rounded-full`, `rounded-none`

**Colors** (full Tailwind palette):
- `text-red-500` → `.text_color(rgb(0xef4444))`
- `bg-blue-600` → `.bg(rgb(0x2563eb))`
- `border-green-500` → `.border_color(rgb(0x22c55e))`
- Arbitrary colors: `bg-[#ff0000]`, `text-[#333]`, `border-[#11223344]`, `bg-[rgb(15,23,42)]`, `text-[rgba(15,23,42,0.8)]`

**Effects:**
- `shadow-none`, `shadow-2xs`, `shadow-xs`, `shadow-sm`, `shadow-md`, `shadow-lg`, `shadow-xl`, `shadow-2xl`
- `overflow-hidden`, `overflow-x-hidden`, `overflow-y-hidden`, `overflow-scroll`
- `cursor-pointer`, `cursor-default`, `cursor-text`, `cursor-move`, `cursor-grab`, `cursor-not-allowed`, resize cursor variants
- `debug-outline` enables GPUI debug borders in debug builds and is a no-op in release builds

**Grid placement:**
- `col-span-*`, `col-start-*`, `col-end-*`, `row-span-*`, `row-start-*`, `row-end-*`
- `col-span-full`, `col-start-auto`, `col-end-auto`, `row-span-full`, `row-start-auto`, `row-end-auto`

**Supported colors:** slate, gray, zinc, neutral, stone, red, orange, amber, yellow, lime, green, emerald, teal, cyan, sky, blue, indigo, violet, purple, fuchsia, pink, rose (shades 50-950) + white, black

### 5. Event Handling

```rust
rsx! {
    <button onClick={cx.listener(|view, _, _window, cx| {
        println!("clicked");
    })}>
        {"Click me"}
    </button>
}
```

Supported events (camelCase / snake_case):

| Event | Method |
|-------|--------|
| `onClick` / `on_click` | `.on_click(handler)` |
| `onMouseDown` / `on_mouse_down` | `.on_mouse_down(button, handler)` |
| `onMouseUp` / `on_mouse_up` | `.on_mouse_up(button, handler)` |
| `onMouseMove` / `on_mouse_move` | `.on_mouse_move(handler)` |
| `onMouseDownOut` / `on_mouse_down_out` | `.on_mouse_down_out(handler)` |
| `onMouseUpOut` / `on_mouse_up_out` | `.on_mouse_up_out(button, handler)` |
| `onAnyMouseDown` / `on_any_mouse_down` | `.on_any_mouse_down(handler)` |
| `onAnyMouseUp` / `on_any_mouse_up` | `.on_any_mouse_up(handler)` |
| `onKeyDown` / `on_key_down` | `.on_key_down(handler)` |
| `onKeyUp` / `on_key_up` | `.on_key_up(handler)` |
| `onModifiersChanged` / `on_modifiers_changed` | `.on_modifiers_changed(handler)` |
| `onHover` / `on_hover` | `.on_hover(handler)` |
| `onScrollWheel` / `on_scroll_wheel` | `.on_scroll_wheel(handler)` |
| `onDrag` / `on_drag` | `.on_drag(value, constructor)` |
| `onDragMove` / `on_drag_move` | `.on_drag_move(handler)` |
| `onDrop` / `on_drop` | `.on_drop(handler)` |
| `onAction` / `on_action` | `.on_action(handler)` |
| `onBoxedAction` / `on_boxed_action` | `.on_boxed_action(action, handler)` |
| `captureAnyMouseDown` / `capture_any_mouse_down` | `.capture_any_mouse_down(handler)` |
| `captureAnyMouseUp` / `capture_any_mouse_up` | `.capture_any_mouse_up(handler)` |
| `captureKeyDown` / `capture_key_down` | `.capture_key_down(handler)` |
| `captureKeyUp` / `capture_key_up` | `.capture_key_up(handler)` |
| `captureAction` / `capture_action` | `.capture_action(handler)` |

Methods with multiple GPUI parameters use tuple syntax in RSX:
`onMouseDown={(MouseButton::Left, handler)}` and `onDrag={(value, constructor)}`.

### 6. Nested Elements

```rust
rsx! {
    <div>
        <h1>{"Title"}</h1>
        <p>{"Description"}</p>
        <div>
            <button>{"Action 1"}</button>
            <button>{"Action 2"}</button>
        </div>
    </div>
}
```

### 7. Expressions

```rust
rsx! {
    <div>
        {format!("Count: {}", self.count)}
        {self.render_child_component()}
        {if self.show {
            rsx! { <span>{"Visible"}</span> }
        } else {
            rsx! { <span>{"Hidden"}</span> }
        }}
    </div>
}
```

### 8. List Rendering

#### Using iterators (traditional)

```rust
rsx! {
    <div>
        {self.items.iter().map(|item| {
            rsx! {
                <div key={item.id}>
                    {item.name.clone()}
                </div>
            }
        }).collect::<Vec<_>>()}
    </div>
}
```

#### Using for-loop syntax sugar

```rust
rsx! {
    <ul>
        {for item in &self.items {
            <li>{item.name.clone()}</li>
        }}
    </ul>
}
```

Expands to:
```rust
div().children((&self.items).into_iter().map(|item| {
    div().child(item.name.clone())
}))
```

#### Loop safety — `key` attribute

Elements with stateful attributes (`onClick`, `onHover`, `onDrag`, `tooltip`,
`focusable`, `overflowScroll`, `trackScroll`, or `overflow-scroll`) inside a for-loop **must**
provide `id` or `key`, otherwise the macro emits a compile error:

```rust
// ❌ compile error — all <li> would share the same auto ID
{for item in &self.items { <li onClick={handler}>{item}</li> }}

// ✅ key makes every ID unique per iteration
{for item in &self.items {
    <li key={item.id} onClick={handler}>{item.name.clone()}</li>
}}
// → div().id(format!("src/list.rs::__rsx_li_L42C8_{}", item.id)).on_click(handler)…
```

`key` is consumed by the macro and does **not** become a `.key()` method call.
It accepts any type implementing `Display`. On elements without stateful attributes,
`key` is silently ignored (no `.id()` is injected).

For-loops also support ranges and method calls:

```rust
rsx! {
    <div>
        {for i in 0..5 {
            <span>{i}</span>
        }}
    </div>
}
```

### 9. Spread Syntax

```rust
rsx! {
    <div>
        {...items.iter().map(|item| rsx! { <span>{item}</span> })}
    </div>
}
```

### 10. Dynamic Class

GPUI-RSX supports both static and dynamic `class` attributes.

#### Static Class (Compile-time — Recommended)

```rust
// ✅ Best performance - parsed at compile time, supports the documented subset
rsx! {
    <div class="flex gap-4 bg-blue-500">
        {"Static styles"}
    </div>
}
```

#### Dynamic Class (Runtime)

```rust
let classes = if is_active { "flex gap-4" } else { "block" };
rsx! {
    <div class={classes}>
        {"Dynamic styles"}
    </div>
}
```

> **Supported at runtime:** common layout/spacing/typography utilities, the full Tailwind color
> palette, arbitrary colors (e.g. `bg-[#ff0000]`, `text-[#f00]`, `bg-[rgba(15,23,42,0.8)]`),
> arbitrary spacing and sizing lengths (`w-[280px]`, `h-[50%]`, `gap-[14px]`, `mx-[1.25rem]`),
> fraction sizing (`w-6/24`), and numeric prefix fallback (e.g. `gap-7`, `p-5`, `opacity-33`).
> Truly unsupported classes (e.g. Tailwind variants or unknown utilities) are **silently ignored**
> in release builds and print a warning in debug builds.
>
> **Recommended alternatives** (in priority order):
> 1. **String literal** (best): `class="flex gap-4"` — compile-time, supports the documented subset
> 2. **Conditional literal**: `class={if active { "flex gap-4" } else { "block" }}` — still a literal
> 3. **Individual attributes**: `<div flex gap_4 />` — compile-time, type-checked
> 4. **`when` attribute**: `when={(cond, |el| el.flex())}` — compile-time, fully flexible
> 5. **Dynamic expression**: `class={expr}` — runtime parser, narrower coverage than static literals

**Common Patterns:**

```rust
// ✅ Conditional literal (compile-time, documented subset)
let button_class = if primary { "bg-blue-500 text-white" } else { "bg-gray-200 text-black" };

// ✅ when attribute (compile-time, fully flexible)
rsx! { <div when={(primary, |el| el.bg(rgb(0x3b82f6)).text_color(rgb(0xffffff)))} /> }

// ✅ Dynamic string with numeric prefix and arbitrary hex colors
let classes = format!("flex gap-{} bg-[#ff0000]", spacing);  // gap-7, gap-32, etc. all work
```

### 11. Macro Modes and Expansion Preview

`rsx!` is permissive by default: unsupported static class names are ignored when they cannot be parsed safely, while invalid arbitrary values emit compile errors. Use `rsx_strict!` to reject unsupported static classes:

```rust
use gpui_rsx::{rsx_expand, rsx_permissive, rsx_strict};

rsx_strict! { <div class="flex w-[280px]" /> }
rsx_permissive! { <div class="hover:bg-blue-500 flex" /> }

let preview = rsx_expand! {
    <div class="flex w-[280px] bg-[rgba(15,23,42,0.8)]" />
};
assert!(preview.contains("rgba"));
```

Strict dynamic classes panic when an unsupported runtime token is evaluated. `rsx_expand!` returns a string preview for debugging and does not type-check the generated GPUI expression.

Dynamic class capability summary:

| Capability | Static `class="..."` | Dynamic `class={expr}` |
|------------|----------------------|-------------------------|
| Layout, spacing, sizing | Supported | Supported subset |
| Colors and opacity | Supported | Supported |
| Arbitrary lengths/colors | Supported | Supported |
| Fraction sizing | Supported | Supported |
| Stateful scroll classes | Supported with auto ID | Not supported |
| Unknown Tailwind variants | Ignored in permissive, error in strict | Ignored in permissive, panic in strict |

### 12. Desktop Three-Column Layout

```rust
rsx! {
    <div class="flex h-full w-full bg-zinc-100">
        <nav class="w-[72px] min-w-[72px] bg-zinc-950" />

        <aside class="w-[280px] min-w-[280px] border-r border-zinc-200 bg-white">
            {"Projects"}
        </aside>

        <main class="flex-1 min-w-0 p-[18px]">
            {"Conversation, plans, diff, and results"}
        </main>

        <aside class="w-6/24 min-w-[320px] max-w-[460px] border-l border-zinc-200 bg-white">
            {"Execution trace"}
        </aside>
    </div>
}
```

`min-w-0` is important in desktop split layouts because it allows the center pane to shrink instead of pushing fixed sidebars out of the window.

### 13. Attribute Mapping Reference

camelCase attributes are automatically mapped to GPUI snake_case methods:

| RSX Attribute | GPUI Method |
|---------------|-------------|
| `opacity` | `.opacity()` |
| `visible` / `invisible` | `.visible()` / `.invisible()` |
| `width` / `height` | `.w()` / `.h()` |
| `minWidth` / `maxWidth` | `.min_w()` / `.max_w()` |
| `minHeight` / `maxHeight` | `.min_h()` / `.max_h()` |
| `gapX` / `gapY` | `.gap_x()` / `.gap_y()` |
| `flexBasis` | `.flex_basis()` |
| `flexGrow` / `flexShrink` (flags) | `.flex_grow()` / `.flex_shrink()` |
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

Attributes not in this table are passed through as-is (e.g., `bg={color}` → `.bg(color)`).

### 14. Conditional Styling with `when` and `whenSome`

#### when - Apply styles based on condition

```rust
rsx! {
    <div
        flex
        when={(is_active, |this| {
            this.bg(rgb(0x3b82f6))
                .text_color(rgb(0xffffff))
        })}
    >
        {"Button"}
    </div>
}
```

#### whenSome - Apply styles when Option has value

```rust
let custom_width: Option<f32> = Some(200.0);

rsx! {
    <div
        flex
        whenSome={(custom_width, |this, w| this.w(px(w)))}
    >
        {"Content"}
    </div>
}
```

#### whenClass - Apply static classes based on condition

```rust
rsx! {
    <div
        class="flex px-2"
        whenClass={(active, "bg-neutral-900 text-white")}
        whenClass={(!active, "text-neutral-600")}
    />
}
```

`whenClass` only accepts string literals. Stateful classes such as `overflow-scroll` are rejected; use `when={(cond, |el| el.overflow_scroll())}` when ID-sensitive GPUI methods are needed.

#### Multiple conditions

```rust
rsx! {
    <button
        class="px-4 py-2 rounded-md"
        when={(is_selected, |this| this.bg(rgb(0x3b82f6)))}
        when={(is_disabled, |this| this.bg(rgb(0xe5e7eb)))}
        whenSome={(custom_color, |this, color| this.bg(rgb(color)))}
    >
        {"Button"}
    </button>
}
```

### 15. Styled Flag (Default Tag Styles)

The `styled` flag injects sensible default styles based on the tag name:

```rust
rsx! {
    <h1 styled>{"Title"}</h1>
    // Expands to: div().text_3xl().font_weight(FontWeight::BOLD).child("Title")

    <button styled>{"Click"}</button>
    // Expands to: div().cursor_pointer().child("Click")
}
```

Default styles per tag:

| Tag | Default Styles |
|-----|---------------|
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

User attributes are applied after defaults and can override them.

## 🎯 Complete Example

### Todo App

```rust
use gpui::*;
use gpui_rsx::rsx;

struct TodoApp {
    todos: Vec<Todo>,
    input: String,
}

struct Todo {
    id: usize,
    text: String,
    completed: bool,
}

impl Render for TodoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="flex flex-col gap-4 p-4">
                <h1 class="text-2xl font-bold">
                    {"Todo List"}
                </h1>

                <div class="flex gap-2">
                    <input
                        placeholder="Add a todo..."
                        value={self.input.clone()}
                    />
                    <button
                        class="bg-blue-500 text-white px-4 py-2 rounded-md"
                        onClick={cx.listener(|view, _, _window, cx| {
                            view.add_todo();
                            cx.notify();
                        })}
                    >
                        {"Add"}
                    </button>
                </div>

                <div class="flex flex-col gap-2">
                    {for todo in self.todos.iter() {
                        <div
                            class="flex gap-2 items-center p-2 rounded-md"
                            bg={if todo.completed {
                                rgb(0xf3f4f6)
                            } else {
                                rgb(0xffffff)
                            }}
                        >
                            <span>{todo.text.clone()}</span>
                        </div>
                    }}
                </div>
            </div>
        }
    }
}

impl TodoApp {
    fn add_todo(&mut self) {
        if !self.input.is_empty() {
            self.todos.push(Todo {
                id: self.todos.len(),
                text: self.input.clone(),
                completed: false,
            });
            self.input.clear();
        }
    }
}
```

## 🔧 Advanced Usage

### Custom Components

```rust
fn render_card(&self, title: &str, content: &str) -> impl IntoElement {
    rsx! {
        <div class="rounded-lg shadow-md p-6">
            <h2 class="text-xl font-bold">
                {title}
            </h2>
            <p class="text-gray-600">
                {content}
            </p>
        </div>
    }
}

fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    rsx! {
        <div>
            {self.render_card("Title 1", "Content 1")}
            {self.render_card("Title 2", "Content 2")}
        </div>
    }
}
```

### Builder-backed Components

Use `base={expr}` when a component needs a custom constructor instead of the default `<Tag />` to `Tag()` expansion:

```rust
rsx! {
    <Button
        base={Button::new("continue")}
        label={"Continue"}
        small
        primary
    />
}
```

This expands to `Button::new("continue").label("Continue").small().primary()`. The `base` attribute is consumed by the macro and does not generate `.base(...)`.

Path-qualified component tags are supported, which is useful when components live in modules:

```rust
rsx! {
    <ui::TaskCard
        base={ui::TaskCard::new(task.id)}
        title={task.title.clone()}
    />
}
```

For `gpui-component`, keep imports explicit and use `base` for constructors that need IDs or custom arguments:

```rust
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::Sizable as _;

rsx! {
    <Button
        base={Button::new("continue")}
        label={"Continue"}
        small
        primary
    />
}
```

### Conditional Rendering

```rust
rsx! {
    <div>
        {if self.loading {
            rsx! { <div>{"Loading..."}</div> }
        } else if let Some(error) = &self.error {
            rsx! { <div class="text-red-500">{error.clone()}</div> }
        } else {
            rsx! { <div>{self.render_content()}</div> }
        }}
    </div>
}
```

### Dynamic Styling

```rust
fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let bg_color = if self.is_active {
        rgb(0x3b82f6)
    } else {
        rgb(0x6b7280)
    };

    rsx! {
        <div bg={bg_color} class="px-4 py-2 rounded-md">
            {"Button"}
        </div>
    }
}
```

## 📊 Performance

GPUI-RSX is a **compile-time macro** that expands static RSX into direct GPUI builder calls. Static markup has no parser at runtime; dynamic `class={expr}` intentionally uses a small runtime matcher.

| Metric | Traditional GPUI | GPUI-RSX |
|--------|------------------|----------|
| Code Size | 100 lines | 50 lines (-50%) |
| Runtime Performance | Baseline | Same |
| Type Safety | ✅ | ✅ |
| Compile-time Checking | ✅ | ✅ |

### v0.3.2 Fixes & Improvements
- **Fixed** `parse_single_class` panic on Tailwind variant syntax (`hover:bg-blue-500`): invalid
  class names are now silently skipped instead of calling `syn::Ident::new` with illegal characters
- **Added** 7 classes to dynamic match table: `rounded-none`, `rounded-xl`, `cursor-default`,
  `cursor-text`, `shadow-sm`, `shadow-md`, `shadow-lg`
- **Docs** styled defaults: added `li`, `p`, `label`, `form` entries; fixed `overflowX`/`overflowY`
  method names; removed unsupported `text-4xl`/`text-5xl`; updated dynamic class description

### v0.3.1 Fixes & Features
- **Fixed** `is_stateful_attr`: `hover`/`active`/`focus`/`group` are `Styled` trait methods and
  no longer trigger unnecessary `.id()` injection
- **Added** `key={expr}` attribute: composite auto ID for stateful elements in for-loops
- **Added** compile error when a stateful element in a for-loop has no `id` or `key`
- `key` on non-stateful elements is silently ignored (no unintended type change to `Stateful<Div>`)

### v0.3.0 Refactoring
- Eliminated ~60 duplicate method definitions in `tests/common/mod.rs` (823 → 456 lines)
- Simplified black/white color entry generation in `runtime.rs` (method names encoded in data)
- Extracted `is_directional_border()` helper in `class.rs` for clearer border logic

### v0.2.2 Optimizations

**Compile-time Performance:**
- `split_ascii_whitespace` replaces `split_whitespace` in class parsing
- Unified `text_` prefix handling (single `strip_prefix` call)
- Early fast-path for empty elements
- `Vec::with_capacity(attrs * 2 + children)` for class-heavy elements

**Runtime Performance:**
- `.children([...])` batching threshold lowered 3 → 2

**Binary Size:**
- Applications may opt into `panic = "abort"` in their own release profile to remove unwind tables

### v0.2.1 Optimizations

**Compile-time Performance:**
- O(1) color / attribute / spacing lookups via `match` (jump table, no linear scan)
- Single-pass attribute scanning in `generate_element`
- Thread-local cache for dynamic class match arms (generated once per process)

**Memory Allocation Reductions:**
- `parse_class_string` returns an iterator (no intermediate `Vec`)
- `generate_attr_methods` pushes directly into caller's buffer
- `Cow<str>` for class name transformations (zero-copy when no `-` present)
- `Vec::with_capacity` pre-allocation throughout

**Runtime Performance:**
- Zero-copy dynamic class strings via `AsRef<str>` (`&str` needs no allocation)

**Binary Size:**
- Dynamic class match table extracted to `#[inline(never)]` + LLVM ICF deduplication
- Multiple `class={expr}` in same component share one function body

## 🛠️ Development

### Build

```bash
cd gpui-rsx
cargo build
```

### Test

```bash
cargo test --test macro_tests
```

### Expand Macros (Debugging)

```bash
# Install cargo-expand
cargo install cargo-expand

# View expanded code
cargo expand --lib
```

## 💡 Best Practices

### 1. Component Splitting

Break complex UIs into small, reusable components:

```rust
// ✅ Recommended: Split into multiple methods
fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    rsx! {
        <div>
            {self.render_header()}
            {self.render_content()}
            {self.render_footer()}
        </div>
    }
}

fn render_header(&self) -> impl IntoElement {
    rsx! { <header>{"Header"}</header> }
}
```

### 2. Use Constants

Extract repeated styles as constants:

```rust
const PRIMARY_BG: Rgb = rgb(0x3b82f6);
const PRIMARY_TEXT: Rgb = rgb(0xffffff);

rsx! {
    <button bg={PRIMARY_BG} text_color={PRIMARY_TEXT}>
        {"Button"}
    </button>
}
```

### 3. Avoid Over-nesting

```rust
// ❌ Not recommended: Over-nested
rsx! {
    <div>
        <div>
            <div>
                <div>
                    {"Content"}
                </div>
            </div>
        </div>
    </div>
}

// ✅ Recommended: Flatten structure
rsx! {
    <div class="flex flex-col gap-4">
        {"Content"}
    </div>
}
```

## 🐛 FAQ

### Q1: How to use variables in RSX?

```rust
let title = "Hello";
rsx! {
    <div>{title}</div>
}
```

### Q2: How to handle Option types?

```rust
rsx! {
    <div>
        {if let Some(text) = &self.optional_text {
            rsx! { <span>{text.clone()}</span> }
        } else {
            rsx! { <span>{"No text"}</span> }
        }}
    </div>
}
```

### Q3: What does the expanded macro code look like?

Use `rsx_expand!` for a local string preview, or `cargo expand` to inspect the full crate:

```rust
let preview = gpui_rsx::rsx_expand! {
    <div class="flex w-[280px] bg-[rgba(15,23,42,0.8)]" />
};
```

```bash
cargo expand --lib
```

### Q4: Which elements are supported?

All GPUI-supported elements can be used, such as `div`, `button`, `input`, `span`, etc.

### Q5: Can I use dynamic class values?

Yes, but with an important limitation:

```rust
// ✅ Static literal (compile-time, documented subset — recommended)
rsx! { <div class="flex gap-4" /> }

// ✅ Individual attributes (compile-time, type-checked)
rsx! { <div bg={dynamic_color} flex /> }

// ✅ Conditional styling with `when` (compile-time, fully flexible)
rsx! { <div when={(is_active, |this| this.bg(rgb(0x3b82f6)))} /> }

// ✅ Dynamic expression with numeric prefix, arbitrary lengths, and arbitrary colors
let classes = if active { "flex gap-4" } else { "block" };
rsx! { <div class={classes} /> }
// Tailwind variants and unknown utilities are ignored in permissive mode.
```

**Tip:** When you need dynamic styling, prefer `when`/`whenSome` or individual value
attributes (`bg={color}`) — they are compile-time and support everything GPUI offers.

### Q6: How do I mix different element types in a Fragment?

`rsx!` fragments return `Vec<impl IntoElement>`, so all root items need the same concrete type. Prefer wrapping mixed children in a parent element:

```rust
rsx! {
    <div>
        <div />
        {Button::new("save")}
    </div>
}
```

If you really need a Fragment, erase the mixed items explicitly:

```rust
rsx! {
    <>
        {div().into_any_element()}
        {Button::new("save").into_any_element()}
    </>
}
```

## 🤝 Contributing

Contributions are welcome! Feel free to submit Issues or Pull Requests.

### Development Workflow

1. Fork the project
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push branch: `git push origin feature/amazing-feature`
5. Submit a Pull Request

### Code Standards

- Use `rustfmt` to format code
- Use `clippy` to check code quality
- Add tests for new features
- Update documentation

## 📝 License

MIT License

## 🙏 Acknowledgments

Inspired by:
- [Dioxus RSX](https://dioxuslabs.com/) - RSX syntax design
- [Yew html! macro](https://yew.rs/) - html! macro
- [React JSX](https://react.dev/) - JSX syntax
- [GPUI](https://www.gpui.rs/) - Underlying UI framework

---

**Make GPUI development more enjoyable!** 🎉
