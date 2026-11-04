# 开发验收与 P2 研究记录

记录日期：2026-11-03，收尾更新：2026-11-04（Asia/Shanghai）。原始开发基线为 `5183749`，
提交前审查基于 `630e5a5`；后续变更尚未提交。
本记录描述本地验收与研究结论，不替代发布支持矩阵，也不表示远程 CI 已运行。

## 提交前审查修复（2026-11-04）

- 路径配置 fixture 改用包含测试标签和原子序号的目录名，以原子创建和重试排除已有目录。
  新增八个线程创建共 128 个项目的回归，检查目录唯一和源码互不覆盖；不再依赖时间戳精度。
- 负尺寸提示检查解析后的长度，覆盖任意负值、双重负号、零值、状态变体和两种刻度。
  正值与零值回归启用 `deny(deprecated)`；动态 matcher 只生成样式调用，避免带入静态警告。
- GPUI 路径在每次展开内部按需解析并共享给子节点、循环和自动 ID，最多解析一次。
  缓存只存在于该次展开，不在全局主题缓存中保存跨宏调用的 token 或 span。

## P0 与 P1 的交付范围

本记录中的 P0/P1/P2 编号用于追溯原始评估，不代表公开 API 或发布级别。

| 任务 | 结果与证据 |
| --- | --- |
| P0-01 接入阻断 | 锁定 Kit 的 demo 构建、测试、doctest 与官方 GPUI 0.2.2 定向 fixture 通过；未复现新的运行时接入阻断 |
| P0-02 基础说明 | 中英文 README、入门、语法及工具指引明确标签别名、刻度、动态 class 与预览边界；六段入门/组件示例通过真实 Kit 编译 |
| P0-03 兼容范围 | 文档区分 Kit 完整映射目标、官方 GPUI 定向验证和未验证组合；生成矩阵与依赖图检查保持一致 |
| P0-04 ID | 修正“循环必须有 key”为“key 或显式 id”；真实输入测试验证显式 ID 的列表重排与点击目标 |
| P1-01 诊断 | 复现两个兄弟节点缺 key 时只报告第一处；生成器聚合独立错误，工具保留不同位置的同文错误，编译失败快照与 UTF-16 快速修复回归覆盖两处错误；真实 VS Code 扩展宿主验证与 rust-analyzer 共存 |
| P1-02 用法 | 双语增加构造方式和模式决策表；四段组件默认值/children/混合 Fragment 原文示例通过真实 Kit 编译 |
| P1-03 验证补缺 | 增加真实 GPUI 焦点进入和离开时的布局断言；保留已有 API 合约、点击、键盘、hover、循环及组件验证 |
| P1-04 维护基线 | 保留既有 spec/方法表生成流程，新增本记录、改名依赖 fixture 的 CI 与本地检查入口，并同步双语 changelog |

兄弟节点错误聚合不改变成功展开的 builder 顺序。父节点本身无法构造时，仍可能先报告父节点错误；本轮不承诺一次穷举无效程序中的所有错误。

## 环境和验证边界

- 本机：macOS 26.6.2 / arm64；workspace 使用 Rust 1.99.1，demo 使用 Rust 1.98.1。
- 主要运行目标：Kit 0.8.1 / gpui-pre 0.3.10 / gpui-component 0.8.1。
- 官方 GPUI 0.2.2 编译并运行共用的反馈、身份标识和状态更新回归，不代表完整方法表适配。
- 无窗口交互使用 GPUI 测试平台；桌面窗口、辅助功能、IME 和各系统渲染仍需单独人工验收。
- 编辑器验收使用官方 VS Code 1.96.4 / rust-analyzer 0.3.2887 的 macOS arm64 发行包；这是扩展声明的最低 VS Code 系列验证，未据此声明其他编辑器或所有版本均已验收。

可复现的主要命令：

