---
title: 发布检查清单
description: 发布 GPUI-RSX 前应运行的命令和检查项。
---

先运行扩展发布前验证：

```bash
scripts/check.sh --release
```

如果 registry 下载需要本地代理，使用：

```bash
scripts/check.sh --proxy --release
```

该脚本会运行根 crate 格式、测试、clippy，以及基于固定 lockfile 的真实 GPUI demo
check/clippy/test。`--release` 模式还会检查 benchmark 目标、确认 demo 中只有一个 `gpui-pre`
实例、构建文档站，并执行 cargo publish dry-run。

## 发布包内容

发布前确认 crates.io 包只包含构建 crate 和展示文档所需文件：

```bash
cargo package --list --allow-dirty
```

发布包应包含 manifest、license、README、`src/**` 和 benchmark 目标；不应包含 `demo/`、
`docs/`、`.github/`、`tests/` 或本地构建产物。

如果本地环境不能跑 docs 或 package 验证，可以显式拆开执行：

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

发布前还需要：

- 用 `cargo xtask bump <version>` 统一设置版本：workspace manifest 与内部依赖要求、
  `Cargo.lock`、`demo/Cargo.lock`、`editors/zed/Cargo.lock` 中的 workspace 条目、VS Code 与 Zed
  插件 manifest、README 和文档中的安装片段，以及兼容性矩阵。CI 会运行 `cargo xtask bump --check`；
- 更新 `CHANGELOG.md` 和 `CHANGELOG_CN.md`，并标注破坏性 API 变更；
- 去掉已发布功能的“未发布”标记：`rg -n -i 'unreleased|未发布' docs/src README*.md`；
- 确认 demo lockfile 仍指向预期的 Kit 与 GPUI 快照版本；
- 合并后确认 GitHub Pages 部署成功。

## 工具包

按依赖顺序发布：`gpui-rsx-core`、`gpui-rsx-tools`、`gpui-rsx`、`cargo-rsx`。
发布检查脚本会在发布到 registry 之前，用本地 patch 验证全部包。
构建 WASM 后还需运行 `node scripts/test-playground.mjs`；构建 VSIX，并以 `wasm32-wasip1`
构建 `editors/zed`，把两个编辑器插件包附加到 GitHub Release。Marketplace 认证和 Zed
registry 审核与 crate 发布分开跟进。

## 0.10 行为检查

使用 `demo/tests` 在真实 GPUI 上验证整数 key 身份、共享 runtime 样式、泛型组件 / 回调、
format class 的求值与覆盖顺序。通过 CLI / LSP 测试验证 UTF-16 修改合并、配置监听、
本文件 prop 补全与可应用的修复。运行默认与全 feature 的 workspace 测试，以及 tools
无默认 feature 测试。仅对有意变化更新展开与诊断快照，并检查差异。
运行 `cargo xtask spec --check`、`cargo xtask compat-matrix --check`、
`cargo xtask gen-tables --check`，并使用
`GPUI_API_SNAPSHOT_REQUIRED=1 cargo test -p xtask --test gpui_api_snapshot` 要求 GPUI API 快照通过。

构建文档 / WASM 后，运行 `node scripts/test-playground.mjs` 检查主题、缓存键、输入限制与
转换刻度。有可用浏览器时交互验证主题错误、分享恢复、转换与响应式布局。在
`editors/vscode` 中运行 `bun run check`、`bun test` 与 `bun run build`。
