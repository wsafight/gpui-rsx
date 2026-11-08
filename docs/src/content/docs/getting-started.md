---
title: Getting Started
description: Install GPUI-RSX and render a first GPUI view.
---

Write a GPUI view with RSX tags and Rust expressions. The macro expands static markup into GPUI builder calls at compile time. This guide sets up the dependencies and opens a first window.

:::note[GPUI elements and HTML-like names]
Names such as `button` and `input` are aliases for GPUI `div()`, with no HTML input or form behavior. For a component library's controls, use explicit constructors such as `<Button base={Button::new("save")} label={"Save"} />`; see [Components](/gpui-rsx/usage/syntax/#components) for imports and construction rules.
:::

## Install

Add GPUI Kit, aliased as `gpui` for macro-generated paths, and GPUI-RSX:

```toml
[dependencies]
gpui = { package = "gpui-kit", version = "=0.9.0" }
gpui-rsx = "0.12.0"
```

Kit pins its matching `gpui-pre` snapshot and re-exports GPUI, components and assets. Commit `Cargo.lock` to reproduce the complete dependency graph.

If you are upgrading from Zed Git or the official `gpui = "0.2.2"` package, follow the current Kit section in the [migration guide](/gpui-rsx/guides/migration/).

## Complete View

Create `src/main.rs` with a complete view. Import both GPUI and its prelude so the generated builder methods are in scope.

```rust
use gpui::*;
use gpui::prelude::*;
use gpui_rsx::rsx;

struct HelloView;

impl Render for HelloView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="size-full flex flex-col gap-[12px] p-[24px]">
                <h1 class="text-xl font-bold">{"Hello GPUI-RSX"}</h1>
                <p class="text-gray-600">{"This markup expands to GPUI builder calls."}</p>
            </div>
        }
    }
}

fn main() {
    application().with_assets(assets::Assets).run(|cx: &mut App| {
        gpui::init(cx);
        gpui::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_cx| HelloView)
        }).unwrap();
        cx.activate(true);
    });
}
```

## Run the application

Run from your application directory:

```sh
cargo run
```

GPUI needs the native build dependencies for your platform. Use the [compatibility matrix](/gpui-rsx/compatibility/) to align GPUI Kit, Rust, and component versions.

To try an existing application first, choose one from the [runnable demos](/gpui-rsx/examples/demos/). Start with `counter` for events or `component` for native controls.

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

:::note[Pixel spacing by default]
`p-4` and `gap-4` mean 4px. For Tailwind's 4px units, use `rsx_tw!` or a [project configuration](/gpui-rsx/guides/theming/). Named attributes such as `gap_4` keep GPUI's native units, independently of the class scale.
:::

Static class strings are parsed by the macro. Dynamic strings are handled by a generated runtime matcher and are useful for values assembled at runtime:

```rust
let row_class = if selected { "bg-blue-500 text-white" } else { "bg-gray-100" };

rsx! {
    <div class={row_class}>{label.as_str()}</div>
}
```

Dynamic strings support a smaller set of classes: state variants and stateful scroll classes require static markup. Unknown dynamic classes are ignored in permissive release builds; strict mode panics unless an `onUnknownClass` handler is supplied. See [Class Handling](/gpui-rsx/usage/class/) for the support matrix.

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

## Check the Demo

The repository includes a demo crate that exercises real GPUI integration:

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
```

The demo lockfile pins Kit, its GPUI snapshot and `gpui-component` so compatibility problems are reproducible.

Run the complete multi-pane example directly:

```bash
cargo run --manifest-path demo/Cargo.toml --bin incident_console --locked
```

## Next Steps

- Use [Themes & Design Tokens](/gpui-rsx/guides/theming/) to share colors, spacing, and typography.
- Set up completion, diagnostics, and formatting with [CLI & Editors](/gpui-rsx/guides/tooling/).
- Read [Syntax Reference](/gpui-rsx/usage/syntax/) for elements, attributes, children, and fragments.
- Read [Class Handling](/gpui-rsx/usage/class/) before relying on dynamic class strings.
- Read [IDs and Keys](/gpui-rsx/usage/ids/) before rendering interactive rows in loops.
- Explore the [Incident Console](/gpui-rsx/examples/incident-console/) for a modular application example.
- Read [Compatibility](/gpui-rsx/compatibility/) when aligning GPUI, gpui-component, Rust, and the demo lockfile.