```bash
cargo test --workspace --locked --offline
cargo test --workspace --all-features --locked --offline
cargo test -p gpui-rsx-tools --no-default-features --offline
cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
cargo +1.98.1 test --manifest-path demo/Cargo.toml --lib --bins --tests --locked --offline
cargo +1.98.1 test --manifest-path demo/Cargo.toml --doc --locked --offline
cargo +1.98.1 test --manifest-path tests/fixtures/gpui-0.2.2/Cargo.toml --offline
cargo +1.98.1 test --manifest-path tests/fixtures/gpui-renamed/Cargo.toml --target-dir demo/target --offline
cargo xtask spec --check
cargo xtask compat-matrix --check
cargo xtask gen-tables --check
cargo xtask check-gpui
cargo xtask line-limit
bun run --cwd docs check
bun run --cwd docs build
cargo build -p cargo-rsx --locked
bun run --cwd editors/vscode test:integration
node --test scripts/dev-reload*.test.mjs
```

本轮最终结果：

| 检查 | 结果 |
| --- | --- |
| workspace 默认 / 全 feature 测试 | 通过；新增路径和状态辅助回归已纳入 |
| tools 无默认 feature 测试及 Clippy | 通过 |
| workspace 全 feature、demo 全 target、改名 fixture 全 target Clippy | 通过，使用 `-D warnings` |
| demo 测试 / doctest | 通过，含 12 项语法交互测试、公开事件辅助宏的交互回归与 18 项编译失败 doctest |
| 改名 fixture / 官方 GPUI 0.2.2 fixture | 前者三项运行测试通过且无需 `gpui` 别名，包含泛型和弃用组件；后者运行反馈、身份标识、状态更新测试通过 |
| spec、兼容矩阵、方法表、API 快照、依赖图、行数、版本一致性 | 通过；API 快照启用 `GPUI_API_SNAPSHOT_REQUIRED=1` |
| 根 crate Rust 1.88 MSRV / Rust 格式 | 通过；demo 与新 fixture 检查本轮修改文件 |
| 文档检查、WASM/站点构建、站点链接 | 2026-11-04 复核通过；36 页、1383 处内部链接和资源 |
| 兼容自动化脚本 / 重载原型 / 本地检查脚本语法 | 分别 5 / 17 项 Node 测试通过；`bash -n scripts/check.sh` 通过 |
| VS Code 类型检查 / 单元测试 / 真实宿主测试 / 打包 | 通过；5 项单元测试、独立配置下的 rust-analyzer 共存/格式化切换/重启及 VSIX 打包；新增依赖的 Bun 审计通过 |
| Zed WASM 产物 | `wasm32-wasip1` release 构建通过；不等于实际编辑器宿主验收 |

2026-11-03 的一次本地 workspace 全 feature 测试通过，用时 121.15 秒。该值包含已有构建缓存、编译及测试进程，期间存在其他验证任务；它是开发验收成本记录，不是冷构建或渲染性能基准。预览服务测试需要监听本地端口，限制本地端口的沙箱不适合运行这组测试。

额外扫描发现 `CONTRIBUTING.md` 原有的 `CODE_OF_CONDUCT.md` 链接指向缺失文件。
2026-11-04 文档复核已移除这两处失效链接，并在贡献指南中直接说明讨论要求。
仓库 Markdown 相对文件链接与站点页面、锚点和资源链接分别检查。

2026-11-04 文档复核扫描了仓库 62 份 Markdown/MDX，补齐中英文未发布日志及 API 索引，
统一状态通知、负尺寸警告、循环 ID、配置路径和验证范围的说明。
workspace 的 38 项 Rustdoc 测试与新构建的 Playground WASM/worker 检查通过。
本轮文档修改没有改变运行时代码；默认/全 feature workspace 与 Clippy 结果沿用前一轮
已通过的回归。重载套件曾在并行重负载下发生一次等待超时，隔离和整套重跑均通过。

### 编辑器共存验收

测试关闭 rust-analyzer 的过程宏展开和保存时检查，验证的是签名 stub 项目中的语言服务
共存及格式化切换；不代表完整应用中两项扩展同时执行过程宏和检查的验收。

