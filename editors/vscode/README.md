# GPUI RSX for VS Code

Install the server first:

```sh
cargo install cargo-rsx
```

Provides supported class completion, class diagnostics and quick fixes, expanded GPUI
hover, theme color swatches, semantic highlighting, and optional RSX formatting inside Rust
files. Formatting is off by default because it only rewrites RSX macro bodies; enable it with
`gpuiRsx.format.enable` and keep rust-analyzer as the default Rust formatter.
Semantic tokens distinguish supported utilities, theme tokens and invalid classes. The
server applies incremental edits and reuses analysis for unchanged RSX bodies. It runs alongside
rust-analyzer. The extension also looks in `~/.cargo/bin`; configure `gpuiRsx.serverPath` if
`cargo-rsx` is installed elsewhere. The server restarts when GPUI RSX settings change.
The extension accepts any server patch release with the same major.minor version.
Its install command selects the latest compatible patch from crates.io.

Build a VSIX from the repository:

```sh
cd editors/vscode
bun install --frozen-lockfile
bun run check
bun run package
code --install-extension gpui-rsx-0.10.2.vsix
```

VSIX packaging requires Node.js 22 or newer. The extension still supports VS Code
1.96; type checking uses that API version. The `minimatch` override selects a security
fix for the language client and packaging tools until their dependency ranges include it.
The checked-in Bun patch adapts the client's glob validation to that API; frozen
installs apply it automatically. `bun test` exercises the actual client matcher.

Formatting leaves commented macro bodies unchanged to preserve comments. Dynamic class
strings are diagnosed at runtime; class completion operates in static literals within
balanced RSX macro bodies. Static preview and JSX conversion are available through the
CLI and documentation Playground.

The server also completes tags, GPUI methods and props from `#[component]` declarations in
the current Rust file, and offers fixes for missing props/keys and mismatched tags. It registers
`rsx.toml` / `Cargo.toml` watches when the client supports dynamic file-watch registration.
Keep rust-analyzer enabled for imported components and Rust type checking.
