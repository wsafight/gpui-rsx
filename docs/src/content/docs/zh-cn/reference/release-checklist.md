---
title: 发布检查清单
description: 发布 GPUI-RSX 前应运行的命令和检查项。
---

先运行 Rust 检查：

```bash
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

确认 class benchmark 目标仍能编译：

```bash
cargo bench --bench class_performance --no-run
```

用固定 lockfile 检查真实 GPUI demo：

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui
```

构建文档站：

```bash
cd docs
pnpm install --frozen-lockfile
pnpm run build
```

发布前还需要：

- 更新 `CHANGELOG.md` 和 `CHANGELOG_CN.md`；
- 确认 `Cargo.toml` 版本和 README 安装片段；
- 确认 demo lockfile 仍指向预期 GPUI revision；
- 合并后确认 GitHub Pages 部署成功。