[隔离运行器](../editors/vscode/scripts/test-integration.mjs) 使用临时 workspace、扩展目录和用户配置；成功后删除临时配置，失败时保留日志位置。实际验收确认两项扩展均激活，RSX 诊断/快速修复/静态 class 补全与 Rust 原生 hover/方法补全同时可用。格式化默认关闭时使用 rust-analyzer；显式开启 GPUI RSX 后只格式化 RSX，关闭并重启后可恢复 Rust 格式化。

本机同时验证了复用已有 rust-analyzer 发行包和自动安装固定版本的路径；测试宿主正常退出，临时配置清理成功。清理对后台构建结束前的短暂目录写入进行有限重试。

宿主测试使用本地签名 stub 创建小型 Rust 项目；事件、焦点、循环等运行时行为仍由真实 Kit 测试验证。测试接入现有 tooling CI，Linux 使用 `xvfb-run`；本机实际运行结果仅覆盖 macOS arm64，CI 结果需在提交后由对应环境确认。Zed 与完整桌面交互人工验收仍未执行。

## P2：实现与研究结论

### P2-01 GPUI 路径配置

新增项目级 `[options] gpui-path = "::engine"`，也支持 `crate::ui` 根部重新导出。
默认路径仍为 `::gpui`。配置只接受绝对路径或 crate 根路径，不接受泛型参数、`self` 或 `super`。
已审计并覆盖普通/擦除根节点、空循环、自动 ID、动态 key、组件 builder/渲染、事件/状态辅助和动态运行时。

[改名依赖 fixture](../tests/fixtures/gpui-renamed/src/lib.rs) 只声明 `engine = { package = "gpui-kit", version = "=0.8.1" }`，
通过自己的 `rsx.toml` 设定路径，无需 `extern crate engine as gpui;`。
测试覆盖组件默认属性与 children、自动 ID、动态字符串 key、混合 Fragment、空循环、事件/状态辅助、
内联动态 class，以及默认和显式路径的两种共享运行时刻度。

显式 `rsx_runtime!(::other_engine)` 优先于配置。用户表达式、prop 类型与上游 derive 不被重写；
手写 `#[derive(engine::IntoElement)]` 仍可能需要 `gpui` 别名。函数组件在自定义路径下直接实现
`IntoElement`，避免调用上游硬编码路径的 derive。双语兼容文档说明上述边界，生成 spec 与提示词已同步。

新增[crate 根重新导出 fixture](../tests/fixtures/gpui-reexport/src/lib.rs)，通过替身验证
library、binary、example、integration-test 各 target 的 `crate::ui` 路径和共享运行时。
[真实 Cargo 重编译测试](../tests/configured_path_tests.rs) 分别只使用标记、组件、监听器、
更新宏或默认运行时，保持源码不变，仅修改 `rsx.toml`，检查有效路径→无效路径→恢复的行为；
另行验证显式运行时路径覆盖配置。已有配置的修改仍独立验证宏生成的依赖追踪，不依赖构建脚本。
新增生命周期回归直接复用 [demo 构建脚本](../demo/build.rs)，覆盖上述五类入口在配置缺失时编译、
首次新增无效配置、恢复、删除和重新创建，全程保持该入口的 Rust 源码不变。
首次创建配置的自动追踪需要消费项目的 `build.rs` 注册 `cargo:rerun-if-changed=rsx.toml`；
demo 已接入，双语 README 和兼容文档说明接入方法及缺失文件时的重复构建代价。
替身测试不替代真实 Kit 的函数组件或交互回归。

### P2-02 可选事件辅助

样本来自 `demo/src/bin/incident_console/{details,queue,sidebar,view}.rs`：12 处 `cx.listener` 对应 12 处显式 `cx.notify()`。这是仓库内样本，不代表外部用户采用情况。

[事件辅助宏](../crates/gpui-rsx-core/src/listener.rs) 提供两种显式形式：

- `rsx_listener!(cx, handler)` 直接委托 `Context::listener`，通知由处理器负责。
- `rsx_listener!(cx, notify, handler)` 在处理器正常返回后通知，包括没有修改状态的提前返回。

