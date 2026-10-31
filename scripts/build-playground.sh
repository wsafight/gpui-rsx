#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
rsx_pack_mode=normal
if command -v wasm-bindgen >/dev/null && [[ "$(wasm-bindgen --version)" == "wasm-bindgen 0.2.130" ]]; then
    rsx_pack_mode=no-install
fi
RUSTUP_TOOLCHAIN=1.98.1 wasm-pack build --mode "$rsx_pack_mode" crates/gpui-rsx-wasm --target web --out-dir ../../docs/public/playground/pkg --release --no-pack
