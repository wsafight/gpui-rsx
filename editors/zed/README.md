# GPUI RSX for Zed

Install the server with `cargo install cargo-rsx --version 0.8.1 --locked`. In Zed, run **zed: install dev
extension** and select this directory. The extension adds the GPUI RSX language server
to Rust alongside rust-analyzer. If you customize Rust language servers in settings,
include `"gpui-rsx"` and `"..."` in that list.

```json
{"languages":{"Rust":{"language_servers":["rust-analyzer","gpui-rsx","..."]}}}
```

To validate/build the extension manually:

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

Public Zed marketplace publication requires an entry in zed-industries/extensions and
upstream review. A development installation works before marketplace acceptance.