两者都显式传入上下文，保留视图/事件/窗口/上下文四参签名，不根据变量名推断意图。[无窗口回归](../demo/tests/event_listeners.rs) 验证 move 捕获、点击、鼠标按钮元组、真实事件参数、原生及辅助宏的无通知处理器，以及提前返回，使用 `observe` 统计实际通知。额外回归检查上下文/处理器各求值一次、上下文借用释放、处理器名字查找、函数表达式和语法错误。Kit、官方 GPUI 0.2.2 与改名依赖 fixture 都纳入签名编译验证。

已确认两项约束：读取泛型事件字段时可能需要显式标注 `&MouseDownEvent`；无状态变化的提前返回仍会通知。辅助宏支持同步 `Fn`，不推断通知是否必要，也不承诺 panic 或异步任务结束时通知。IncidentConsole 的筛选、队列选择和顶部操作采用显式 `notify` 形式；其他原生写法继续有效。双语语法文档、Rustdoc 与生成 spec 同步说明通知责任。

### P2-03 显式状态辅助

[状态更新宏](../crates/gpui-rsx-core/src/update.rs) 提供 `rsx_update!(view, cx, update)`。
调用方显式传入可变视图和匹配的上下文，执行同步 `FnOnce`，在正常返回后通知并返回结果。
可将多项字段更新合并到一个通知入口，支持 move 捕获、借用捕获与提前返回。
[真实后端测试](../demo/tests/state_updates.rs) 通过实体观察者统计通知，并验证普通赋值不通知。
根 crate 测试验证参数各求值一次和生成 helper 的名字查找；Counter 的重置事件采用该入口。

补充回归覆盖 `Result::Err` / `?` 正常返回仍通知、借用局部变量、panic 后保留已有修改但不通知、
恢复更新，以及不同实体只通知各自观察者。Kit 和官方 GPUI 0.2.2 运行同样四项状态测试。
真实点击测试验证普通监听器与更新宏组合修改多个字段时只有一次观察通知。
编译通过用例覆盖泛型、生命周期和拥有值返回；五份编译失败快照覆盖上下文类型不匹配、
不可变视图、跨异步借用、错误参数数量和错误的监听器通知标记。

该宏不接管实体所有权，不拦截任意赋值，不做相等性判断、panic 回滚或异步完成通知。
无变化的正常返回仍会通知；不要与 `rsx_listener!(cx, notify, ...)` 重复承担通知。

### P2-04 身份标识验收

[身份标识矩阵](../demo/tests/identity_contract.rs) 验证同文件同一行的不同宏调用、同宏兄弟节点、
源码移动、兄弟插入、列表插入/重排以及显式实例和行 ID。
自动 ID 区分调用位置，显式 ID 提供跨源码位置的身份；已有无窗口交互回归另外验证重排后的点击目标。
Kit 与官方 GPUI 0.2.2 运行同一矩阵。验收未发现需要改算法的缺陷，因此保留现有算法。
未实现自动跨版本或进程重启后的状态恢复，不将显式 ID 的稳定性扩大成此类承诺。

### P2-05 版本验证范围

官方 GPUI 0.2.2 fixture 从仅检查编译扩展到运行反馈、身份标识和状态更新回归，接入既有 macOS CI。
改名依赖 fixture 验证 Kit 0.8.1 上的路径配置；两者均不代表完整方法表已适配其他快照。
没有未经验证地扩大版本区间或后端支持声明。

### P2-06 开发重载原型

[构建/重启脚本](../scripts/dev-reload.mjs) 监视源码与配置，串行构建、合并构建期间的改动，
成功后重启应用；失败保留旧应用，退出时取消构建并清理子进程。
[十三项 Node 回归](../scripts/dev-reload.test.mjs) 覆盖失败恢复、构建合并、文件增删/输出目录排除、
退出取消、启动错误、强制终止与跨 stdout 数据块的就绪标记。
[四项 CLI 测试](../scripts/dev-reload-cli.test.mjs) 用独立子进程和可控 Cargo/应用替身验证
参数传递、带空格的路径、自定义监视目录、默认监听范围、真实文件改动、失败恢复、报告落盘及构建取消/进程清理。
涉及 POSIX 信号的用例在 Windows 跳过，其余跨平台用例保留；本机执行结果仅代表 macOS。
CI 与本地检查入口均使用 `scripts/dev-reload*.test.mjs`。

