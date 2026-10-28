---
title: Release Checklist
description: Commands and checks to run before releasing GPUI-RSX.
---

Run the extended pre-release validation suite:

```bash
scripts/check.sh --release
```

If registry downloads need the local proxy, use:

```bash
scripts/check.sh --proxy --release
```

The script runs root crate formatting, tests, clippy, and the real GPUI demo check/clippy/tests against
the pinned lockfile. In `--release` mode it also checks the benchmark target, confirms the demo uses
a single `gpui-pre` instance, builds the docs site, and runs a cargo publish dry-run.

## Package Contents

Before publishing, verify the crates.io package contains only the files needed to build and display
the crate documentation:

```bash
cargo package --list --allow-dirty
```

The package should include the manifest, license, README, `src/**`, and the benchmark target. It
should not include `demo/`, `docs/`, `.github/`, `tests/`, or local build output.

If the local environment cannot run docs or package validation, split those checks explicitly:

```bash
cargo bench --bench class_performance --no-run
cargo xtask check-gpui
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui-pre
cargo package --list --allow-dirty
bun install --cwd docs --frozen-lockfile
bun run --cwd docs check
bun run --cwd docs build
cargo publish --dry-run --allow-dirty
```

Before publishing:

- update `CHANGELOG.md` and `CHANGELOG_CN.md`,
- verify `Cargo.toml` version and README install snippets,
- ensure the demo lockfile still reflects the intended Kit and GPUI snapshot versions,
- confirm GitHub Pages deployment succeeds after merge.

## 0.9 tooling packages

Publish in dependency order: `gpui-rsx-core`, `gpui-rsx-tools`, `gpui-rsx`, `cargo-rsx`.
The release check script verifies all packages with local patches before registry publication.
Also run `node scripts/test-playground.mjs` after building WASM; build the VSIX and
`editors/zed` for `wasm32-wasip1`. Attach both editor packages to the GitHub release.
Track Marketplace authentication and Zed registry review separately from crate publication.
