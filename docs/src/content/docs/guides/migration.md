---
title: Migration Guide
description: Migrate from manual GPUI builder chains and upgrade to the GPUI-RSX 0.6 GPUI target.
---

## From Manual GPUI to RSX

GPUI-RSX can be adopted incrementally. Manual GPUI builders and RSX output are both normal GPUI elements, so they can live side by side.

### Elements and Attributes

Before:

```rust
div()
    .flex()
    .flex_col()
    .gap(px(16.0))
    .bg(rgb(0x3b82f6))
```

After:

```rust
rsx! {
    <div class="flex flex-col gap-4 bg-blue-500" />
}
```

Use direct attributes when the source code already has a typed Rust value:

```rust
rsx! {
    <div w={px(width)} bg={self.background_color()} />
}
```

### Children

Before:

```rust
div()
    .child("Hello")
    .child(div().child("World"))
    .child(button().child("Click"))
```

After:

```rust
rsx! {
    <div>
        "Hello"
        <span>{"World"}</span>
        <button>{"Click"}</button>
    </div>
}
```

### Events

Before:

```rust
button()
    .on_click(cx.listener(|view, _event, _window, cx| {
        view.count += 1;
        cx.notify();
    }))
    .child("Click")
```

After:

```rust
rsx! {
    <button onClick={cx.listener(|view, _event, _window, cx| {
        view.count += 1;
        cx.notify();
    })}>
        {"Click"}
    </button>
}
```

### Lists

Before:

```rust
let mut list = div().flex().flex_col();

for item in &self.items {
    list = list.child(div().child(item.name.clone()));
}

list
```

After:

```rust
rsx! {
    <div class="flex flex-col">
        {for item in &self.items {
            <div>{item.name.as_str()}</div>
        }}
    </div>
}
```

If rows are stateful, add `key={...}`:

```rust
rsx! {
    <div class="flex flex-col">
        {for item in &self.items {
            <button key={item.id} onClick={handler}>
                {item.name.as_str()}
            </button>
        }}
    </div>
}
```

### Conditional Rendering

Use Rust conditionals for different children:

```rust
rsx! {
    <div>
        {if show_message {
            rsx! { <span>{"Message visible"}</span> }
        } else {
            rsx! { <span>{"Message hidden"}</span> }
        }}
    </div>
}
```

Use `when` when the same element receives optional builder calls:

```rust
rsx! {
    <div when={(show_message, |el| el.child("Message visible"))} />
}
```

## 0.5.x / crates.io GPUI 0.2.2 to 0.6.0 / Zed Git GPUI

GPUI-RSX `0.6.x` targets GPUI from the Zed repository, not the crates.io `gpui = "0.2.2"` package.

The Zed git dependency still reports the package version as `gpui v0.2.2`, but its API surface is different from the crates.io release. This matters for generated calls such as `flex_grow_1`, `flex_shrink_1`, alignment helpers, desktop sizing utilities, and the newer startup path.

### Replace Dependencies

Before:

```toml
[dependencies]
gpui = "0.2.2"
gpui-rsx = "0.5"
```

After:

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit", "runtime_shaders", "wayland", "x11"] }
gpui-rsx = "0.6"
```

Applications should commit `Cargo.lock` to pin the exact resolved Zed revision.

If your app also depends on `gpui-component`, keep direct `gpui` and `gpui_platform` dependencies on the same source and revision resolved by `gpui-component`. Mixing a bare Zed git dependency with another pinned `rev` can create duplicate GPUI crate instances and incompatible component types.

### Update Application Startup

Before:

```rust
Application::new().run(|cx: &mut App| {
    cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| MyView));
});
```

After:

```rust
use gpui_platform::application;

application().run(|cx: &mut App| {
    cx.open_window(WindowOptions::default(), |_, cx| {
        cx.new(|_| MyView)
    }).unwrap();
    cx.activate(true);
});
```

### Verify the Dependency Graph

Use the demo crate as a known-good integration target:

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui
```

If `cargo tree` shows `git+https://github.com/zed-industries/zed#...`, the project is using Zed git GPUI even if the displayed package version is `0.2.2`.

## Incremental Migration Plan

1. Add `gpui_rsx::rsx` imports in one view module.
2. Convert a small static subtree first.
3. Convert child lists and repeated rows after the static structure compiles.
4. Add `key={...}` only to repeated elements that need stateful IDs.
5. Run `cargo check` after each meaningful step.

Hybrid code is fine:

```rust
rsx! {
    <div>
        {self.legacy_gpui_builder()}
        <section class="flex flex-col gap-4">
            {self.render_new_rsx_panel()}
        </section>
    </div>
}
```

## Common Migration Patterns

| Manual GPUI | GPUI-RSX |
| --- | --- |
| `div().child("Hello")` | `rsx! { <div>{"Hello"}</div> }` |
| `div().child(format!("Count: {count}"))` | `rsx! { <div>{format!("Count: {count}")}</div> }` |
| `.w(px(200.0)).h(px(100.0))` | `<div w={px(200.0)} h={px(100.0)} />` |
| `.flex().gap(px(4.0))` | `<div class="flex gap-4" />` |
| builder constructor with arguments | `<Button base={Button::new("id")} />` |

## Checklist

- [ ] Dependencies use Zed git GPUI for `gpui` and `gpui_platform`.
- [ ] Application startup uses `gpui_platform::application()`.
- [ ] `Cargo.lock` is committed for applications.
- [ ] `gpui-component`, if present, resolves to the same GPUI source.
- [ ] New RSX modules import `gpui_rsx::rsx`.
- [ ] Repeated stateful elements have `key={...}` or explicit `id={...}`.
- [ ] The migrated view passes `cargo check`.
- [ ] Interactions still call `cx.notify()` after state changes.
