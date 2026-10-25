# GPUI RSX for VS Code

Install the server first:

```sh
cargo install cargo-rsx
```

Provides supported class completion, class diagnostics and quick fixes, expanded GPUI
hover, color swatches, and RSX formatting inside Rust files. It runs alongside
rust-analyzer. Configure `gpuiRsx.serverPath` if `cargo-rsx` is not in the editor's PATH.
Use **GPUI RSX: Restart language server** after changing this setting.

Build a VSIX from the repository:

```sh
cd editors/vscode
bun install --frozen-lockfile
bun run check
bun run package
code --install-extension gpui-rsx-0.8.2.vsix
```

Formatting leaves commented macro bodies unchanged to preserve comments. Dynamic class
strings are diagnosed at runtime; class completion operates in static literals within
balanced RSX macro bodies. Static preview and JSX conversion are available through the
CLI and documentation Playground.
