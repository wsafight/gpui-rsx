---
title: Runnable Demos
description: Choose a native GPUI example, run it, and find the APIs it demonstrates.
---

The `demo/` crate contains ten desktop examples. They use the local GPUI-RSX 0.11 source and the locked GPUI Kit dependency set. Start with a small view to learn one API, or open a console to follow state across several panels.

## Choose an example

Run from the repository root, replacing `counter` with any binary below:

```sh
cargo run --manifest-path demo/Cargo.toml --bin counter --locked
```

The crate pins Rust 1.98.1. Install the native dependencies for your platform before building; see [compatibility](/gpui-rsx/compatibility/).

| Binary | What to try | Source / API focus |
| --- | --- | --- |
| `starter` | Run the README's counter. | Minimal complete app, native Button, `rsx_listener!`. |
| `hello` | Open a minimal view. | Static RSX, semantic theme colors, explicit pixel spacing. |
| `counter` | Increase, decrease, then reset the count. | `rsx_listener!`, `rsx_update!`, conditional classes, disabled native Button. |
| `palette` | Compare swatches and their class names. | Dynamic class expressions and color values. |
| `task_list` | Complete and reopen individual tasks. | Keys on repeated stateful controls, captured task IDs, derived completion counts. |
| `component` | Type into the input and open the popover. | Kit Button/Input/Popover, function components, context injection, child props. |
| `syntax_gallery` | Toggle conditions and reverse rows. Scroll to compare the other syntax examples. | Fragments, spread, keyed loops, components, state variants, negative margins, SVG/image/canvas. |
| `api_surface` | Toggle conditional styles and hover the group. | Flex aliases, accessibility attributes, tooltips, group states. |
| `project_dashboard` | Filter work, change density, complete items, and add a follow-up. | Shared tokens, selection reconciliation, conditional detail fragments, empty-state recovery. |
| `incident_console` | Filter, assign, advance status, and inject a signal. | Modular panels, shared runtime classes, lifecycle methods, timeline, empty states. |

## Features used in the current version

The counter uses `rsx_listener!(cx, notify, ...)` for increment/decrement. Reset uses the handler-managed form with `rsx_update!`, so each action sends one notification. These are optional helpers; direct `cx.listener` callbacks remain valid.

The syntax gallery uses `rsx_fragment!` with spread children and shows 0.11 negative margin classes with overlapping avatars. Negative sizes have different layout behavior: GPUI clamps widths, heights, and maximum sizes to zero and ignores negative minimum sizes. See [Class Handling](/gpui-rsx/usage/class/).

`demo/rsx.toml` enables the Kit theme preset and shared dynamic runtime. Small examples use semantic theme colors. The two consoles share named project colors so their panels stay consistent. Keep fixed layout classes static; dynamic class strings support a smaller utility set.

## Check behavior without opening windows

```sh
cargo +1.98.1 test --manifest-path demo/Cargo.toml --lib --bins --tests --locked
cargo +1.98.1 test --manifest-path demo/Cargo.toml --doc --locked
```

These tests cover model transitions, real GPUI layout, IDs, callbacks, and rejected syntax. They do not compare screenshots. `api_contract`, `generated_api_contract`, and `semantics_contract` are compile contracts, so they are not listed as interactive examples.

For a rebuild/restart loop, use the [development reload workflow](/gpui-rsx/guides/performance/#measure-development-reloads). It resets application state on each restart.
