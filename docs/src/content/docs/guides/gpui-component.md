---
title: gpui-component
description: Use GPUI-RSX with gpui-component builders without duplicate GPUI crate instances.
---

`gpui-component` works with GPUI-RSX when its component builders and the application's GPUI dependencies resolve to the same GPUI source.

## Dependency Shape

The demo uses:

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit", "runtime_shaders", "wayland", "x11"] }
gpui-rsx = { path = ".." }
gpui-component = { git = "https://github.com/longbridge/gpui-component", rev = "8752104289424b7f35045b68a2d394018da48e7e" }
```

Commit `Cargo.lock` for applications so the resolved Zed revision does not float unexpectedly.

## Use `base` for Constructors

GPUI-RSX can infer simple constructors from tag names, but component builders often need explicit constructors. Use the macro-only `base` attribute:

```rust
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::Sizable as _;

rsx! {
    <Button
        base={Button::new("save")}
        label={"Save"}
        small
        primary
    />
}
```

The `base` attribute replaces the inferred constructor and does not generate a `.base(...)` method call.

## Import Extension Traits

Many `gpui-component` methods come from extension traits. Import those traits explicitly near the component usage:

```rust
use gpui_component::button::ButtonVariants as _;
use gpui_component::Sizable as _;
```

This keeps macro-generated method chains type-checkable and makes missing trait imports easy to diagnose.

## Compatibility Check

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui
```

If component types do not line up, inspect the tree first. Most failures come from duplicate GPUI crate instances.
