---
title: Best Practices
description: Organize views, keep state updates explicit, and choose IDs and classes that survive refactoring.
---

Use these patterns when a view grows beyond a short render method. The [Incident Console](/gpui-rsx/examples/incident-console/) shows how the pieces fit together in a runnable application.

## Structure Views by Workflow

Extract reusable UI into methods or small builders once a render block stops being easy to scan:

```rust
impl MyView {
    fn render_header(&self) -> impl IntoElement {
        rsx! {
            <header class="flex items-center justify-between p-4 border-b border-gray-200">
                <h1 styled>{self.title.as_str()}</h1>
                {self.render_actions()}
            </header>
        }
    }

    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="flex flex-col h-full">
                {self.render_header()}
                <main class="flex-1 min-h-0">{self.render_content()}</main>
            </div>
        }
    }
}
```

Prefer extraction around real concepts such as header, sidebar, row, editor, and status panel. Avoid splitting every small element into a separate method.

## Keep Types Predictable

GPUI-RSX generates normal Rust expressions, so Rust type rules still apply. Return `impl IntoElement` from render helpers:

```rust
fn render_item(&self, item: &Item) -> impl IntoElement {
    rsx! {
        <li class="flex items-center gap-2">
            {item.name.as_str()}
        </li>
    }
}
```

When branch types become awkward, wrap the branch output in a common parent:

```rust
fn render_body(&self) -> impl IntoElement {
    rsx! {
        <div>
            {match &self.state {
                State::Loading => rsx! { <span>{"Loading"}</span> },
                State::Ready(data) => rsx! { <section>{data.title.as_str()}</section> },
                State::Error(error) => rsx! { <p class="text-red-600">{error.as_str()}</p> },
            }}
        </div>
    }
}
```

## Prefer References in Lists

Avoid unnecessary cloning in render loops:

```rust
rsx! {
    <ul>
        {for item in &self.items {
            <li>{item.name.as_str()}</li>
        }}
    </ul>
}
```

If an interactive row is repeated, add a `key`:

```rust
rsx! {
    <ul>
        {for item in &self.items {
            <li key={item.id} onClick={cx.listener(Self::select_item)}>
                {item.name.as_str()}
            </li>
        }}
    </ul>
}
```

When a list can grow past a few hundred uniform-height rows, switch from `{for ...}` to `<uniform_list>` so GPUI only builds the visible slice on each frame:

```rust
rsx! {
    <uniform_list
        id="incident-rows"
        count={self.rows.len()}
        class="flex-1 size-full"
        render={cx.processor(|view, range, _window, cx| {
            range.map(|ix| view.render_row(ix, cx)).collect()
        })}
    />
}
```

## Style by Stability

Pick the simplest compile-time styling form that matches how dynamic a value is:

| Situation | Recommended form | Cost |
| --- | --- | --- |
| Fixed layout and colors | `class="flex flex-col gap-4 p-4"` | Compile-time |
| Toggling known utility groups | `class={["flex p-8", selected && "bg-blue-500 text-white"]}` or `whenClass` | Compile-time |
| Computed Rust number or color | `w={px(self.sidebar_width)}` or `bg={cx.theme().background}` | Compile-time |
| Arbitrary conditional GPUI builder calls | `when={(cond, \|el\| ...)}` / `whenSome` | Compile-time |
| Externally assembled class strings | `class={runtime_string}` | Runtime `StyleRefinement` matcher |

Use literal classes for stable structure:

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4 bg-white border border-gray-200 rounded-md" />
}
```

Use composable class arrays or `whenClass` for small static variants:

```rust
rsx! {
    <button
        class={[
            "px-8 py-4 rounded-md",
            if selected { "bg-blue-500 text-white" } else { "bg-gray-100 text-gray-900" },
            disabled && "opacity-50",
        ]}
    >
        {label.as_str()}
    </button>
}
```

Use direct attributes when a value is already a Rust expression:

```rust
rsx! {
    <aside class="min-w-0 border-r border-gray-200" w={px(self.sidebar_width)} />
}
```

Reserve dynamic `class={expr}` for strings genuinely assembled at runtime, and keep them short and predictable.

## Keep Event Logic Out of Markup

Small listeners are fine inline. Move real behavior into named methods:

```rust
impl MyView {
    fn handle_submit(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.validate() {
            self.submit(cx);
            cx.notify();
        }
    }
}

rsx! {
    <button onClick={cx.listener(Self::handle_submit)}>
        {"Submit"}
    </button>
}
```

Batch related state changes before a single `cx.notify()`:

```rust
onClick={cx.listener(|view, _, _window, cx| {
    view.count += 1;
    view.last_update = Instant::now();
    view.recompute_summary();
    cx.notify();
})}
```

## Use Conditional Forms Deliberately

Use Rust `if` or `match` when children differ:

```rust
rsx! {
    <div>
        {if self.items.is_empty() {
            rsx! { <p class="text-gray-500">{"No items"}</p> }
        } else {
            self.render_items()
        }}
    </div>
}
```

Use `when` when the same element gets optional builder calls:

```rust
rsx! {
    <div
        class="px-4 py-2"
        when={(self.has_error, |el| el.bg(rgb(0xfef2f2)).text_color(rgb(0xb91c1c)))}
    >
        {message.as_str()}
    </div>
}
```

Use `whenSome` for optional values:

```rust
rsx! {
    <div whenSome={(self.max_width, |el, width| el.max_w(px(width)))} />
}
```

## Debug Macro Output

Use `rsx_expand!` for a local preview:

```rust
let preview = gpui_rsx::rsx_expand! {
    <div class="flex gap-4 bg-blue-500" />
};
```

For full type checking and compiler diagnostics, still rely on `cargo check` or the repository's demo check:

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
```

## Avoid These Patterns

Avoid a single huge RSX block. Split by UI concepts once scanning becomes hard.

Avoid cloning values only to satisfy children. Prefer `as_str()`, references, or small render helper methods.

Avoid using `key` as a general list marker. It only affects elements that need a generated stateful ID.

Avoid depending on Tailwind variants GPUI cannot express, such as `md:` breakpoints or `dark:`. State variants (`hover:`, `active:`, `focus:`, group variants) are supported; for everything else use GPUI methods, `when`, or explicit state.