新增回归实际复现并修复两处问题：stdout 大数据块在截断前未检查就绪标记，以及等待旧进程
退出期间出现新改动时短暂启动过期构建。修复后先检查完整输出，在停止旧进程后再次确认待处理改动。
如果此时后续构建失败，会回退启动本轮最近成功的产物，并在启动和就绪报告中标记 `fallback: true`，
避免旧进程退出后应用消失。后续改动仍可恢复正常构建和启动。
默认监听同时覆盖根 `Cargo.toml`、core 的 `Cargo.toml` 与源码，避免依赖或 feature 配置变化漏掉重建；
core 的测试目录不纳入应用依赖监听。

```sh
node scripts/dev-reload.mjs --manifest-path demo/Cargo.toml --bin counter --offline --report /tmp/rsx-reload.json
```

2026-11-03 在本机 Counter 上记录以下观测（毫秒）：

| 触发 | 构建 | 启动到 render 标记 | 检测到改动至 render 标记 |
| --- | ---: | ---: | ---: |
| 缓存初次构建 | 574.524 | 3687.545 | 4262.267 |
| 实际视图标题修改 | 1444.141 | 1981.451 | 3431.286 |
| 实际 core key helper 修改 | 3185.441 | 2122.497 | 5313.689 |

计时基于 `performance.now()`；Counter 仅在脚本设置环境标志时输出首个 render 调用标记。
数据含热缓存且并行执行其他验证，没有受控基线对照，不能证明加速收益。
300 ms 轮询等待发生在检测之前；标记不等于 GPU 呈现或用户可见时间。
这是会丢失应用状态的构建/重启原型，未实现进程内热替换或状态保留。

### P2-07 与 P2-08 的范围决定

保持默认 px、HTML 标签为 Div 别名、普通 Fragment 的具体返回类型，避免把研究建议直接变成破坏性变更。
可通过刻度选项、真实组件和 `rsx_fragment!` 满足不同需求，双语文档已有对应示例。
工具投入本轮集中在可复现的诊断和编辑器共存问题，并接入 CI；没有足够使用证据支持新增
模板、风格 lint、MCP、Agent skill 或发布渠道，这些方向保留为候选，不记为已实现功能。

## 外部验收与后续研究

| 项目 | 当前证据与剩余工作 |
| --- | --- |
| 原生桌面交互 | Counter 进程可运行，原生自动化读取窗口两次超时；未完成桌面点击、IME、辅助功能和视觉验收 |
| Zed 宿主 | 本机未安装 Zed，保留实际编辑器共存验收；不能用 VS Code 结果替代 |
| Linux / Windows | 已有对应 CI 入口，本机不能执行其他系统的原生验收；远程 CI 尚未运行 |
| 真正热重载 | 当前交付为构建/重启原型；进程内替换、状态保留和受控收益比较仍需设计与验证 |
| 额外版本与工具扩展 | 按实际目标需求单独验证；本轮不宣称任意版本兼容或新增分发渠道 |

P0/P1 的实现与可在本机完成的 P2 工作已交付；以上平台验收和研究项明确保持未完成。
原评估文档中的决定、验收证据与剩余边界归档于本记录及使用文档，临时评估文件已删除。

## 后续维护记录方式

每次上游升级、迁移、发布或工具故障，在对应 PR/问题中记录以下字段，再汇总到本文件：

| 字段 | 记录内容 |
| --- | --- |
| 范围 | 起止提交、GPUI/Kit 版本、锁文件及平台 |
| 复现 | 最小输入、实际错误、触发命令与预期行为 |
| 修复 | 修改文件、是否改变默认值、用户需要执行的迁移步骤 |
| 成本 | 实际人工工时、CI/本地耗时、缓存条件、失败和重跑原因；未测量项写“未测” |
| 验证 | 编译/无窗口交互/桌面人工各自结果，及未覆盖场景 |

本轮没有可核实的外部接入成功率、迁移工时或发布工时数据，不据此给采用率和资源预算设定量化目标。
