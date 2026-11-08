#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
    echo 'Usage: install-ci-tool.sh TOOL VERSION INSTALL_ROOT' >&2
    exit 1
fi

tool=$1
version=$2
install_root=$3

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Expected an exact tool version, got: $version" >&2
    exit 1
fi

case "$tool" in
    bun) binary=bun; expected="$version" ;;
    binaryen) binary=wasm-opt; expected="wasm-opt version ${version%%.*} (version_${version%%.*})" ;;
    wasm-pack|cargo-audit) binary=$tool; expected="$tool $version" ;;
    wasm-bindgen-cli) binary=wasm-bindgen; expected="wasm-bindgen $version" ;;
    *) echo "Unsupported CI tool: $tool" >&2; exit 1 ;;
esac

if [[ ! -x "$install_root/bin/$binary" ]]; then
    case "$tool" in
        bun|binaryen)
            npm install --global --prefix "$install_root" "$tool@$version" \
                --registry=https://registry.npmjs.org \
                --fetch-retries=3 --fetch-retry-mintimeout=1000 \
                --fetch-retry-maxtimeout=10000 --fetch-timeout=60000 \
                --no-audit --no-fund
            ;;
        *)
            CARGO_NET_RETRY=3 CARGO_HTTP_TIMEOUT=60 \
                cargo install --locked --version "$version" --root "$install_root" "$tool"
            ;;
    esac
fi

actual=$("$install_root/bin/$binary" --version)
if [[ "$actual" != "$expected" ]]; then
    echo "CI tool version mismatch: expected '$expected', got '$actual'" >&2
    exit 1
fi
printf '%s\n' "$actual"
