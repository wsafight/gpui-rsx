---
title: gpui-component
description: Use GPUI-RSX with gpui-component builders without duplicate GPUI crate instances.
---

`gpui-component` works with GPUI-RSX when its component builders and the application's GPUI dependencies resolve to the same GPUI source.

## Dependency Shape

The demo uses:

```toml
[dependencies]
gpui = { package = "gpui-kit", version = "=0.8.0" }
gpui-rsx = { path = ".." }
```

Kit supplies the matching GPUI snapshot, component library and assets. Commit the application lockfile. Initialize with `gpui::init(cx)` and open windows with `gpui::open_window(...)`.

## Use `base` for Constructors

GPUI-RSX can infer simple constructors from tag names, but component builders often need explicit constructors. Use the macro-only `base` attribute:

```rust
use gpui::component::button::{Button, ButtonVariants as _};
use gpui::component::Sizable as _;

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

Use `base` for component methods whose names overlap GPUI stateful attributes. For example,
`gpui::component::tab::Tab::aria_label` is a component builder; writing it as an RSX attribute would
be classified as GPUI's stateful `aria_label`. Keep the component-specific chain explicit:

```rust
<Tab base={Tab::new().label("Overview").aria_label("Overview tab")} underline />
```

Kit exports styled components through `gpui::component` and unstyled behavior through
`gpui::base`. The serializable `Edges` helper is exported by `gpui::base` and `gpui::component`; all layers share the same engine.

## Import Extension Traits

Many `gpui-component` methods come from extension traits. Import those traits explicitly near the component usage:

```rust
use gpui::component::button::ButtonVariants as _;
use gpui::component::Sizable as _;
```

This keeps macro-generated method chains type-checkable and makes missing trait imports easy to diagnose.

## Compatibility Check

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
cargo xtask check-gpui
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui-pre
```

If component types do not line up, inspect the tree first. Most failures come from duplicate GPUI crate instances.
