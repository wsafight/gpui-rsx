# cargo-rsx

Tools for [GPUI-RSX](https://github.com/wsafight/gpui-rsx). Requires Rust 1.98+.

```sh
cargo install cargo-rsx
cargo rsx doctor --manifest-path demo/Cargo.toml
cargo rsx lint --deny-warnings
cargo rsx fmt --check
cargo rsx fmt src/main.rs --write
cargo rsx migrate src/main.rs              # dry run; inspect first
cargo rsx migrate src/main.rs --write      # run once, then enable Tailwind scale
cargo rsx convert component.tsx            # JSON: RSX + source-positioned TODOs
cargo rsx expand layout.rsx --tailwind
cargo rsx preview src/main.rs --watch      # localhost:9230, reloads layout
cargo rsx theme check --manifest-path demo/Cargo.toml
cargo rsx theme list --json
cargo rsx theme explain bg-primary
cargo rsx lsp                              # stdio language server
```

`doctor` accepts registry, Git, Kit and standalone GPUI graphs, including aliases.
`lint` walks Cargo workspace packages, respects each package's `rsx.toml`, and reports
source positions, suggestions and dynamic-class limitations. `--json` produces structured
reports. Diagnostics in dynamic strings happen at runtime.

`fmt` only rewrites RSX macro bodies. Commented macro bodies remain unchanged so comments
are never lost. `migrate` converts numeric px values to equivalent Tailwind units, including
literal branches and state classes. Back up/review the diff, run it once, then enable
`tailwind-scale` or set `[options] spacing-scale = "tailwind"` in `rsx.toml`. Dynamic values
and manually overridden per-macro scale require review. The command refuses projects
already configured for Tailwind; `rsx_tw!` bodies are untouched.

JSX conversion uses SWC, preserves static content and supported classes, and reports
JavaScript expressions, event handlers and unsupported styles for manual work. It is a
starting point, not a React runtime translator. Preview approximates static layouts in
HTML and labels expressions/events/custom components and unsupported styles.

`theme check/list/explain` validates and inspects the same tokens used by the macro,
including text size, line height, opacity and shadow expressions. All commands accept
`--json`; invalid configs and unknown theme classes exit with status 1.

The LSP provides incremental document sync, supported-class completion, expansion hover,
diagnostics, quick fixes, semantic tokens, color swatches and RSX formatting inside Rust macro bodies. Install `cargo-rsx` first,
then install the VS Code or Zed extension from `editors/` in the repository.
