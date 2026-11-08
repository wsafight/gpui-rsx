# 开发流程检查

这份记录对应工作流与回归保障的开发改动。宏生成 API 的完整适配目标继续使用
demo 锁定的 Kit 0.8.1；Kit 0.9.0 夹具只验证热补丁接入路径。

## 按改动运行 CI

`.github/scripts/ci-changes.cjs` 根据 PR 相对基准分支的文件差异选择检查。文档改动运行
文档与 Playground 检查；core 改动还验证原生 demo、工具和编辑器。未知路径采用完整
检查，main、每周定期和手动运行也执行完整检查。

Playground WASM 只构建一次，worker 测试通过后上传为 `playground-wasm`，供文档构建
下载。`docs.yml` 由 CI 调用，PR 只构建与测试，main 推送才部署 Pages。
`Required Checks` 汇总成功、跳过和失败状态；分类失败、任务失败或取消会使它失败。

```sh
node --test .github/scripts/*.test.cjs scripts/test-hot-reload.test.mjs scripts/test-readme.test.mjs
```

## 文档和原生界面回归

```sh
bun install --cwd docs --frozen-lockfile
bunx --cwd docs playwright install chromium
bun run --cwd docs build
bun run --cwd docs test:browser
cargo +1.98.1 test --manifest-path demo/Cargo.toml --bin incident_console --locked
```

浏览器检查覆盖中英文、明暗主题、375px 与 1280px 视口、导航、表格滚动、诊断定位和
分享恢复。首页截图、失败截图和 trace 保留为 CI 产物；当前没有像素差异基准。
测试使用独立预览端口，并绕过 Astro 的后台预览锁，便于与日常预览并行。
本地已有兼容 Chromium 时，可设置 `RSX_CHROMIUM_EXECUTABLE` 指定二进制。

控制台测试实际挂载 Kit 窗口并派发点击，检查筛选、详情页签、条目完成后的选择协调，
以及空队列恢复。`test_support` 在正式构建中保留原生元素类型，不额外插入布局节点。
README 的首个示例来自 `demo/src/bin/starter.rs`，CI 同时编译该二进制并检查双语文本
与源码一致。文本一致性检查每次都运行，单独修改 README 也能发现示例偏差。

## 热补丁验证

```sh
cargo run -p cargo-rsx -- doctor --manifest-path tests/fixtures/hot-reload/Cargo.toml --features hotreload,kit --hot-reload --json
node scripts/test-hot-reload.mjs
```

脚本要求可用的桌面会话和 Dioxus CLI 0.7.10；`RSX_DX` 可指定独立安装的二进制。
它修改临时副本，在原生 render 回调中检查 revision、PID 和状态，结束后停止进程。
脚本的进程退出、超时清理和失败记录保留另有测试。2026-11-08 的可见窗口验证中，
修改版本文字和面板颜色后 PID 不变，秒数从 27 继续到 35，点击数保留为 26。
脚本本身不派发点击，也不做像素差异比较。步骤、结果和限制见
[热更新夹具说明](../tests/fixtures/hot-reload/README.md)。

## 性能基准

CI 的宏展开与编辑器分析门禁分别使用 10% 和 20% 阈值，首次失败会重测两端。
分析覆盖 80/320/1280 个宏的增量更新和 5000 行文档。它们测量库内操作，不能替代
真实应用的编译测量。

```sh
node scripts/measure-builds.mjs
```

脚本使用 demo 的 Rust 工具链、Cargo.lock 与最小原生应用，在新 target 中依次构建、
重复构建并修改视图后重建。registry 依赖需事先缓存。每次运行创建独立目录，保留
三个 timing HTML、日志和 JSON；不会删除用户的构建缓存。比较结果时应保持环境一致，
也不要与其他构建并行运行。

2026-11-08，在本机 macOS arm64、Rust 1.98.1 上运行一次，Kit 0.8.1 的 starter 得到：

| 场景 | 耗时 |
| --- | --- |
| 空 target 首次构建 | 69.882 秒 |
| 无改动再次构建 | 0.767 秒 |
| 视图文字 Count 改为 Total 后重建 | 1.335 秒 |

运行时没有其他 Cargo 构建，热更新演示窗口保持运行。依赖源码已缓存；这里的首次构建
不含下载。测量使用本分支提交前的工作区，只运行一轮，没有与 main 做对照，不能据此
声称构建提速。原始结果保存在本地 `target/build-baselines/measurement-DpMOEs/`，
包含每阶段的 Cargo timing HTML、日志与 `results.json`；这些产物不提交到仓库。
