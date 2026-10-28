---
title: CLI & Editors
description: Set up the Rust CLI, format and check RSX, convert JSX, and connect VS Code or Zed.
---

The CLI and editor language server share the macro's parser and class configuration. Install with Rust 1.98 or newer; the macro/core library supports Rust 1.88.

## Install and diagnose

```sh
cargo install cargo-rsx --version 0.9.0 --locked
cargo rsx doctor --manifest-path demo/Cargo.toml
```

The `demo/Cargo.toml` examples below run from the repository root. For your own app, use its manifest path. `doctor` checks the resolved Cargo graph, including aliases, Git dependencies, and published snapshots.

## Check and format

```sh
cargo rsx lint --deny-warnings
cargo rsx lint --json
cargo rsx fmt --check
cargo rsx fmt src/main.rs --write
```

`lint` follows workspace members and reads each package's `rsx.toml`. Use `--tailwind` if your spacing scale comes from a Cargo feature. Dynamic class values are checked at runtime; the CLI reports that limitation as information.

`fmt` rewrites only RSX macro bodies and keeps surrounding Rust unchanged. Bodies containing comments remain unchanged. Use `--check` in CI and `--write` to apply edits.

## Inspect a theme

```sh
cargo rsx theme check --manifest-path demo/Cargo.toml
cargo rsx theme list --manifest-path demo/Cargo.toml --json
cargo rsx theme explain bg-primary --manifest-path demo/Cargo.toml
```

`check` validates the configuration; `list` shows generated classes and GPUI methods; `explain` traces one class to its source token and expansion. All three accept `--json`. See [Themes & Design Tokens](/gpui-rsx/guides/theming/) for configuration examples and exit codes.

## Convert and preview

```sh
cargo rsx convert component.tsx
cargo rsx preview src/main.rs --watch
```

Conversion uses SWC and returns JSON containing RSX plus review notes for unsupported classes, JavaScript expressions, and handlers. Implement those behaviors in Rust before using the result.

Preview serves an approximate static layout at `http://127.0.0.1:9230` and labels omitted behavior. Use `--index N` to choose a macro. The [Playground](/gpui-rsx/playground/) runs the same parser, converter, and preview locally in your browser. Verify final rendering and interactions in a native GPUI application.

## Migrate spacing

```sh
cargo rsx migrate src/main.rs           # review the dry run
cargo rsx migrate src/main.rs --write   # apply once
```

Migration divides numeric class values by four to preserve pixel dimensions when enabling Tailwind scale. After applying it once, enable `tailwind-scale` or set `[options] spacing-scale = "tailwind"` in `rsx.toml`. The command leaves `rsx_tw!`, explicit units, and fractions untouched. Review dynamic strings and scale overrides manually; follow the [migration guide](/gpui-rsx/guides/migration/) for existing applications.

## Connect your editor

| Editor | Setup |
| --- | --- |
| VS Code | Download the `.vsix` from the [0.9.0 release](https://github.com/wsafight/gpui-rsx/releases/tag/v0.9.0), then run **Extensions: Install from VSIX**. |
| Zed | Install `editors/zed` from the repository as a development extension. If you customize Rust language servers, add `"gpui-rsx"` alongside `"rust-analyzer"`. |

Both integrations use `cargo rsx lsp` and provide:

- Static class completion and expansion hover.
- Diagnostics, quick fixes, and theme color swatches.
- Semantic highlighting for supported classes, theme tokens, and invalid classes.
- RSX formatting inside Rust files.

The server applies incremental edits and reuses analysis for unchanged macro bodies. Class completion operates in static literals within balanced RSX macros. Keep rust-analyzer enabled for Rust type checking.

If VS Code cannot find the server, set `gpuiRsx.serverPath` to the `cargo-rsx` executable, then run **GPUI RSX: Restart language server**. Extension marketplace status is recorded separately in release notes.
