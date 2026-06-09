---
title: Compatibility
description: GPUI, gpui-component, Rust, and demo lockfile compatibility notes.
---

GPUI-RSX `0.6.x` targets GPUI from the Zed repository instead of the crates.io `gpui = "0.2.2"` package.

The Zed git dependency currently still reports its crate package version as `gpui v0.2.2`, but it exposes a different API surface from the crates.io release. This is why `cargo tree` can show `gpui v0.2.2` even when the project is correctly using Zed git GPUI.

## Current Demo Matrix

| Item | Value |
| --- | --- |
| GPUI-RSX | `0.6.0` |
| GPUI source | `https://github.com/zed-industries/zed` |
| GPUI resolved revision | `f164afda46188939f76c24aba4099d04423bc356` |
| `gpui-component` revision | `8752104289424b7f35045b68a2d394018da48e7e` |
| Demo Rust toolchain | `1.95.0` |
| Demo check | `cargo check --manifest-path demo/Cargo.toml --bins --locked` |

## Avoid Duplicate GPUI Crates

When using `gpui-component`, keep all direct GPUI dependencies on the same source and revision as `gpui-component`.

Bad dependency graphs often look like this:

- one `gpui` crate from a bare Zed git dependency,
- another `gpui` crate from a pinned `rev`,
- component types that no longer match because they came from different crate instances.

Pinning the application lockfile and using the demo as a reference keeps the graph deterministic.

## Inspect the Resolved GPUI

```bash
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui
```

If the source is `git+https://github.com/zed-industries/zed#...`, the demo is using Zed git GPUI even if the displayed package version is `0.2.2`.
