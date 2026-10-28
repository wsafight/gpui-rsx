---
title: CLI and editors
description: Diagnose, format, migrate and preview RSX; connect VS Code or Zed.
---

Install the CLI with Rust 1.98 or newer (the macro library still supports Rust 1.88):

```sh
cargo install cargo-rsx --version 0.9.0
cargo rsx doctor --manifest-path demo/Cargo.toml
cargo rsx lint --deny-warnings
cargo rsx lint --json
cargo rsx fmt --check
cargo rsx fmt src/main.rs --write
cargo rsx migrate src/main.rs                 # dry run
cargo rsx convert component.tsx              # JSON: RSX and conversion notes
cargo rsx preview src/main.rs --watch        # http://127.0.0.1:9230
cargo rsx theme check --manifest-path demo/Cargo.toml
cargo rsx theme list --manifest-path demo/Cargo.toml
cargo rsx theme explain bg-primary --manifest-path demo/Cargo.toml
```

`doctor` checks the actual resolved Cargo graph, including aliases, Git dependencies and
published snapshots. `lint` follows workspace members and per-package `rsx.toml` settings.
Use `--tailwind` if your scale comes from a Cargo feature. Dynamic class values are checked
at runtime; the CLI reports that limitation as information.

`fmt` rewrites only RSX macro bodies and keeps surrounding Rust unchanged. Macro bodies
containing comments remain unchanged. Choose `--check` for CI or `--write` to apply edits.

`migrate` preserves numeric pixel spacing by dividing class values by four. Review the dry
run, apply with `--write` once, then enable `tailwind-scale` or set `[options] spacing-scale =
"tailwind"` in `rsx.toml`. It leaves `rsx_tw!`, explicit units and fractions untouched.
Review dynamic strings and any scale overrides manually.

`theme check` validates the nearest `rsx.toml`; `list` emits every generated class and GPUI
method, while `explain` shows the source token and expansion for one class. Add `--json` to
all three commands for editor or CI integrations. The LSP uses the same parser and reports
invalid theme entries at the start of the document.

JSX/TSX conversion uses SWC and produces RSX plus notes for unsupported classes, JavaScript
expressions and handlers. Preview approximates static layout and labels omitted behavior;
use `--index N` for a particular macro. The [Playground](../../playground/) runs the same
parser, converter and preview locally in your browser.

## Editors

Download the VSIX from the GitHub release and use **Extensions: Install from VSIX** in
VS Code. Configure `gpuiRsx.serverPath` if `cargo-rsx` is not in PATH. Restart with
**GPUI RSX: Restart language server**.

For Zed, install `editors/zed` as a development extension. Add `"gpui-rsx"` alongside
`"rust-analyzer"` if you customize Rust language servers. Public marketplace availability
is recorded in the release notes; downloading an extension package does not imply market
approval.

Both editors provide class completion, expansion hover, diagnostics, quick fixes, color
swatches and formatting through `cargo rsx lsp`. Class completion operates in static
literals within balanced RSX macro bodies. rust-analyzer continues to check Rust types.
