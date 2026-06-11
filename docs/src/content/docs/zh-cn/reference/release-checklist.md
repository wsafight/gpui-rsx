---
title: 发布检查清单
description: 发布 GPUI-RSX 前应运行的命令和检查项。
---

先运行扩展发布前验证：

```bash
scripts/check.sh --release
```

如果 Zed GPUI git 依赖需要本地代理，使用：

```bash
scripts/check.sh --proxy --release
```

该脚本会运行根 crate 格式、测试、clippy，以及基于固定 lockfile 的真实 GPUI demo
check/clippy。`--release` 模式还会检查 benchmark 目标、确认 demo 中只有一个 `gpui`
实例、构建文档站，并执行 cargo publish dry-run。

如果本地环境不能跑 docs 或 package 验证，可以显式拆开执行：

```bash
cargo bench --bench class_performance --no-run
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui
pnpm --dir docs install --frozen-lockfile
pnpm --dir docs run check
pnpm --dir docs run build
cargo publish --dry-run --allow-dirty
```

发布前还需要：

- 更新 `CHANGELOG.md` 和 `CHANGELOG_CN.md`；
- 确认 `Cargo.toml` 版本和 README 安装片段；
- 确认 demo lockfile 仍指向预期 GPUI revision；
- 合并后确认 GitHub Pages 部署成功。
