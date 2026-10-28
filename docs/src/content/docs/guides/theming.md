---
title: Themes & Design Tokens
description: Define project-wide colors, spacing, typography, opacity, and shadows in rsx.toml.
---

Keep shared visual decisions in `rsx.toml`, next to your crate's `Cargo.toml`. Static class literals resolve these tokens during macro expansion; the generated code remains ordinary GPUI builder calls.

## Define your tokens

Start with a small configuration:

```toml title="rsx.toml"
[colors]
brand = "#2563eb"

[radius]
card = "12px"

[spacing]
gutter = "18px"

[text-size]
body = "14px"

[line-height]
body = "20px"

[opacity]
muted = "60%"
```

Use the generated classes in a static literal:

```rust
rsx! {
    <div class="flex flex-col gap-gutter p-gutter rounded-card">
        <h1 class="text-brand font-bold">{"Project overview"}</h1>
        <p class="text-size-body leading-body opacity-muted">
            {"A shared vocabulary for your views."}
        </p>
    </div>
}
```

| Section | Example class | GPUI method |
| --- | --- | --- |
| `colors` | `bg-brand`, `text-brand`, `border-brand` | `.bg()`, `.text_color()`, `.border_color()` |
| `radius` | `rounded-card` | `.rounded()` |
| `spacing` | `p-gutter`, `gap-gutter` | `.p()`, `.gap()` and directional equivalents |
| `text-size` | `text-size-body` | `.text_size()` |
| `line-height` | `leading-body` | `.line_height()` |
| `opacity` | `opacity-muted` | `.opacity()` |
| `shadow` | `shadow-card` | `.shadow()` |

Use explicit units for lengths. Opacity accepts a quoted fraction (`"0.6"`) or percentage (`"60%"`). Duplicate keys, malformed values, and conflicting `text-size-*` classes are configuration errors.

## Follow the application theme

The `gpui-component` preset supplies semantic color classes such as `bg-background`, `text-foreground`, and `bg-primary`:

```toml title="rsx.toml"
[options]
preset = "gpui-component"
# context = "cx"  # default; change this if your render context has another name
```

Import the theme trait and keep `cx` in scope where the macro expands:

```rust
use gpui::*;
use gpui::prelude::*;
use gpui::component::ActiveTheme;
use gpui_rsx::rsx;

// Inside a Render implementation with cx: &mut Context<Self>:
rsx! {
    <div class="bg-background text-foreground p-[24px]">
        {"Follows the active application theme."}
    </div>
}
```

The preset generates expressions such as `cx.theme().colors.background`. Literal hex tokens remain fixed colors. A custom color with the same name overrides the preset entry.

For component constructors, initialization, and application setup, see [gpui-component integration](/gpui-rsx/guides/gpui-component/).

## Rust expressions and shadows

Tokens can contain Rust expressions evaluated when the view renders:

```toml title="rsx.toml"
[colors]
surface = { expr = "cx.theme().colors.background" }

[shadow]
card = { expr = "vec![BoxShadow::new(px(0.0), px(1.0), hsla(0.0, 0.0, 0.0, 0.2))]" }
```

Use `class="bg-surface shadow-card"`. Shadow tokens require an expression that produces a value accepted by GPUI's `.shadow()` method. Every referenced name, including `BoxShadow`, `px`, and `hsla`, must be available in the Rust scope where the macro expands.

:::note[Static classes and preview]
Theme tokens and state variants work in static class literals, including literal conditional branches. Runtime-generated class strings cannot resolve them. Static token values appear in preview and editor color swatches; arbitrary Rust expressions are not executed by preview.
:::

## Inspect the configuration

After [installing the CLI](/gpui-rsx/guides/tooling/), run these commands from your application or point to a package manifest:

```sh
cargo rsx theme check --manifest-path demo/Cargo.toml
cargo rsx theme list --manifest-path demo/Cargo.toml
cargo rsx theme explain hover:opacity-muted --manifest-path demo/Cargo.toml
```

All three commands accept `--json`. Validation failures return exit status `1`; usage errors return `2`. The editor language server uses the same configuration parser.

## Choose a spacing scale

Numeric spacing defaults to pixels: `gap-4` means 4px. To opt into Tailwind's 4px units for a crate:

```toml title="rsx.toml"
[options]
spacing-scale = "tailwind"
```

You can combine this setting with `preset` in the same `[options]` section. Explicit lengths such as `gap-[14px]` and named tokens such as `gap-gutter` keep their declared values. Review the [spacing migration guide](/gpui-rsx/guides/migration/#spacing-scale-state-variants-and-components) before changing an existing layout.
