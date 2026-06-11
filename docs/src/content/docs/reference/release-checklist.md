---
title: Release Checklist
description: Commands and checks to run before releasing GPUI-RSX.
---

Run the extended pre-release validation suite:

```bash
scripts/check.sh --release
```

If the Zed GPUI git dependency needs the local proxy, use:

```bash
scripts/check.sh --proxy --release
```

The script runs root crate formatting, tests, clippy, and the real GPUI demo check/clippy against
the pinned lockfile. In `--release` mode it also checks the benchmark target, confirms the demo uses
a single `gpui` instance, builds the docs site, and runs a cargo publish dry-run.

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
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui
cargo package --list --allow-dirty
pnpm --dir docs install --frozen-lockfile
pnpm --dir docs run check
pnpm --dir docs run build
cargo publish --dry-run --allow-dirty
```

Before publishing:

- update `CHANGELOG.md` and `CHANGELOG_CN.md`,
- verify `Cargo.toml` version and README install snippets,
- ensure the demo lockfile still reflects the intended GPUI revision,
- confirm GitHub Pages deployment succeeds after merge.
