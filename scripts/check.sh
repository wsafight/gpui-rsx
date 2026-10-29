#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: scripts/check.sh [--proxy] [--skip-demo] [--release] [--skip-docs] [--skip-package]

Runs the standard local validation suite:
  cargo fmt --all -- --check
  cargo xtask spec --check
  cargo xtask compat-matrix --check
  cargo xtask check-gpui
  cargo test --workspace
  cargo test --workspace --all-features
  cargo test -p gpui-rsx-tools --no-default-features
  cargo clippy -p gpui-rsx-tools --all-targets --no-default-features -- -D warnings
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  cargo +1.98.1 check --manifest-path demo/Cargo.toml --bins --locked
  cargo +1.98.1 clippy --manifest-path demo/Cargo.toml --all-targets --locked -- -D warnings
  cargo +1.98.1 test --manifest-path demo/Cargo.toml --lib --bins --tests --locked

With --release, also runs:
  cargo bench --bench class_performance --no-run
  cargo +1.98.1 tree --manifest-path demo/Cargo.toml --locked -i gpui-pre
  bun install --cwd docs --frozen-lockfile
  bun run --cwd docs check
  bun run --cwd docs build
  cargo publish --dry-run --allow-dirty (all four packages, local dependency patches)

Options:
  --proxy         Set http_proxy/https_proxy/all_proxy to 127.0.0.1:7890 for git/registry deps.
  --skip-demo     Skip demo manifest checks.
  --release       Run the extended pre-release validation suite.
  --skip-docs     Skip docs install/check/build in --release mode.
  --skip-package  Skip cargo publish dry-run in --release mode.
  -h, --help      Show this help.
USAGE
}

skip_demo=0
release=0
skip_docs=0
skip_package=0

while (($#)); do
    case "$1" in
        --proxy)
            export http_proxy="http://127.0.0.1:7890"
            export https_proxy="http://127.0.0.1:7890"
            export all_proxy="socks5://127.0.0.1:7890"
            ;;
        --skip-demo)
            skip_demo=1
            ;;
        --release)
            release=1
            ;;
        --skip-docs)
            skip_docs=1
            ;;
        --skip-package)
            skip_package=1
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "unknown option: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

run() {
    printf '\n+ %s\n' "$*"
    "$@"
}

run cargo fmt --all -- --check
run cargo xtask spec --check
run cargo xtask compat-matrix --check
run cargo xtask line-limit
run cargo xtask bump --check
run cargo test --workspace
run cargo test --workspace --all-features
run cargo test -p gpui-rsx-tools --no-default-features
run cargo clippy -p gpui-rsx-tools --all-targets --no-default-features -- -D warnings
run cargo clippy --workspace --all-targets --all-features -- -D warnings

if [[ "$skip_demo" -eq 0 ]]; then
    run cargo xtask check-gpui
    run cargo xtask gen-tables --check
    GPUI_API_SNAPSHOT_REQUIRED=1 run cargo test -p xtask --test gpui_api_snapshot
    run cargo +1.98.1 check --manifest-path demo/Cargo.toml --bins --locked
    run cargo +1.98.1 clippy --manifest-path demo/Cargo.toml --all-targets --locked -- -D warnings
    run cargo +1.98.1 test --manifest-path demo/Cargo.toml --lib --bins --tests --locked
fi

if [[ "$release" -eq 1 ]]; then
    run cargo bench --bench class_performance --no-run

    if [[ "$skip_demo" -eq 0 ]]; then
        run cargo +1.98.1 tree --manifest-path demo/Cargo.toml --locked -i gpui-pre
    fi

    if [[ "$skip_docs" -eq 0 ]]; then
        run bun install --cwd docs --frozen-lockfile
        run bun run --cwd docs check
        run bun run --cwd docs build
    fi

    if [[ "$skip_package" -eq 0 ]]; then
        rsx_release_root="$(pwd)"
        rsx_core_patch="patch.crates-io.gpui-rsx-core.path=\"$rsx_release_root/crates/gpui-rsx-core\""
        rsx_tools_patch="patch.crates-io.gpui-rsx-tools.path=\"$rsx_release_root/crates/gpui-rsx-tools\""
        run cargo publish -p gpui-rsx-core --dry-run --allow-dirty
        run cargo publish -p gpui-rsx-tools --dry-run --allow-dirty --config "$rsx_core_patch"
        run cargo publish -p gpui-rsx --dry-run --allow-dirty --config "$rsx_core_patch"
        run cargo publish -p cargo-rsx --dry-run --allow-dirty --config "$rsx_core_patch" --config "$rsx_tools_patch"
    fi
fi
