---
title: Release Checklist
description: Commands and checks to run before releasing GPUI-RSX.
---

Run the Rust checks first:

```bash
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Check the class benchmark target still compiles:

```bash
cargo bench --bench class_performance --no-run
```

Check the real GPUI demo against the pinned lockfile:

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui
```

Build the documentation site:

```bash
cd docs
pnpm install --frozen-lockfile
pnpm run build
```

Before publishing:

- update `CHANGELOG.md` and `CHANGELOG_CN.md`,
- verify `Cargo.toml` version and README install snippets,
- ensure the demo lockfile still reflects the intended GPUI revision,
- confirm GitHub Pages deployment succeeds after merge.
