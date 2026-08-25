# GPUI-RSX Demos

These demos target GPUI from the Zed repository:

```toml
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit", "runtime_shaders", "wayland", "x11"] }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }
gpui-rsx = { path = ".." }
```

`gpui` and `gpui_platform` intentionally use the same unqualified Zed git source as `gpui-component` so all components share one GPUI type universe. `demo/Cargo.lock` pins the exact resolved Zed commit; do not add a direct `rev` to only the top-level GPUI dependencies unless `gpui-component` is moved to the same source.

The resolved Zed dependency requires Rust 1.95.0 or newer. This demo pins its toolchain to Rust 1.95.0 so local checks and CI use the same compiler.

```bash
cd demo
cargo run --bin hello
cargo run --bin counter
cargo run --bin palette
cargo run --bin task_list
cargo run --bin project_dashboard
cargo run --bin api_surface
cargo run --bin component
```

`api_contract` is a compile-only binary for API compatibility checks:

```bash
cd demo
cargo check --bin api_contract
```

`project_dashboard` is the most complete interactive example. It includes filtered keyed lists, selection state, conditional classes, dynamic progress, density controls, and item mutations.

The committed `Cargo.lock` and Rust toolchain pin the resolved `gpui-component` and Zed revisions so demo builds are reproducible. When updating either git dependency, regenerate `Cargo.lock`, check the generated GPUI APIs, and run:

```bash
cd demo
cargo check --bins
```
