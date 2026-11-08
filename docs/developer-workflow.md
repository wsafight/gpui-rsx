# 开发流程检查

这份记录对应工作流与回归保障的开发改动。宏生成 API 的完整适配目标继续使用
demo 锁定的 Kit 0.9.0 / GPUI Pre 0.3.11；独立热更新夹具验证同一版本的热补丁接入路径。

## 按改动运行 CI

`.github/scripts/ci-changes.cjs` 根据 PR 相对基准分支的文件差异选择检查。文档改动运行
文档与 Playground 检查；core 改动还验证原生 demo、工具和编辑器。未知路径采用完整
检查，main、每周定期和手动运行也执行完整检查。

Playground WASM 只构建一次，worker 测试通过后上传为 `playground-wasm`，供文档构建
下载。`docs.yml` 由 CI 调用，PR 只构建与测试，main 推送才部署 Pages。
`Required Checks` 汇总成功、跳过和失败状态；分类失败、任务失败或取消会使它失败。

```sh
node --test .github/scripts/*.test.cjs scripts/test-hot-reload.test.mjs scripts/test-readme.test.mjs scripts/measure-native.test.mjs
```

## 文档和原生界面回归

```sh
bun install --cwd docs --frozen-lockfile
bunx --cwd docs playwright install chromium
bun run --cwd docs build
bun run --cwd docs test:browser
cargo +1.98.1 test --manifest-path demo/Cargo.toml --bin incident_console --test native_controls --all-features --locked
```

浏览器检查覆盖中英文、明暗主题、375px 与 1280px 视口、导航、表格滚动、诊断定位和
分享恢复。首页截图、失败截图和 trace 保留为 CI 产物；当前没有像素差异基准。
测试使用独立预览端口，并绕过 Astro 的后台预览锁，便于与日常预览并行。
本地已有兼容 Chromium 时，可设置 `RSX_CHROMIUM_EXECUTABLE` 指定二进制。

控制台测试实际挂载 Kit 窗口并派发点击，检查筛选、详情页签、条目完成后的选择协调、
空队列恢复，以及 1千、1万、10万条数据的虚拟滚动和选择保持。
`demo/tests/native_controls.rs` 另行检查 Kit 输入框的 Unicode 输入、组合字符退格、
全选替换，以及多次界面更新后的文本、焦点和单次点击回调。退格断言遵循 Kit 当前
逐个删除组合标记的行为，不假定按完整字素簇删除。这些是无头窗口中的输入事件，
不覆盖操作系统输入法、物理键盘或实际屏幕呈现。`test_support` 在正式构建中保留
原生元素类型，不额外插入布局节点。
README 的首个示例来自 `demo/src/bin/starter.rs`，CI 同时编译该二进制并检查双语文本
与源码一致。文本一致性检查每次都运行，单独修改 README 也能发现示例偏差。

## 边界与失败路径回归

```sh
cargo test -p gpui-rsx-tools --no-default-features --lib --tests
cargo test -p cargo-rsx --test recovery
```

- 宏发现覆盖导入别名、作用域、原始标识符、字符串与嵌套注释恢复，以及恢复预算耗尽。
  Unicode 文档按每个 UTF-8 边界逐步输入和删除，逐步对比增量分析与全量分析的结果。
- 同名 `macro_rules!` 的测试覆盖定义前后、嵌套块和模块、递归调用、Unicode 与原始
  标识符，以及增量添加、删除与恢复。CLI 写入前后逐字节对比自定义宏源码；LSP
  检查遮蔽后的诊断清除、空格式化结果和解除遮蔽后的悬停恢复。
- CLI 在不完整源码中允许只读预览，但拒绝格式化与迁移写入；同项目内完好的文件也
  必须保持原样。真实 LSP 消息验证中文与 emoji 后的 UTF-16 定位、增量修复、诊断清除
  和别名模式切换。
- 控制台将全部筛选与排序组合对照不使用缓存的结果，并在连续修改后检查旧快照不变、
  ID 索引和选择一致。无头窗口实际点击 10万条数据的末行，再切换数据量、清空与恢复。
- 可选采样器的测试随 `--all-features` 运行，验证 10 帧预热、60 帧采样、回调去重、
  分位数、基线扣除、计数下溢和写入失败。
- 两个原生验证脚本覆盖分块 UTF-8 日志、不完整 render 行、陈旧报告、进程提前退出、
  超时与信号清理。热补丁成功还要求 PID 和状态连续，不能只匹配到新的版本文字。

## 热补丁验证

```sh
cargo run -p cargo-rsx -- doctor --manifest-path tests/fixtures/hot-reload/Cargo.toml --features hotreload,kit --hot-reload --json
node scripts/test-hot-reload.mjs
```

脚本要求可用的桌面会话和 Dioxus CLI 0.7.10；`RSX_DX` 可指定独立安装的二进制。
它修改临时副本，在原生 render 回调中检查 revision、PID 和状态，结束后停止进程。
报告记录源码写入到观察到 render 日志的耗时及轮询间隔，不代表屏幕呈现延迟。
构建失败、进程退出、超时清理和失败记录保留另有测试。2026-11-08 的可见窗口验证中，
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

## 原生大数据采样

```sh
node scripts/measure-native.mjs
node scripts/measure-native.mjs --release
```

脚本使用 Kit 0.9.0、Rust 1.98.1 和真实窗口，依次采样 1千、1万、10万条数据。
每组先预热 10 帧，再记录 60 帧滚动和选择。过滤、排序及首次数据加载不在这段
稳态采样内。运行时保持窗口可见；缺少呈现样本会使验证失败，而不是当作零耗时。
原始数据和日志保存在 `target/native-baselines/`。这些 CPU 侧数据不包含 GPU 完成、
屏幕扫描、真实输入延迟、分配次数或内存占用，不能换算成承诺的 FPS。

2026-11-08，在 Apple M3 Pro、macOS Darwin 25.6.0 上进行 debug 采样，结果如下：

| 数据量 | CPU draw p95 | 呈现间隔 p95 | 每帧最多构建行数 |
| --- | --- | --- | --- |
| 1千条 | 12.952 ms | 17.269 ms | 10 |
| 1万条 | 13.763 ms | 18.104 ms | 10 |
| 10万条 | 14.909 ms | 18.858 ms | 10 |

每组均取得 60 个呈现样本。原始结果在本地
`target/native-baselines/2026-11-08T07-01-14.045Z/`。测量时没有其他 Cargo 构建，
但桌面仍有其他应用运行；这是单轮观测，不是与 main 的受控对照。此前一次采样中，
10万条窗口没有呈现间隔样本，脚本正确拒绝了该轮结果；失败数据仍保留在本地。

同日的分析基准冒烟测试使用 10 个样本、1 秒预热和 1 秒测量。80 个宏的全量分析约
1.282 ms，单宏编辑约 41.8 µs，1280 个宏的增量更新约 509.3 µs。5000 行文档的
增量更新约 103.5 µs，比既有本机基准慢约 7%，未超过 20% 门禁；不能据此声称分析
全面提速。别名作用域和不完整源码恢复以正确性为先，包含宏别名或 `macro_rules!`
定义的文档暂时重新发现整份文件的宏，再复用未变宏体的分析结果。
