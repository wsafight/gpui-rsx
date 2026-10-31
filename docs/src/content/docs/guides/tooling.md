---
title: CLI & Editors
description: Set up the Rust CLI, format and check RSX, convert JSX, and connect VS Code or Zed.
---

The CLI and editor language server share the macro's parser and class configuration. Install with Rust 1.98 or newer; the macro/core library supports Rust 1.88.

## Install and diagnose

```sh
cargo install cargo-rsx --version 0.10.1 --locked
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

`lint` follows workspace members and reads each package's `rsx.toml`. The spacing scale follows the macro: `rsx.toml` first, then the `tailwind-scale` feature of `gpui-rsx`; `--tailwind` forces the Tailwind scale. Dynamic class values are checked at runtime; the CLI reports that limitation as information.

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
cargo rsx preview demo --index 0
```

Conversion uses SWC and returns JSON containing RSX plus review notes for unsupported classes, JavaScript expressions, and handlers. Implement those behaviors in Rust before using the result.

Directory preview finds Rust files in the package and selects the first RSX macro, or the
macro chosen with `--index N`. `--watch` also refreshes after configuration edits.

## Inspect generated Rust

```sh
printf '<div class="flex gap-4" />' | cargo rsx expand -
cargo rsx expand body.rsx --manifest-path demo/Cargo.toml
```

`expand` accepts a standalone RSX body and prints the complete Rust expansion, including
inline dynamic helpers when shared runtime is not configured. Use `--strict` or `--tailwind`
to override the mode. For a Rust file with multiple macros, use `preview --index N` to inspect
layout or the editor's expansion hover; `cargo check` validates the generated Rust types.

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
| VS Code | Download the `.vsix` from the [0.10.1 release](https://github.com/wsafight/gpui-rsx/releases/tag/v0.10.1), then run **Extensions: Install from VSIX**. |
| Zed | Install `editors/zed` from the repository as a development extension. If you customize Rust language servers, add `"gpui-rsx"` alongside `"rust-analyzer"`. |

Both integrations use `cargo rsx lsp` and provide:

- Static class completion and expansion hover.
- Diagnostics, quick fixes, and theme color swatches.
- Semantic highlighting for supported classes, theme tokens, and invalid classes.
- Optional RSX formatting inside Rust files.

RSX formatting is off by default because it rewrites RSX macro bodies only, never the
surrounding Rust, and an editor would otherwise offer it in place of rustfmt. Enable it with
`"gpuiRsx.format.enable": true` in VS Code, or in Zed with
`{"lsp":{"gpui-rsx":{"initialization_options":{"formatting":true}}}}`. `cargo rsx fmt` is always
available.

The server applies incremental edits and reuses analysis for unchanged macro bodies. Class completion operates in static literals within balanced RSX macros. Keep rust-analyzer enabled for Rust type checking.

If VS Code cannot find the server, set `gpuiRsx.serverPath` to the `cargo-rsx` executable, then run **GPUI RSX: Restart language server**. Extension marketplace status is recorded separately in release notes.

## Diagnostic filters and local component hints

Every command supports `--help`, including `theme check/list/explain`. Stdin lint accepts
both Rust files and standalone bodies:

```sh
printf '<div class={classes}/>' | cargo rsx lint - --min-severity warning
cargo rsx lint --quiet --deny-warnings
cargo rsx convert component.tsx --tailwind
```

`--min-severity error|warning|information` filters text and JSON diagnostics; the default is
`information`. `--quiet` / `-q` shows errors only. Filtering does not change failure status:
`--deny-warnings` still fails for hidden warnings. Tailwind conversion preserves numeric
class values and emits an `rsx_tw! { ... }` expression; default conversion retains pixel output.
Simple identifiers and member access survive conversion, while JavaScript calls, operators,
optional chaining and handlers require manual Rust work.

Preserved expressions such as `{items}` receive no manual-conversion diagnostic. The
converter does not resolve Rust types or ownership; collections used as children may need
`{...items}` or `{for ...}`, and field access may require borrowing or cloning.

The LSP completes tags, GPUI methods and props declared by `#[component]` in the current
Rust file (including inline modules). It offers fixes for bare text, mismatched closing tags,
missing required props and missing loop keys. Prop/key fixes insert editable `todo!` expressions;
replace them before running the application. This catalog does not resolve imported components,
external modules, macros or Rust types; rust-analyzer supplies type checking.

VS Code and Zed clients supporting dynamic file-watch registration receive watches for
`rsx.toml` and `Cargo.toml` from the server. Configuration changes reanalyze open documents.

The Playground accepts an optional `rsx.toml` theme (up to 20,000 characters), includes it
in share links and shows configuration errors beside source diagnostics. Static theme tokens
appear in preview; arbitrary Rust expressions are never executed. Theme spacing options take
precedence over the scale checkbox. Conversion follows that effective scale when selecting `rsx_tw!` output.
