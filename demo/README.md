# GPUI-RSX Demos

The demos use GPUI Kit 0.8.0 and its pinned `gpui-pre 0.3.9` snapshot:

```toml
gpui = { package = "gpui-kit", version = "=0.8.0" }
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
```

`project_dashboard` demonstrates keyed lists, selection, conditional classes,
progress, density controls and item mutations. `incident_console` combines
filters, sorting, incident lifecycle transitions, a timeline and empty states;
its model and UI modules live under `src/bin/incident_console/`.

`api_contract`, `generated_api_contract` and `semantics_contract` are compile-only
contracts. They cover GPUI methods, every generated theme color, the Kit alias,
component props, state variants, spacing scales and dynamic classes. `rsx.toml`
enables semantic colors; the preset reads `cx.theme().colors.*` so a color such
as `list` remains distinct from `Theme.list` settings.

From the repository root, verify or update the matching dependency set:

```bash
cargo xtask check-gpui
cargo xtask gen-tables --check
cargo +1.98.1 check --manifest-path demo/Cargo.toml --bins --locked
cargo +1.98.1 clippy --manifest-path demo/Cargo.toml --bins --locked -- -D warnings
cargo +1.98.1 test --manifest-path demo/Cargo.toml --bins --locked
```

The compatibility workflow follows stable, non-yanked Kit releases and verifies
the matching snapshot before proposing an update. GPUI Fast is not enabled.
