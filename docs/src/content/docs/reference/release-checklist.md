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

- set the version everywhere with `cargo xtask bump <version>`: workspace manifests and
  internal requirements, workspace entries in the root, demo, Zed, fuzz and test fixture
  lockfiles, the VS Code and Zed extension manifests, install snippets in the
  README and docs, and the compatibility matrix. CI runs `cargo xtask bump --check`,
- update `CHANGELOG.md` and `CHANGELOG_CN.md`, and mark breaking API changes,
- remove "unreleased" labels for shipped features: `rg -n -i 'unreleased|未发布' docs/src README*.md`,
- ensure the demo lockfile still reflects the intended Kit and GPUI snapshot versions,
- confirm GitHub Pages deployment succeeds after merge.

CI audits all four active Rust lockfiles and both Bun lockfiles. Before releasing,
run `cargo audit --file <lockfile>` for each Rust project and `bun audit --cwd docs`
and `bun audit --cwd editors/vscode` after updating dependencies.

## Tooling packages

Publish in dependency order: `gpui-rsx-core`, `gpui-rsx-tools`, `gpui-rsx`, `cargo-rsx`.
The release check script verifies all packages with local patches before registry publication.
Also run `node scripts/test-playground.mjs` after building WASM; build the VSIX and
`editors/zed` for `wasm32-wasip1`. Attach both editor packages to the GitHub release.
Track Marketplace authentication and Zed registry review separately from crate publication.

## 0.10 behavior checks

Verify numeric key identity, shared runtime styles, generic components/callbacks, and format
class evaluation/order against real GPUI (`demo/tests`). Verify LSP UTF-16 coalescing,
configuration watches, local prop completion and applicable fixes through CLI/LSP tests.
Run both `cargo test --workspace` and `cargo test --workspace --all-features`, plus tools
without default features. Regenerate intentional expansion/diagnostic snapshots and review
the diffs. Check generated specs and tables with `cargo xtask spec --check`,
`cargo xtask compat-matrix --check`, `cargo xtask gen-tables --check`, and require the GPUI
API snapshot with `GPUI_API_SNAPSHOT_REQUIRED=1 cargo test -p xtask --test gpui_api_snapshot`.

After building the docs/WASM, run `node scripts/test-playground.mjs` for themes, cache keys,
input limits and conversion scale. When a browser is available, check theme errors, share
restore, conversion and responsive layout interactively. Validate VS Code with `bun run check`,
`bun test` and `bun run build` from `editors/vscode`.
