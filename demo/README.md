# GPUI-RSX Demos

The demos use GPUI Kit 0.8.1 and its pinned `gpui-pre 0.3.10` snapshot:

```toml
gpui = { package = "gpui-kit", version = "=0.8.1" }
gpui-rsx = { path = ".." }
```

The Cargo alias keeps `gpui::` available to RSX expansions and GPUI derives.
Kit supplies the matching engine, components, platform entry point and assets.
Import components through `gpui::component`; initialize with `gpui::init(cx)`
and open windows with `gpui::open_window(...)` to mount Kit's root.

The demo toolchain is pinned to Rust 1.98.1. The macro/core MSRV remains Rust 1.88.
Commit `Cargo.lock` to reproduce the complete dependency graph.

On Debian/Ubuntu, install the native GPUI dependencies before building the demos:

```bash
sudo apt-get update
sudo apt-get install -y libfontconfig1-dev libwayland-dev libx11-dev libxcb1-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libxkbcommon-x11-dev
```

```bash
cd demo
cargo run --bin hello --locked
cargo run --bin counter --locked
cargo run --bin palette --locked
cargo run --bin task_list --locked
cargo run --bin project_dashboard --locked
cargo run --bin incident_console --locked
cargo run --bin api_surface --locked
cargo run --bin component --locked
cargo run --bin syntax_gallery --locked
```

`project_dashboard` demonstrates keyed lists, selection, conditional classes,
progress, density controls and item mutations. `incident_console` combines
filters, sorting, incident lifecycle transitions, a timeline and empty states;
its model and UI modules live under `src/bin/incident_console/`.

`syntax_gallery` is an interactive reference for the RSX grammar. Toggle conditions,
click the event example, reverse keyed rows, and inspect function/struct components,
fragments, spread children, nested loops, state classes, themes, SVG, images and canvas.
Its view lives in `src/syntax/` and is also rendered by the automated tests.

Run the demo tests without opening a desktop window:

```bash
cargo +1.98.1 test --manifest-path demo/Cargo.toml --lib --bins --tests --locked
cargo +1.98.1 test --manifest-path demo/Cargo.toml --doc --locked
```

From inside `demo/`, omit `--manifest-path demo/Cargo.toml`. These tests use Kit's
`test-support` feature only for development; regular builds keep the normal backend.

| Syntax / behavior | Real GPUI coverage |
| --- | --- |
| HTML aliases, paired/self-closing tags, flags, values and camelCase aliases | `tests/syntax_styles.rs` |
| `styled`, conditional class expressions, `when`, `whenSome`, `whenClass`, visibility | Refinement assertions plus rendered bounds in `tests/ui_syntax.rs` |
| `rsx!`, `rsx_strict!`, `rsx_permissive!`, `rsx_tw!`, `rsx_expand!` | `tests/syntax_styles.rs` |
| `rsx_runtime!`, default and explicit GPUI paths, both spacing scales | `tests/shared_runtime.rs`, `tests/runtime_paths.rs` |
| Static/dynamic colors, lengths, fractions, grids, theme tokens and unknown handlers | `tests/syntax_styles.rs`, `tests/shared_runtime.rs` |
| Function/struct props, defaults, boolean setters, `Option`, callbacks, generics, const parameters and children | `tests/syntax_components.rs`, `tests/component_props.rs` |
| Render-time `Window`/`App` injection, fragments, spread and nested loop order | `tests/ui_syntax.rs` |
| Fragment construction order, shared mutable state and generated binding hygiene | `tests/syntax_styles.rs` |
| Native/path/base constructors, third-party builders, auto IDs and explicit IDs/keys | `tests/auto_id.rs`, `tests/key_identity.rs` |
| Click/mouse/keyboard callbacks, hover styles and keyed row reordering | Native event dispatch in `tests/ui_syntax.rs` |
| SVG, both image source spellings, grayscale and canvas | `tests/syntax_styles.rs`; canvas prepaint/paint assertions in `tests/ui_syntax.rs` |
| Invalid syntax, missing props/IDs, invalid values and unsupported annotations | Compiler rejection tests in `src/syntax/rejected.rs` (`--doc`) |

The headless tests check layout, identity, evaluation order and callbacks; they do
not compare rendered pixels. Static fragments add no wrapper element. Directly
nested iterators are collected once per outer iteration so inner closures can
safely borrow the outer loop's pattern and the caller's state. Root fragments and
mixed loop bodies consume each loop or spread before constructing the next sibling,
preserving source order and releasing mutable borrows between siblings.

`api_contract`, `generated_api_contract` and `semantics_contract` are compile-only
contracts. They cover GPUI methods, every generated theme color, the Kit alias,
component props, state variants, spacing scales and dynamic classes. `rsx.toml`
enables semantic colors; the preset reads `cx.theme().colors.*` so a color such
as `list` remains distinct from `Theme.list` settings.

From the repository root, verify or update the matching dependency set:

```bash
cargo xtask check-gpui
cargo run --quiet -p cargo-rsx -- doctor --manifest-path demo/Cargo.toml
cargo xtask gen-tables --check
cargo +1.98.1 check --manifest-path demo/Cargo.toml --bins --locked
cargo +1.98.1 clippy --manifest-path demo/Cargo.toml --all-targets --locked -- -D warnings
cargo +1.98.1 test --manifest-path demo/Cargo.toml --lib --bins --tests --locked
cargo +1.98.1 test --manifest-path demo/Cargo.toml --doc --locked
```

The compatibility workflow follows stable, non-yanked Kit releases and verifies
the matching snapshot before proposing an update. GPUI Fast is not enabled.
