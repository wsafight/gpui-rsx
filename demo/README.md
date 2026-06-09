# GPUI-RSX Demos

These demos target GPUI from the Zed repository:

```toml
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit", "runtime_shaders", "wayland", "x11"] }
gpui-component = { git = "https://github.com/longbridge/gpui-component", rev = "8752104289424b7f35045b68a2d394018da48e7e" }
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
cargo run --bin api_surface
cargo run --bin component
```

The `gpui-component` rev, `Cargo.lock`, and Rust toolchain are pinned so demo builds are reproducible. When updating `gpui-component` or the resolved Zed revision, regenerate `Cargo.lock`, check the generated GPUI APIs, and run:

```bash
cd demo
cargo check --bins
```
