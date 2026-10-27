---
title: Migration Guide
description: Adopt RSX and migrate existing Zed Git applications to GPUI Kit.
---


## Zed Git to GPUI Kit 0.8

Replace the separate Zed Git `gpui`, `gpui_platform` and Git `gpui-component`
dependencies with Kit, preserving the `gpui` crate name for generated code:

```toml
[dependencies]
gpui = { package = "gpui-kit", version = "=0.8.0" }
gpui-rsx = "0.8.3"
```

Kit 0.8.0 pins `gpui-pre = 0.3.9`. Use `gpui::component` for styled controls and
`gpui::base` for unstyled behavior. Replace `gpui_platform::application()` with
`gpui::application()`, initialize Kit before opening a window, and mount its root:

```rust
gpui::application().with_assets(gpui::assets::Assets).run(|cx| {
    gpui::init(cx);
    gpui::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
        cx.new(|_| MyView)
    }).expect("failed to open window");
});
```

Regenerate and commit the application lockfile after the dependency change. The
repository demo uses Rust 1.98.1; the macro/core MSRV remains Rust 1.88. Check
`cargo xtask check-gpui` and the real demo bins before merging the update. GPUI
Fast is a separate optional backend, outside the current compatibility target.

The `gpui-component` theme preset now emits `cx.theme().colors.<field>`. This
keeps colors such as `list` distinct from Kit settings such as `Theme.list`.
Import `gpui::component::ActiveTheme` when using the preset.

The 0.5-to-0.6 section below documents the historical Zed Git target; use the
current [compatibility matrix](/gpui-rsx/compatibility/) for new dependencies.

## Spacing scale, state variants and components

Numeric spacing/sizing remains in pixels by default: `gap-4` is 4px. To use
Tailwind units (4px per unit), choose `rsx_tw!`, enable the `tailwind-scale` Cargo
feature, or put the following in an `rsx.toml` next to your crate's `Cargo.toml`:

```toml
[options]
spacing-scale = "tailwind"
```

| Class | Default | Tailwind scale |
| --- | --- | --- |
| `gap-4` | 4px | 16px |
| `p-2.5` | 2.5px | 10px |
| `w-64` | 64px | 256px |
| `gap-[14px]` | 14px | 14px |

To preserve an existing layout when switching, replace numeric values with explicit
pixels (`gap-4` → `gap-[4px]`) or divide by four (`gap-4` → `gap-1`). Check static
literals, conditional branches and dynamic strings. Typed attributes (`gap={px(4.0)}`),
fractions, arbitrary values, colors, opacity and GPUI presets (`rounded-md`, `border-2`)
are unchanged. Prefer crate-local `rsx.toml` in libraries because Cargo features unify
across the dependency graph.

Static class literals now accept `hover:`, `focus:`, `focus-visible:`, `active:`,
`group-hover:` and `group-active:` (including named groups). Stateful variants inject
IDs; supply stable keys in loops. Responsive and `dark:` variants emit warnings in
permissive mode and errors in strict mode. Dynamic class strings cannot use variants
or theme tokens. `onUnknownClass={|class: &str| ...}` handles unsupported dynamic classes
without the strict mode's default panic; `dynamic-class-report` collects debug reports
until process exit.

`#[component]` generates a props builder for named structs. Plain fields are required;
`#[prop(default)]` and `#[prop(default = expr)]` are optional. A `#[children]` field
collects children. Third-party constructors continue to work with `base={...}`.

Theme tokens in `rsx.toml` support `[colors]`, `[radius]` and `[spacing]`. Use
`[options] preset = "gpui-component"` for semantic colors; import the theme extension
trait and keep `cx` in scope. Tokens with `{ expr = "cx.theme().background" }` read the
current theme during rendering. Literal hex tokens remain fixed colors.

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
    <div class="flex flex-col gap-[16px] bg-blue-500" />
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

For the historical Zed Git target, inspect the application dependency tree. The current demo uses Kit; see the compatibility matrix above:

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui-pre
cargo xtask check-gpui
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

- [ ] The current Kit release is aliased as `gpui` and uses its matching snapshot.
- [ ] Startup uses `gpui::application()`, `gpui::init(cx)` and `gpui::open_window(...)`.
- [ ] `Cargo.lock` is committed for applications.
- [ ] Components are imported through `gpui::component` and share the GPUI instance.
- [ ] New RSX modules import `gpui_rsx::rsx`.
- [ ] Repeated stateful elements have `key={...}` or explicit `id={...}`.
- [ ] The migrated view passes `cargo check`.
- [ ] Interactions still call `cx.notify()` after state changes.
