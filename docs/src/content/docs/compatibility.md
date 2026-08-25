---
title: Compatibility
description: GPUI, gpui-component, Rust, and demo lockfile compatibility notes.
---

GPUI-RSX `0.7.x` targets GPUI from the Zed repository instead of the crates.io `gpui = "0.2.2"` package.

The Zed git dependency currently still reports its crate package version as `gpui v0.2.2`, but it exposes a different API surface from the crates.io release. This is why `cargo tree` can show `gpui v0.2.2` even when the project is correctly using Zed git GPUI.

## Current Demo Matrix

| Item | Value |
| --- | --- |
| GPUI-RSX | `0.7.0` |
| GPUI source | `https://github.com/zed-industries/zed` |
| GPUI resolved revision | `e973593455af18719be22b0455c3f928c6ccc24d` |
| `gpui-component` revision | `7885c41663c7a6cc68ad0c99b1ba33550f807ff0` |
| `gpui-base` revision | `7885c41663c7a6cc68ad0c99b1ba33550f807ff0` |
| GPUI-RSX root MSRV | `1.88.0` |
| Demo Rust toolchain | `1.95.0` |
| Demo check | `cargo +1.95.0 check --manifest-path demo/Cargo.toml --bins --locked` |

The root MSRV is a lower bound, not a required day-to-day toolchain. Newer stable Rust versions are
supported; this locked demo revision was also verified with Rust `1.98.0`. Rust `1.95.0` remains the
explicit reproducibility lane for the demo.

## Avoid Duplicate GPUI Crates

When using `gpui-component`, keep all direct GPUI dependencies on the same source and revision as `gpui-component`.

Bad dependency graphs often look like this:

- one `gpui` crate from a bare Zed git dependency,
- another `gpui` crate from a pinned `rev`,
- component types that no longer match because they came from different crate instances.

Pinning the application lockfile and using the demo as a reference keeps the graph deterministic.

The current gpui-component main branch splits lower-level controls into `gpui-base`. Applications
normally keep depending on the `gpui-component` facade; its public re-exports cover supported base
types. Add a direct `gpui-base` dependency only when using an API that the facade does not export,
and keep it on the same source revision.

## Inspect the Resolved GPUI

```bash
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui
```

If the source is `git+https://github.com/zed-industries/zed#...`, the demo is using Zed git GPUI even if the displayed package version is `0.2.2`.
