---
title: GPUI-RSX
description: A compile-time RSX macro for building GPUI views with JSX-like syntax.
---

GPUI-RSX is a Rust procedural macro that turns JSX-like markup into native GPUI builder calls at compile time.

Use it when you want GPUI code that keeps the type safety and performance profile of normal Rust, but reads closer to the shape of the UI you are building.

## What it provides

- HTML-like tags that expand to GPUI elements.
- Attribute syntax for GPUI builder methods.
- `class="..."` support for a Tailwind-like subset mapped to GPUI APIs.
- Conditional rendering and Rust expressions inside `{ ... }`.
- Fragment support with `<>...</>`.
- Loop rendering with `key={...}` support for repeated stateful elements.
- Strict, permissive, and expansion-preview macros.

## What it is not

GPUI-RSX is not a browser renderer and it is not a Tailwind CSS runtime. Static RSX is compiled into Rust code. Dynamic classes are handled by a generated runtime matcher that supports the common GPUI utility subset documented in this site.

## Main macros

```rust
use gpui_rsx::{rsx, rsx_expand, rsx_permissive, rsx_strict};
```

| Macro | Use case |
| --- | --- |
| `rsx!` | Default permissive RSX macro. |
| `rsx_strict!` | Rejects unsupported static classes and panics on unsupported dynamic classes when evaluated. |
| `rsx_permissive!` | Explicit permissive mode for unknown class handling. |
| `rsx_expand!` | Returns a string preview of generated GPUI builder code for debugging. |

## Documentation Map

The Starlight site is the canonical documentation set. The old `docs/*.md` files have been folded into these pages:

- [Getting Started](/gpui-rsx/getting-started/) for installation and a first view.
- [Syntax Reference](/gpui-rsx/usage/syntax/) for elements, attributes, children, and conditionals.
- [Class Handling](/gpui-rsx/usage/class/) for static and dynamic utility classes.
- [IDs and Keys](/gpui-rsx/usage/ids/) for auto IDs, loop safety, and explicit IDs.
- [Performance](/gpui-rsx/guides/performance/) for static expansion, dynamic class cost, render-loop allocation, and validation.
- [Best Practices](/gpui-rsx/guides/best-practices/) for project structure and everyday patterns.
- [Migration Guide](/gpui-rsx/guides/migration/) for manual GPUI migration and the `0.6.x` GPUI target change.
- [Troubleshooting](/gpui-rsx/guides/troubleshooting/) for parser, type, runtime, and dependency issues.
- [API Reference](/gpui-rsx/reference/api/) for compact macro and mapping tables.
