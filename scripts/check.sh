#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: scripts/check.sh [--proxy] [--skip-demo] [--release] [--skip-docs] [--skip-package]

Runs the standard local validation suite:
  cargo fmt --all -- --check
  cargo test --all-features
  cargo clippy --all-targets --all-features -- -D warnings
  cargo +1.95.0 check --manifest-path demo/Cargo.toml --bins --locked
  cargo +1.95.0 clippy --manifest-path demo/Cargo.toml --bins --locked -- -D warnings

With --release, also runs:
  cargo bench --bench class_performance --no-run
  cargo +1.95.0 tree --manifest-path demo/Cargo.toml --locked -i gpui
  pnpm --dir docs install --frozen-lockfile
  pnpm --dir docs run check
  pnpm --dir docs run build
  cargo publish --dry-run --allow-dirty

Options:
  --proxy         Set http_proxy/https_proxy/all_proxy to 127.0.0.1:7890 for git/npm deps.
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

# Keep release checks deterministic and avoid pnpm's background update check from
# emitting network errors when the local proxy is unavailable.
export npm_config_update_notifier=false
export NO_UPDATE_NOTIFIER=1

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
run cargo test --all-features
run cargo clippy --all-targets --all-features -- -D warnings

if [[ "$skip_demo" -eq 0 ]]; then
    run cargo +1.95.0 check --manifest-path demo/Cargo.toml --bins --locked
    run cargo +1.95.0 clippy --manifest-path demo/Cargo.toml --bins --locked -- -D warnings
fi

if [[ "$release" -eq 1 ]]; then
    run cargo bench --bench class_performance --no-run

    if [[ "$skip_demo" -eq 0 ]]; then
        run cargo +1.95.0 tree --manifest-path demo/Cargo.toml --locked -i gpui
    fi

    if [[ "$skip_docs" -eq 0 ]]; then
        run pnpm --dir docs install --frozen-lockfile
        run pnpm --dir docs run check
        run pnpm --dir docs run build
    fi

    if [[ "$skip_package" -eq 0 ]]; then
        run cargo publish --dry-run --allow-dirty
    fi
fi
