---
title: Compatibility
description: GPUI Kit, gpui-pre, gpui-component, Rust, and demo lockfile compatibility notes.
---

The current verified target is GPUI Kit and its exact `gpui-pre` snapshot.
Longbridge publishes the `gpui-pre-*` family as snapshots of Zed GPUI; its version
numbers are independent of the official `gpui` crate.

The macro and core crates have no GPUI runtime dependency. The demo names
`gpui-kit` as `gpui` to preserve paths used by RSX expansions and GPUI derives.

<!-- gpui-rsx:compat:start -->
### Verified Compatibility

| GPUI-RSX | GPUI Kit | GPUI snapshot | Component |
| --- | --- | --- | --- |
| `0.8.0` | `0.8.0` | `gpui-pre@0.3.9` | `gpui-component@0.8.0` |

This matrix comes from demo/Cargo.lock. Kit supplies GPUI, components and assets; the Cargo alias below preserves the gpui:: path used by RSX expansions. Commit the application lockfile.

```toml
[dependencies]
gpui = { package = "gpui-kit", version = "=0.8.0" }
gpui-rsx = "0.8.0"
```
<!-- gpui-rsx:compat:end -->

## Current Demo Matrix

| Item | Value |
| --- | --- |
| Rendering backend | Kit's default `gpui-pre`, published on crates.io |
| GPUI-RSX root MSRV | `1.88.0` |
| Demo Rust toolchain | `1.98.1` |
| Demo check | `cargo +1.98.1 check --manifest-path demo/Cargo.toml --bins --locked` |

The root MSRV and demo toolchain are managed separately. Working on the macro does
not require GPUI platform dependencies.

## Share One GPUI Type Identity

Use Kit's exports through `gpui::component`, `gpui::base` and `gpui::platform`.
Call `gpui::init(cx)` before opening windows and use `gpui::open_window(...)` to
mount the root. Load icons with `application().with_assets(gpui::assets::Assets)`.

Applications no longer need direct dependencies on the old Zed Git `gpui`,
`gpui_platform`, or Git component library. Each crate instance owns distinct Rust
types; types from different GPUI instances cannot be interchanged.

```bash
cargo xtask check-gpui
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui-pre
```

`check-gpui` requires one rendering engine, matching platform/macros snapshot
versions, and the same GPUI instance in Kit, base and component. It prints the
verified package versions as JSON for CI.

## Update the Matching Set

Follow Kit's published releases. The compatibility workflow selects the latest
stable, non-yanked Kit on crates.io, updates the exact demo pin, and regenerates
the lockfile, API tables, snapshots, specification and this matrix. Linux and
macOS validate the same patch before a repair PR is created; failures update one
tracking issue.

GPUI Fast is optional. The demo and compatibility matrix cover Kit's default
gpui-pre backend.
