---
title: Getting Started
description: Install GPUI-RSX and render a first GPUI view.
---

GPUI-RSX is a procedural macro for writing GPUI views with JSX-like syntax. Static markup expands to normal GPUI builder calls during compilation, so the generated UI remains ordinary Rust code.

## Install

Add GPUI from the Zed repository and GPUI-RSX to your application:

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit", "runtime_shaders", "wayland", "x11"] }
gpui-rsx = "0.6"
```

Applications should commit `Cargo.lock` so the exact GPUI git revision is pinned.

If you are upgrading from the crates.io `gpui = "0.2.2"` package, read the [migration guide](/gpui-rsx/guides/migration/#05x--cratesio-gpui-022--060--zed-git-gpui) before changing dependencies.

## Imports

```rust
use gpui::*;
use gpui::prelude::*;
use gpui_platform::application;
use gpui_rsx::rsx;
```

## First Element

The smallest RSX expression creates a GPUI `div()`:

```rust
rsx! {
    <div>{"Hello, GPUI!"}</div>
}
```

String children can be written as literals or Rust expressions:

```rust
rsx! {
    <div>
        "Count: "
        {self.count.to_string()}
    </div>
}
```

## Attributes

Flag attributes become zero-argument method calls:

```rust
rsx! {
    <div flex flex_col rounded_md />
}
```

Value attributes pass Rust expressions to builder methods:

```rust
rsx! {
    <div gap={px(16.0)} bg={rgb(0x3b82f6)} />
}
```

CamelCase aliases are mapped where GPUI uses snake_case:

```rust
rsx! {
    <button onClick={handler} textColor={rgb(0xffffff)}>
        {"Save"}
    </button>
}
```

## Class Attribute

Use `class` for stable Tailwind-like utility lists:

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4 bg-blue-500 text-white">
        {"Content"}
    </div>
}
```

Static class strings are parsed by the macro. Dynamic strings are handled by a generated runtime matcher and are useful for values assembled at runtime:

```rust
let row_class = if selected { "bg-blue-500 text-white" } else { "bg-gray-100" };

rsx! {
    <div class={row_class}>{label.as_str()}</div>
}
```

Prefer direct attributes when the value is already a Rust expression:

```rust
rsx! {
    <div class="flex" w={px(self.sidebar_width)} />
}
```

## Events

Event attributes accept normal GPUI listeners:

```rust
rsx! {
    <button
        class="px-4 py-2 bg-blue-500 text-white rounded-md cursor-pointer"
        onClick={cx.listener(|view, _, _window, cx| {
            view.count += 1;
            cx.notify();
        })}
    >
        {"Increment"}
    </button>
}
```

For repeated stateful elements inside `{for ...}`, add `key={...}` so each iteration gets a unique identity:

```rust
rsx! {
    <ul>
        {for task in &self.tasks {
            <li key={task.id} onClick={handler}>
                {task.title.as_str()}
            </li>
        }}
    </ul>
}
```

## Complete View

```rust
use gpui::*;
use gpui::prelude::*;
use gpui_platform::application;
use gpui_rsx::rsx;

struct HelloView;

impl Render for HelloView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="flex flex-col gap-3 p-4">
                <h1 class="text-xl font-bold">{"Hello GPUI-RSX"}</h1>
                <p class="text-gray-600">{"This markup expands to GPUI builder calls."}</p>
            </div>
        }
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_cx| HelloView)
        }).unwrap();
        cx.activate(true);
    });
}
```

## Check the Demo

The repository includes a demo crate that exercises real GPUI integration:

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
```

The demo lockfile pins GPUI and `gpui-component` so compatibility problems are reproducible.

## Next Steps

- Read [Syntax Reference](/gpui-rsx/usage/syntax/) for elements, attributes, children, and fragments.
- Read [Class Handling](/gpui-rsx/usage/class/) before relying on dynamic class strings.
- Read [IDs and Keys](/gpui-rsx/usage/ids/) before rendering interactive rows in loops.
