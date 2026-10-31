# 更新日志

[English](./CHANGELOG.md) | 简体中文

本文件记录了项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
项目遵循 [语义化版本](https://semver.org/lang/zh-CN/spec/v2.0.0.html)。

## [未发布]

## [0.10.2] - 2026-10-31

### 修复

- `InteractiveText` 的 `onClick` 和 `on_click` 元组属性展开为
  `.on_click(ranges, listener)`，普通 Div 的单监听器事件保持原有行为。
  使用真实 GPUI Kit 后端编译反馈中的原始示例，覆盖回归。
- `cargo xtask bump` 同步更新 fuzz 与测试 fixture 的 lockfile 版本。
- fuzz 目标在重复调用分析函数前克隆 `ClassMode`，修复所有权错误导致的编译失败。
- 修复 macOS Bash 3.2 下 wasm-pack 需要安装匹配的 wasm-bindgen CLI 时的 Playground 构建。

### 变更

- 文档站升级到 Astro 7.4.4、Starlight 0.42.8 和 sharp 0.35.5；保留 Astro
  检查器支持的 TypeScript 6。
- VS Code 扩展升级到 language client 11、vsce 4.1、esbuild 0.28 和
  TypeScript 7，显式配置 Node 模块解析与类型。保留 VS Code 1.96 支持，并通过
  可复现安装的补丁适配已修复的 minimatch API。
- 转换器升级配套 SWC 包，wasm-bindgen 升级到 0.2.130，并同步 Playground
  CLI 版本检查。保留公开的 syn 2 AST 类型和 Rust 1.88 MSRV。

### 安全

- 刷新文档、编辑器和 demo 的传递依赖，包含 demo 的 XML、HTTP、QUIC、TLS
  与 crossbeam 修复，继续使用 GPUI Kit 0.8.1。
- CI 审计四个活跃 Rust 项目与两个 Bun 项目的 lockfile。

## [0.10.1] - 2026-10-31

### 新增

- 可选的 `rsx_fragment!` 始终返回 `Vec<gpui::AnyElement>`，支持混合不同类型的根节点，
  并支持格式化、宏发现与展开 hover。普通 `rsx!` Fragment 保留原有具体元素类型。
- 新增针对官方 GPUI 0.2.2 的 macOS 回归 fixture 与 CI 编译检查，与 Kit demo 共用
  key、组件默认值和 Fragment 的回归测试源码。

### 修复

- 动态 `Display` key 将格式化结果转换为 GPUI 的 `SharedString`，使 `String`、借用
  字符串和 `SharedString` 可在 GPUI 0.2.2 下编译。具体整数保留命名整数快路径，
  key 仍只借用求值一次。
- 组件的无后缀数值默认值按声明的基础数值类型推断，支持算术与位运算表达式。
  字符串、回调、带后缀数值以及其他 `Into` 默认值保留原有转换。
- 快速对比示例显式使用 `rsx_tw!` 表达 16px 间距，说明默认像素刻度，移除编译结果
  完全相同的错误表述。

## [0.10.0] - 2026-10-29

### 新增

- demo 新增可交互语法展示和真实 GPUI 测试，覆盖元素、组件、回调、条件样式、Fragment、
  嵌套循环、ID、主题与特殊标签。无窗口测试分发输入事件并检查布局，编译失败文档测试验证非法语法。
- 可选的 `rsx_runtime!()` 与 `[options] dynamic-runtime = "rsx_runtime"` 在 crate 内
  共享非泛型动态 class helper，覆盖测试、example 和 bin，支持 GPUI 别名和两种间距刻度。
  lint 与 LSP 会为未配置的动态 class 给出建议。本地 50 调用点 stub 测试的 release 编译
  从 12.84 s 降至 0.89 s，测量条件见性能指南。
- `#[component]` 支持同步函数，复用结构体组件的编译期必填属性检查、默认值、布尔 flag
  和 children builder。显式 `#[context]` 参数在 GPUI 渲染时获得 `&mut Window` / `&mut App`。
- 元素、组件 children、根 Fragment 和循环内均支持嵌套 `<>...</>`，不添加包装元素或
  Fragment 运行时向量。格式化保留分组，诊断、class 范围、间距迁移和预览会遍历其中内容。
- 上述能力参考 [@ppmpreetham](https://github.com/ppmpreetham) 的
  [#8](https://github.com/wsafight/gpui-rsx/pull/8)，基于当前架构重新实现。

- 组件 props 支持直接传入 Box / Rc 回调闭包、`Option<T>` 内部值和类型 / const 泛型，
  支持类型推导与显式构造函数；函数组件补齐 GPUI 的 `'static` 约束。`#[prop(callback)]`
  将 trait 别名（`trait OnPress: Fn(..) {}`）的 `Box` / `Rc` 视为回调，长签名同样可推导闭包参数。
- 支持 `bg-red-500/50`、`rounded-[4px]`、`text-[14px]`、`leading-6`，覆盖动态 class；
  CSS 专属工具类会说明对应的 GPUI 限制。
- LSP 支持标签、方法和本文件组件 prop 补全，为裸文本、标签不匹配、缺 prop 和循环缺 key
  提供修复；向支持动态文件监听的客户端注册配置监听，覆盖 VS Code 与 Zed。
- CLI 每个命令支持帮助，stdin lint 接受裸宏体；`--min-severity` / `--quiet` 过滤诊断而不
  改变退出状态。`convert --tailwind` 输出 `rsx_tw!`，保留简单标识符与成员访问，JS 行为仍需检查。
- Playground 支持内存中的 `rsx.toml` 主题、分享链接、缓存与配置诊断，并预览静态 token；
  Rust 表达式不会执行。

### 修复

- 整数 key 通过局部宏生成重复实现，动态长度按前缀复用解析函数，降低宏展开开销，
  同时保留单位、刻度与溢出校验。
- CLI 判断间距时会在展开或迁移前报告 Cargo metadata 失败，避免误用 px。失败后可重新解析，
  显式间距配置仍可在依赖无法解析时使用。
- 混合根 Fragment 和嵌套循环体按源码顺序执行完每段循环或 spread，再构造下一兄弟节点，
  避免共享可变状态时迭代器借用重叠；生成的集合变量不会遮蔽调用方变量。
- `cargo rsx doctor` 按 Kit 实际选中的后端检查，忽略未启用的测试支持依赖，同时继续拒绝
  多个活跃引擎或各组件后端 feature 不一致的配置。
- CLI 按 Cargo 解析后的 feature 判断间距，支持仅由兄弟 workspace 成员启用 `tailwind-scale`，
  使展开结果与 workspace 构建一致，并拒绝重复迁移。显式 `expand - --manifest-path ...`
  与省略 `-` 时一样读取指定包的主题及间距配置。
- 静态 grid 与 line-clamp 整数 class 使用 GPUI 对应类型的 setter，并拒绝溢出；动态长度
  在缩放或除法后检查溢出。被同名宏覆盖的 `format!` 按实际输出处理，避免错误样式或字符串切片越界。
- 嵌套 RSX 循环可借用外层绑定和调用方状态：嵌套迭代器在每次外层闭包返回前收集一次，
  普通节点循环仍使用无需临时集合的 map / 数组路径。
- GPUI 依赖校验接受未启用 Fast 后端的测试支持元数据，同时继续拒绝启用 Fast 或 GPUI 类型不一致的配置。
- 组件标签及 `base` 表达式只为 GPUI 渲染元素自动注入 ID。gpui-component 的 `Button`
  和带 `on_click` / `on_hover` 属性的 `#[component]` 构建器保留自身 setter，返回原生
  元素的路径函数则继续获得自动 ID。
- `#[prop(default = Box::new(|_, _, _| {}))]` 等回调默认值可推导闭包参数，并转换为声明的
  Box / Rc 回调类型，也支持 trait 别名。
- 动态颜色拒绝 `bg-red-500/50/50` 这类重复透明度修饰符，与静态校验保持一致，
  不再静默使用最后一个修饰符。
- 根 Fragment 中包含 `for` 循环或 `{...spread}` 时展开为 `Vec<AnyElement>`，不再生成
  装着迭代器的 `vec!`（混合子节点时无法编译）。
- `#[deprecated]` 组件在使用处给出警告，而不是在自身生成代码中报警；生成的 builder
  使用完整的 `gpui` 路径，并支持 `r#Card` 这类原始标识符。
- lint、迁移和语言服务器会分析嵌套在其他宏内部的 `rsx!` 调用，并使用嵌套宏自己的模式。
- 工具按与宏相同的优先级识别 `gpui-rsx` 的 `tailwind-scale` feature，`migrate` 会拒绝
  重复执行，hover 显示的长度与编译结果一致。
- Code action 只覆盖请求的范围；文件未变化或无法解析时，格式化返回空结果而不是报错。
- class 字符串包含 Unicode 空白时，LSP 补全与 hover 保留有效的 UTF-8 范围，避免语言服务器崩溃。
- `preview` 支持目录参数，`--watch` 在 `rsx.toml` 修改后会刷新。
- 变体内的未知 class（如 `focus:ring-2`）的透传警告改为显示生成的调用
  `.focus(|style| style.ring_2())`，不再给出非法的方法名或 flag 属性。
- 命令会拒绝不适用的选项，`lint` 会检查显式指定的任意扩展名文件，`doctor` 输出纯文本值。

- 不超过 64 位的具体动态整数 key 使用 GPUI 命名整数 ID，无需格式化字符串；其他 Display
  值保留借用、单次求值的格式化路径。
- format class 模板的完整静态首尾 token 在编译期展开，原表达式仍求值一次，样式顺序保留；
  中间部分继续动态处理。
- 平衡宏体内的编辑只扫描该宏体，LSP 合并队列中的连续修改，并复用语义高亮与颜色分析缓存。
  本地 5000 行样本完整分析约 1.12ms，单次增量修改约 0.15ms。
- 常用运行时 class matcher 按首字节分派；性能指南记录实测收益、回退场景与复现命令。

### 变更

- class 行为变更：宽松模式下，静态 `z-*`、`space-x-*`、`space-y-*`、`divide-*`、
  `transition*`、`duration-*`、`ease-*`、`delay-*` 和 `animate-*` class 会警告并忽略，
  不再透传为扩展方法；严格模式下报错。自定义实现改用显式属性（如 `z_10`、`transition`）
  或 `base={...}` 调用。
- 转换器保留 `{items}` / `{item.title}`，不再提示需要手动转换；仍需检查 Rust 类型和
  所有权，集合子节点可能需要 `{...items}` 或 `{for ...}`。
- 语言服务器按文档缓存行偏移，每个文档只保留一份分析结果而不是每次编辑时复制，
  代码拆分为 protocol、document、features 模块。`gpui-rsx-core` 的主题与组件代码拆分为目录模块。
- 新增 `cargo xtask line-limit`（CI 和 `scripts/check.sh` 会运行），限制手写 Rust 文件的非空、
  非测试代码不超过 600 行。
- VS Code 插件从自身版本推导安装版本，从 GUI 启动时也能找到 `~/.cargo/bin`，
  显示真实的启动错误，版本不一致时给出提示，修改设置后自动重启并监听 `Cargo.toml`。
- VS Code 接受 major.minor 相同的服务端补丁版本，安装命令会选择最新兼容补丁，
  不再固定安装插件自身的旧补丁版本。
- 破坏性 API：`Rc<dyn Fn...>` 组件属性与 `Box<dyn Fn...>` 属性及 GPUI 事件 setter 一样接收
  `impl Fn... + 'static`，闭包参数和生命周期可自动推导（`on_press={|_, _, _| ...}`）。
  已有的 `Rc` 值需转发：`on_press={{ let f = f.clone(); move |e, w, cx| f(e, w, cx) }}`。
- 破坏性 API：`gpui-rsx-tools` 的 `Diagnostic::severity` 改为 `#[non_exhaustive]` 的
  `DiagnosticSeverity` 枚举，序列化结果与原字符串相同；整文件分析包含嵌套宏（`macro_calls`
  仍只返回顶层调用，参见 `nested_macro_calls` 与 `call_at`）。`report_body`（`#[non_exhaustive]`
  的 `BodyReport`）一次解析即返回诊断、展开结果和预览，它与 `expand_summary` 均省略动态 class
  helper。详见迁移指南“从 0.9 升级到 0.10”。
- 工具分析、hover 和 Playground 不再生成动态 class helper：80 个动态 class 宏的整文件分析
  从约 67 ms 降至 2 ms。`lint` 按 workspace 解析一次 Cargo feature，而不是每个成员一次。
- 新增 `cargo xtask bump <version>`，统一设置 manifest、lockfile、编辑器插件和安装片段中的
  发布版本；CI 会运行 `cargo xtask bump --check`。
- `gpui-rsx-tools` 不再精确锁定 `serde` 版本。
- 语言服务器仅在 `initializationOptions.formatting` 为 `true` 时声明 RSX 格式化能力
  （VS Code：`gpuiRsx.format.enable`），编辑器默认仍使用 rustfmt 格式化 Rust。

## [0.9.0] - 2026-10-28

### 变更

- 主题加载复用拥有型 `Arc<Theme>`，不再为每次配置修订泄漏静态主题；缓存使用规范路径、内容指纹和
  最多 64 项的 LRU 淘汰机制，并报告配置读取错误。
- API 破坏性变更：`ClassMode`、`Theme`、`MacroOptions` 改用私有字段、builder 和 getter；
  `ClassMode` 由 `Copy` 改为 `Clone`。语法、class 结果、诊断级别和 token 枚举标记为
  non-exhaustive；工具集成可使用 crate 根导出及 `syntax` 模块。
- Theme 新增 `[text-size]`、`[line-height]`、`[opacity]`、`[shadow]`；支持内存主题校验，
  拒绝重复键和有歧义的 class。新增 `cargo rsx theme check/list/explain`，提供文本和 JSON 输出，
  可解释状态前缀生成的代码。
- LSP 支持 UTF-16 增量编辑、文档版本检查和未修改宏体的分析复用；语义高亮区分普通工具类、主题
  token 和无效 class。主题配置错误持续显示为诊断，静态主题颜色支持色块预览。
- class 代码生成与静态预览共享样式操作；预览支持静态主题 token 和 `styled` 默认样式，
  运行时 Rust 表达式仍显示为占位内容。
- feature 默认值破坏性变更：`gpui-rsx-tools` 默认不再启用 SWC；转换功能需显式开启
  `features = ["convert"]`。CLI、WASM 保留转换能力，CI 覆盖启用与禁用 feature 两种情况。
- 同一次宏调用内的动态 class 共享一个 helper，各属性仍独立求值并调用自己的未知 class 回调。
  本地四属性宏展开示例从约 1.99 ms 降至 0.495 ms；80 个宏中复用 79 个宏体的分析从约
  1.00 ms 降至 223 µs。这些测量不代表整个项目的冷编译收益。
- 双语文档补充迁移说明和基准命令；完成范围与验证记录在 `todo/ROADMAP_0.9.0.md`。

## [0.8.3] - 2026-10-27

### 修复

- LSP 在 Windows 上保留文件 URI 的根路径前缀，只在百分号解码后对盘符路径移除额外的 `/`，
  并拒绝不完整的百分号转义。
- 静态预览能识别文件大小和修改时间均不变的编辑，并继续对未改变的源码复用渲染结果。

### 变更

- 展开基准在调用之间重置 span 内存，并对两个版本使用相同的测试代码。性能 CI 隔离每次运行的
  报告，在超过 10% 门限时重新测量基线和当前版本，确认超限后才判定失败。
- 基于 SWC 的 JSX/TSX 转换现在由可选的 `convert` feature 控制，同时默认开启以保持兼容；CLI
  和 WASM Playground 会显式启用该 feature。
- Preview 返回缓存的渲染片段时不再复制整段内容，宏扫描也不再为每个 token 创建临时字符串。
- CI 会取消同一分支中过期的运行，按 job 和 Rust 通道隔离构建缓存，并单独检查 CLI 工具的 Rust 1.98
  MSRV 与宏库的 Rust 1.88 MSRV。

## [0.8.2] - 2026-10-25

### 修复

- `rsx!` 不再因以下输入 panic：含大写字母的原始标识符属性（`r#myAttr`）、缩放后超出 `f32` 的长度
  （`rsx_tw!` 下的 `w-1e38`）、`#[component]` 中的原始标识符 prop（`r#type`）；PascalCase 后同名的
  prop 会生成不同的泛型参数。
- `for` 循环体中直接嵌套的 `for` 或 `{...spread}` 现在会被拍平，而不是生成“迭代器的迭代器”。
- LSP 与 wasm Playground 在每次请求后释放 proc-macro2 的 span 内存，编辑时内存不再持续增长；
  格式错误的 LSP 消息返回 `-32700` 而不是终止服务，`initialize` 之前 / `shutdown` 之后的请求返回规范错误码。
- `cargo rsx fmt` 原样保留 Rust 表达式文本（不再把每个 token 用空格隔开），字面量属性不加花括号，
  保留 CRLF，跳过无法解析或含多行字符串字面量的宏，并且只在所有文件都格式化成功后才写盘。
- `cargo rsx fmt`/`lint` 支持用 `-` 读取 stdin；`lint` 的列号按字符而非字节计算。
- `cargo rsx convert` 把 Tailwind 数值 class 换算为像素（`p-4` → `p-16`），并丢弃不是 Rust 标识符的
  属性名或标签名（`aria-label`、`type`）且给出提示。
- 主题 token 对所有长度前缀（`pt-`、`min-w-` 等）提供补全和建议；`rsx.toml` 中转义引号后的注释能正确解析；
  `#+12345` 不再被当作合法颜色。
- 预览服务器在单个连接失败后继续运行。

### 变更

- 编辑器颜色预览与静态预览复用宏的颜色解析逻辑。
- for 循环缺少 key 的错误信息同时说明 stateful class 也是原因之一。

## [0.8.1] - 2026-10-22

### 变更

- LSP 缓存已解析的 RSX 文档状态，包括 macro 范围、class 字面量范围、诊断和模式解析，
  使补全、悬停、颜色和 code action 复用同一次解析。
- 动态 class helper 按间距刻度使用进程级源码缓存，未知 class 去重改用 `HashSet`。
- `cargo-rsx preview --watch` 对未修改文件复用上一次渲染结果。
- Playground Worker 增加有界结果缓存和过期结果保护，避免重复或已被新编辑替代的结果更新界面。
- VS Code 只在看起来是 GPUI-RSX 项目的工作区启动服务，并提供安装和重启 `cargo-rsx` 的命令。

### 修复

- 所有安装示例和兼容性片段统一更新为 `gpui-rsx`/`cargo-rsx` 0.8.1。
- CI 增加检查，禁止文档重新出现过期的 `gpui-rsx` 0.x 依赖示例。

## [0.8.0] - 2026-10-21

### 新增

- 拆出可复用的 `gpui-rsx-core`，生成 GPUI 方法表、五个 trait 的 API 快照，
  补充展开快照、fuzz 目标与 Criterion 性能回归门禁。
- Tailwind 间距刻度、静态状态前缀、组件属性 builder、主题令牌；所有未知静态 class
  都提供编译期诊断，扩展方法会提示显式属性写法。
- `cargo-rsx`：依赖 doctor、项目 lint（文本/JSON）、RSX 格式化、间距迁移、
  基于 SWC 的 JSX/TSX 转换、静态布局热预览，以及支持补全、悬停、修复和色块的语言服务。
- 复用 Rust core 的 WASM Playground，支持展开、诊断、分享、转换与预览；
  配套可安装的 VS Code 和 Zed 扩展包。
- 自动生成的语法规范、提示词与双语兼容矩阵。

### 变更

- demo 迁移到 GPUI Kit 0.8.0 / gpui-pre 0.3.9。宏库保留 Rust 1.88 MSRV，
  不依赖 GPUI 运行时；CLI 工具要求 Rust 1.98。
- 兼容工作流跟随 Kit 正式版本，经 Linux/macOS 验证后提修复 PR，并归并历史机器人 Issue。

### 说明

- JSX 转换和浏览器预览需要用原生应用复核行为；格式化保留带注释的宏体；
  间距迁移需在启用 Tailwind 刻度前显式执行一次。扩展市场审核与扩展包分发分别记录。
- 模型生成代码成功率、兼容修复时效需要持续测量，本版本不声称已证明这些结果指标。
- VS Code VSIX 和 Zed 扩展压缩包已附在 [v0.8.0 GitHub Release]；市场发布仍需发布者认证和
  registry 审核。


## [0.7.0] - 2026-08-25

### 新增

- 新增完整的项目仪表盘 demo，覆盖 keyed 过滤列表、选中状态、条件 class、动态进度、
  密度控制和条目增删改操作。

### 变更

- 将过程宏 crate 的 MSRV 从 Rust 1.85 提升到 Rust 1.88，并同步更新 CI 检查通道。
- 按模式缓存完整的动态 class helper，使每次宏展开只解析一个 token stream，
  不再分别解析 common、color 和 numeric 三段代码。
- 将文档栈升级到 Astro 7，并把依赖安装、脚本和 CI 从 pnpm 迁移到 Bun 1.4，
  使用固定的 Bun lockfile。
- 使用共享生成宏减少 benchmark 和测试 mock 的重复实现，并补充聚焦的 parser/codegen
  单元测试覆盖。

### 修复

- 防止 class benchmark 初始化被常量折叠，并增加动态 spacing 快速路径与数值 fallback
  的直接对比测量。
- 所有 demo 在窗口创建失败时都会报告错误，不再静默丢弃结果。

## [0.6.0] - 2026-06-12

### 新增

- 新增固定依赖版本的 `demo/` crate，用 Zed git 仓库中的 GPUI 与 `gpui-component`
  一起做真实 API 校验，覆盖 hello、counter、palette、task list、API surface 和组件集成示例。
- 新增 demo crate 的 CI 校验：
  `cargo check --manifest-path demo/Cargo.toml --bins --locked`。
- 新增基于 demo manifest 的 GPUI 兼容性工作流，使未来 Zed GPUI 变更能通过真实 GPUI API
  表面检测。
- 新增 `class={match ...}` 静态展开：当所有 match arm 都返回 class 字符串字面量时，
  不再走动态 class matcher。
- 新增静态 `opacity-*` 范围校验，并补充非法 opacity class 的 UI 测试覆盖。
- 新增 `hoverClass`、`focusClass` 和 `activeClass` 属性，通过 `StyleRefinement` 闭包展开
  GPUI 状态样式 class。
- 新增更多真实 GPUI 交互与无障碍属性映射，包括 `focusVisible`、`tooltipShowDelay`、
  `onAuxClick`、`onA11yAction`、`role` 和 `aria*` 属性。
- 为最新 accessibility、滚动轴限制、外部拖拽、mouse pressure、pinch、grid min/max-content
  和 text ellipsis API 补充 alias 与真实 GPUI contract。
- 新增固定的 42 方法 `StatefulInteractiveElement` 快照，以及 latest 兼容性工作流使用的
  实际源码漂移检查。

### 变更

- permissive 模式下，未知动态 class 的 debug 警告现在按“生成点 + class 值”最多打印一次，
  避免渲染循环中重复写 stderr。
- stateful 元素上的字面量 `key` 现在使用编译期 `concat!` 自动 ID 路径，不再运行时
  `format!`；动态 `key={expr}` 仍保留原有基于 `Display` 的 fallback。
- `visible={expr}` 现在只求值一次，再映射到 `.visible()` 或 `.invisible()`。
- 动态 class 文档现在明确推荐 `if` 和 `match` 字面量表达式作为可编译期展开的替代写法。
- 收敛属性方法元信息，使方法映射、自动 ID 判断和 tuple 参数展开共用同一条查找路径。
- Astro docs workflow 现在会在文档 PR 上运行，执行 `pnpm run check`，并且只在 main push
  时使用收敛后的 Pages 权限部署 Pages artifact。
- 补充 `groupHover` / `groupActive` 的 ID 边界，并说明 GPUI 要求通过
  `group_drag_over::<YourType>(...)` 显式处理 `groupDragOver`。
- 复用 `StyleRefinement` 使用的 helper 宏生成 `MockElement` 的 `Styled` surface，减少测试
  mock 中的重复实现。
- 将共享测试 mock 的状态捕获和基础类型拆到独立的 `tests/common` 子模块。
- 新增 `scripts/check.sh`，统一运行根 crate 和真实 GPUI demo 的标准验证命令，并通过
  `--release` 模式覆盖 benchmark、docs、GPUI tree 和 publish dry-run 检查。
- 收敛 crates.io 发布包内容，仅保留构建和文档展示所需文件。
- 在发布检查清单中补充 crates.io 包内容验证。
- 补充状态 class 属性和显式指定类型的 `group_drag_over` fallback 问题排查说明。
- 明确 0.6.0 新增有状态自动 ID 触发项的性能文档说明。
- 在主要文档入口补充兼容性和发布检查清单页面链接。
- 将 demo lockfile 升级到 Zed GPUI `e9735934` 与 gpui-component / gpui-base `7885c416`。
- 声明根过程宏 crate 的 MSRV 为 Rust 1.85，并保留 demo 的 Rust 1.95 兼容性目标。
- 兼容性 issue 现在先汇总两个平台的结果，再统一改变 issue 状态。

### 修复

- 修复 `benches/class_performance.rs`，使 benchmark 目标重新可以干净编译。
- 修复 for 循环解析，使 `{for item in { items.iter() } { ... }}` 这类 iterator block 表达式可用。
- 修复动态非法 opacity 处理，使运行时非法 `opacity-*` 值被忽略，而不是生成无效 GPUI opacity 调用。
- 修复 demo 依赖说明，避免同时使用 `gpui-component` 时产生重复 GPUI crate 实例。
- 修复 GPUI 状态样式兼容性，使 `hover`、`focus` 和 `active` 闭包与当前 GPUI 一样接收
  `StyleRefinement`。
- 改进状态 class 诊断，对 `overflow-scroll` 和 `debug-outline` 这类元素级 class 给出更具体的
  修复建议。
- 对 `groupDragOver` 属性给出可操作诊断，因为 GPUI 需要显式拖拽数据类型。
- 不再把 `scrollbarWidth` 误判为 stateful，同时保留 overflow scroll 的自动 ID。
- 在静态、strict 和动态 class 路径恢复 `text-ellipsis-start` 并新增 `text-ellipsis-middle`。
- 让 class benchmark 消费带可观察状态的非零大小 builder 结果。

## [0.5.1] - 2026-06-09

### 新增

- 新增 GPUI 0.2.2 元素支持：`<img src={...}>`、`<canvas prepaint={...} paint={...}>`
  和 `<svg src={...}>`。
- 新增 GPUI 0.2.2 图片和 prepaint 相关属性映射，包括 `objectFit`、`withFallback`、
  `withLoading`、`imageCache` 和 `onChildrenPrepainted`。
- 新增 GPUI 0.2.2 圆角半径变体的静态 class 和 strict 模式支持。

### 变更

- 优化 `class={if active { "flex" } else { "block" }}` 这类条件字面量 class，
  使其走静态 class 展开路径，而不是动态 class matcher。
- 跳过已由元素代码生成器消费的属性分析，减少宏展开阶段的重复工作。
- 对常见数值长度、颜色、opacity、line-clamp 和方向性边框 class 先在原始字符串上解析，
  减少静态 class parser 中的字符串规范化分配。
- 更新示例和文档，在 GPUI 0.2.2 需要的位置导入 `gpui::prelude::*`。

### 修复

- 修复 `aspect-square` 的 GPUI 0.2.2 兼容性，改为写入 `style().aspect_ratio`，不再调用
  已移除的 GPUI helper 方法。
- 修复 GPUI helper 移除后 `text-ellipsis-start` 在 permissive/strict 模式下的处理。
- 修复 benchmark mock，使其与当前 GPUI 0.2.2 兼容 API 保持一致。

## [0.5.0] - 2026-05-14

### 新增

- 新增路径型组件标签支持，例如 `<ui::TaskCard />`，包括闭合标签匹配校验和
  stateful 路径组件的自动 ID 生成。
- 新增宏专用 `base={expr}` 支持，可让组件方法链从自定义构造函数或 builder 开始，
  而不是默认的标签构造函数。
- 新增 `whenClass={(condition, "class string")}`，用于按条件应用静态 class。
- 新增 `fontFamily`、`textColor`、`backgroundColor` 和 `borderColor` 的 camelCase 属性别名。

### 变更

- 改进 `whenClass`、路径标签不匹配、for 循环 key 和不支持的 `whiteSpace` 属性诊断，
  提供更精确的 span 和可操作提示。
- 在 README 和 API 文档中补充动态 class 能力边界、builder 构造组件、路径型标签和
  Fragment 混合元素类型指导。

### 移除

- 移除过时的独立 `gpui-rsx-optimization.md` 文档。

## [0.4.4] - 2026-05-14

### 变更

- 统一 common class 支持元数据，使 strict class 校验和动态 class 快速路径共享单一数据源，
  同时保留动态 stateful class 行为。
- 将动态数值 class fallback 生成改为复用共享长度前缀元数据。
- 移除 crate 级 release profile 覆盖，让应用和 workspace 自行控制 LTO、codegen units 和
  panic 策略。

## [0.4.3] - 2026-05-14

### 新增

- 新增 `rsx_strict!`、`rsx_permissive!` 和 `rsx_expand!` 宏，用于更严格的 class
  校验、显式 permissive 模式和生成代码预览。
- 新增 spacing 和 sizing class 的任意长度支持，包括 `px`、`rem`、sizing 百分比，以及
  `w-6/24` 等分数尺寸。
- 新增任意 RGB/RGBA 颜色支持和 GPUI 0.2 字重映射。

### 变更

- 改进动态 class 对任意颜色、任意长度、分数尺寸、`debug-outline` 和 GPUI 0.2 兼容
  helper 的处理。
- 保留未知静态 class 在 permissive 模式下的兼容处理，同时让 `rsx_strict!` 清晰报告不支持的 class。

### 修复

- 修复动态 `font-extralight` 支持，使其正确映射到 `FontWeight::EXTRA_LIGHT`。
- 拒绝 `NaN` 和 `inf` 等非有限数值 class，避免传入 GPUI 长度 helper。
- 修复动态方向性边框 class 处理，并完善多种非法任意值诊断。

## [0.4.2] - 2026-05-12

### 变更

- 在代码生成阶段复用属性扫描结果，避免重复分配属性名和静态 class 字符串。
- 将 for 循环 key 校验并入递归代码生成流程，移除生成前的额外遍历，同时保留循环安全
  auto-ID 诊断。
- 保持 styled 默认 class 位于用户属性之前，同时以单次遍历生成用户属性方法链。

## [0.4.0] - 2026-05-11

### 变更

- 将 GPUI 兼容性文档和示例更新到 GPUI 0.2。
- 更新事件和交互属性映射以匹配 GPUI 0.2 方法签名，包括多参数鼠标和拖拽 API。

### 修复

- 修复 GPUI 0.2 方向性边框 flag 的预设宽度方法生成
  （`border_t` → `.border_t_1()` 及其他方向）。
- 保留 `border_t={px(1.0)}` 等方向性边框带值属性为 `.border_t(value)`，
  避免错误映射到预设宽度方法。
- 避免为连续表达式子节点生成 `.children([...])`，使混合子节点类型能够正确编译。
- 更新 GPUI 0.2 stateful 交互方法和静态 overflow scroll class 的自动 ID 检测。

## [0.3.2] - 2026-02-22

### 🐛 修复

#### `parse_single_class` — Tailwind 变体语法导致的 panic
- **对非标识符 class 名的防御性检查**（`class.rs`）— 包含非字母数字/下划线字符的 class
  （如 `hover:bg-blue-500`、`focus:text-red-500`）此前会导致 `syn::Ident::new` 在编译期
  panic。默认分支现在会在构造 `Ident` 前验证 `method_name` 仅含 ASCII 字母数字和下划线字符；
  无效 class 名静默产生空 `TokenStream`，同一 `class="…"` 字符串中的有效 class 仍正常应用。

#### `Styled` trait — 缺少方向性边框无参方法
- **将 `border_t`、`border_b`、`border_l`、`border_r` 加入 `Styled` trait**
  （`tests/common/mod.rs`）— 这四个方法原本仅以泛型 `<T>` 的固有方法形式存在于 `MockElement`
  上，用于支持 `borderTop={val}` 属性形式。然而 `class="border-t"`（及其他三个方向）已通过
  `is_directional_border` 正确回落到默认分支并生成无参的 `.border_t()`，若存在相关测试便会
  导致编译失败。这四个方法现已作为无参签名（与真实 GPUI API 一致）加入 `Styled` trait，
  原泛型固有方法已被移除。属性测试改为使用 flag 形式（`<div border_t />`），并新增了四个
  测试用例（`test_class_border_t/b/l/r`），覆盖此前未被测试的代码路径。
- **将 `border-t`、`border-b`、`border-l`、`border-r` 加入动态 class match 表**（`runtime.rs`）
  — 这些 class 原本不在 `static_classes` 中，导致 `class={expr}` 中包含 `"border-t"` 等
  时，运行时会静默打印警告并什么都不做。现已与 `border`、`border-2` 一同加入预编译 match 表。

#### `generate_numeric_fallback_code` — 每次调用重复执行 `quote!`
- **数值回退代码的 thread_local 缓存**（`runtime.rs`）— `generate_numeric_fallback_code`
  在每次调用 `generate_dynamic_class_code` 时都重新执行 `quote!`（约 40 条 `if-let` 语句），
  而 `generate_common_class_matches` 已通过 `thread_local` 字符串缓存。新增
  `NUMERIC_FALLBACK_STR` thread_local 和 `get_cached_numeric_fallback()` 函数，应用相同的
  缓存模式：`TokenStream` 仅序列化一次，每次 proc-macro bridge 调用时重新解析，
  消除多个 `class={expr}` 属性时的重复 `quote!` 分配开销。

### ✨ 增强

#### 动态 class match 表 — 新增 8 个 class
- **`rounded-none`、`rounded-xl`** — 加入 `runtime.rs` 静态 match 表；此前仅在静态字符串
  路径中有效，在 `class={expr}` 表达式中使用时会静默失效。
- **`cursor-default`、`cursor-text`** — 同上。
- **`overflow-visible`** — 同上。
- **`shadow-sm`、`shadow-md`、`shadow-lg`** — 同上。
- 同步将上述 8 个方法从 `impl MockElement` 提升至 `tests/common/mod.rs` 中的 `Styled` trait，
  确保生成的 `__rsx_apply_class` 辅助函数的 `E: Styled` 约束可访问这些方法。

### 📖 文档

- **styled 默认样式表**（`lib.rs`、`README.md`）— 补充缺失条目：`li` → `flex items-center`、
  `p` → `text-base`、`label` → `text-sm`、`form` → `flex flex-col gap-4`。
  这些默认值已在 `tables::lookup_tag_default` 中实现，但未在文档中体现。
- **属性映射表**（`lib.rs`、`README.md`）— 补充缺失的 `roundedTop` → `.rounded_t()` 和
  `roundedBottom` → `.rounded_b()` 条目。
- **动态 class 说明**（`lib.rs`、`README.md`）— 将不准确的「约 58 个预编译常用 class」
  表述替换为准确描述：完整 Tailwind 色板（22 色系 × 11 色阶 × 3 前缀 = 726+ 条）、
  常用布局/间距/文字排版工具类，以及通过前缀回退支持间距/尺寸/透明度/z-index 的任意数值。
- **`overflowX` / `overflowY` 方法名**（`README.md`）— 修正 `.overflow_x_hidden()` /
  `.overflow_y_hidden()` 为实际的 GPUI 方法 `.overflow_x()` / `.overflow_y()`。
- **文本大小列表**（`README.md`）— 从支持的 class 模式中移除不存在的 `text-4xl` 和
  `text-5xl`（`is_valid_text_size` 仅支持 `xs` 到 `3xl`）。
- **动态 class 诊断测试**（`tests/diagnostic_tests.rs`）— 将 `test_class_dynamic_value`
  重命名为 `test_class_dynamic_value_is_supported`，并更正注释：`class={expr}` 是合法的
  RSX（不是编译错误），会生成运行时 match 代码。

### ✅ 测试
- 全部 293 个测试通过（231 宏测试 + 36 覆盖率测试 + 24 单元测试 + 2 诊断测试）
- 在 `coverage_tests.rs` 中新增 `test_class_with_non_ident_chars_ignored`
- 在 `macro_tests.rs` 中新增 `test_class_border_t/b/l/r`（覆盖此前未被测试的代码路径）

---

## [0.3.1] - 2026-02-21

### 🐛 修复

#### 自动 ID — `is_stateful_attr` 误判
- **从 stateful 属性检测中移除 `hover`、`active`、`focus`、`group`** —
  这些是 `Styled` trait 的样式方法（接受 `StyleRefinement`），不属于 `StatefulInteractiveElement`
  方法，不需要 `.id()`。之前它们会导致不必要的 `.id()` 注入，将元素类型从 `Div` 静默变为
  `Stateful<Div>`。现在只有 `on_*`/`capture_*` 事件处理器、`tooltip`、`track_focus` 才会触发注入。

#### 自动 ID — 循环内 ID 碰撞
- **for 循环内缺少 `id` 或 `key` 的 stateful 元素现在报编译错误** —
  for 循环的每次迭代共享同一源码位置，因此自动生成的 ID 在所有迭代中完全相同，
  导致 GPUI 状态冲突。宏现在在编译期抛出清晰的错误，指向违规元素并给出可操作的修复建议。

### ✨ 新增

#### `key` 属性 — 循环内的复合自动 ID
- **`key={expr}` 属性** — 一个宏层面的特殊属性（不会生成 `.key()` 方法调用），
  用于为循环内 stateful 元素的自动 ID 提供唯一键：
  ```rust
  // ❌ 编译错误 — 所有 <li> 会共享相同的自动 ID
  {for item in &self.items { <li onClick={handler}>{item}</li> }}

  // ✅ key 使每次迭代获得唯一 ID
  {for item in &self.items { <li key={item.id} onClick={handler}>{item}</li> }}
  // → div().id(format!("src/list.rs::__rsx_li_L42C8_{}", item.id)).on_click(handler)…
  ```
  - ID 格式：`format!(concat!(file!(), "::{prefix}_{}"), key_expr)` —
    前缀在编译期求值，key 在运行时追加，开销仅限一次字符串格式化。
  - `key` 可接受任何实现 `Display` 的类型（整数、`&str`、UUID 等）。
  - **`key` 仅在元素有 stateful 属性（`needs_id = true`）时生效。
    非 stateful 元素上的 `key` 会被静默忽略，不会注入 `.id()`。**
  - 优先级：显式 `id` > stateful + `key` > stateful 无 key > 非 stateful。

### ♻️ 重构

- **`next_auto_id` 重命名并拆分** — 替换为两个职责清晰的函数：
  - `make_auto_id(tag_ident)` — 仅源码位置，编译期 `concat!`
  - `make_keyed_auto_id(tag_ident, key_expr)` — 编译期前缀 + 运行时 key `format!`

### ✅ 测试
- 全部 288 个测试通过（227 宏测试 + 35 覆盖率测试 + 24 单元测试 + 2 诊断测试）
- 更新测试断言：`hover`/`active`/`focus`/`group` 不再断言 stateful 检测；
  新增 `tooltip`/`track_focus` 作为真正 stateful 属性的测试

---

## [0.3.0] - 2026-02-21

### ♻️ 重构

#### 测试基础设施
- **消除 `tests/common/mod.rs` 中的重复方法** — `impl MockElement` 原先定义了约 60 个方法，
  这些方法已由 `impl Styled for MockElement` 提供。直接 `impl` 块现在只保留 `Styled` trait
  中没有的方法（事件处理器、状态样式方法、条件辅助方法等）。文件从 823 行缩减至 456 行，
  每个方法保持唯一来源，消除了静默差异的风险。

#### 代码生成器
- **简化 `runtime.rs` 中 black/white 条目的生成逻辑** — 原先生成 black/white 颜色 match
  分支的循环在运行时通过 `class_str.starts_with("text-")` / `starts_with("bg-")` 来选择
  方法标识符。方法名现在直接编码在数据数组中，完全消除了运行时分支：
  ```rust
  // 之前：方法 ident 在运行时推断
  for (class_str, hex) in [("text-black", 0x000000u32), …] {
      let (method_ident, hex) = if class_str.starts_with("text-") { … };
  }
  // 之后：方法 ident 直接编码在数据中
  for (class_str, method_ident, hex) in [
      ("text-black", &text_color_ident, 0x000000u32), …
  ] { … }
  ```

- **在 `class.rs` 中提取 `is_directional_border(rest)` 辅助函数** — 区分方向性边框类
  （`border-t`、`border-t-2`）与颜色边框类（`border-red-500`）的逻辑原先内联在
  `parse_single_class` 中。现在提取为独立的 `fn is_directional_border(rest: &str) -> bool`
  函数，并附有说明边界情况的文档注释，调用处从 11 行注释+代码缩减为单行可读谓词调用。

### ✅ 测试
- 全部 287 个测试通过（227 宏测试 + 35 覆盖率测试 + 23 单元测试 + 2 诊断测试）
- 零回归

---

## [0.2.2] - 2026-02-18

### 🚀 性能优化

#### 编译时性能
- **`parse_class_string` 改用 `split_ascii_whitespace`** - class 名只含 ASCII 字符，将
  `split_whitespace` 替换为 `split_ascii_whitespace`，跳过每个词元边界处的 Unicode 空白
  字符表查询。
- **统一 `text_` 前缀处理** - 删除 `parse_color_class` 函数。颜色查找（`text-red-500`）
  和文本大小查找（`text-xl`）现在统一在 `parse_single_class` 的单次 `strip_prefix("text_")`
  下处理，消除了每个 `text-*` class 的冗余前缀剥离操作。
- **空元素提前快速路径** - `generate_element` 中的"无属性且无子节点"检查现在是函数的第一步，
  在任何变量初始化或循环入口之前执行。像 `<Icon />` 这样的裸自闭合标签可直接返回，无需扫描属性。
- **`Vec::with_capacity` 容量估算增大** - `generate_element` 中的方法缓冲区预分配从
  `attributes.len() + children.len()` 改为 `attributes.len() * 2 + children.len()`。
  单个 `class` 属性通常展开为 3-4 个方法调用，×2 系数可将 class 密集元素的预期重分配次数
  减半。

#### 运行时性能（生成代码质量）
- **`.children([...])` 聚合阈值 3 → 2** - 2 个及以上连续 `Expr` 子节点现在合并为单次
  `.children([...])` 调用，底层使用栈分配数组。数组无堆分配开销，原阈值 3 过于保守；改为 2
  可在保持相同栈占用的前提下减少方法分派次数。

#### 二进制体积
- **release 构建可选 `panic = "abort"`** - 应用可在自己的 release profile 中启用
  `panic = "abort"`，在适合其 workspace 的前提下移除展开表。

### ✅ 测试
- 全部 236 个测试通过（203 宏测试 + 31 覆盖率测试 + 2 诊断测试）
- 零回归

---

## [0.2.1] - 2026-02-18

### 🚀 性能优化

#### 编译时性能
- **基于 match 的 O(1) 颜色查找** - 将 `COLOR_MAP` 常量数组 + `.iter().find()` 线性扫描
  替换为 `lookup_color()` 中的 `match` 语句。编译器为 match 生成高效跳转表/trie，
  最坏情况下的比较次数从 O(242) 降为 O(1)。
- **属性/间距/文本大小查找同样改用 match** - 相同模式应用于 `lookup_attr_method()`、
  `lookup_spacing_method()` 和 `is_valid_text_size()`，替换了四个线性扫描数组。
- **`generate_element` 单次属性扫描** - `user_id`、`has_styled`、`needs_id` 现在在一次
  循环中提取，不再多次遍历。
- **缓存 `Ident::to_string()` 结果** - 标签名和属性名字符串在每次代码生成调用中只计算一次，
  避免重复堆分配。

#### 运行时性能
- **动态 class 字符串零拷贝** - 从 `String::into()` 改为 `AsRef<str>`。`&str` 输入无需
  分配直接通过；同样高效支持 `String` 和 `Cow<str>`。
- **`Vec` 复用避免循环内重复分配** - `consecutive_exprs` 在循环外分配一次，每轮用 `.clear()`
  清空复用，不再循环内反复分配。
- **动态 class 解析改用 `split_ascii_whitespace`** - 生成的 class 迭代现在使用
  `split_ascii_whitespace` 而非 `split_whitespace`。class 名称仅含 ASCII 字符，
  可避免标准迭代器的 Unicode 空白字符扫描开销。
- **动态 class 空字符串快速路径** - 在 fold 之前增加 `is_empty()` 判断，对空字符串
  完全跳过迭代器创建——这在 `class={if cond { "flex" } else { "" }}` 模式中十分常见。

#### 内存/分配改进
- **`parse_class_string` 返回迭代器** - 避免中间 `Vec<TokenStream>` 分配；调用方通过
  `.extend()` 直接消费到输出缓冲区。
- **`generate_attr_methods` 直接 push 到调用方 `Vec`** - 每个属性列表少一次额外 `Vec` 分配。
- **全面使用 `Vec::with_capacity` 预分配** - 属性列表、子节点列表、方法链均以合理容量提示
  预分配。
- **class 字符串转换使用 `Cow<str>`** - 不含连字符时零拷贝借用；仅在需要 `-` → `_` 替换时
  才分配。

#### 二进制体积优化
- **动态 class match 表提取为 `#[inline(never)]` 局部函数** - 同一组件内多个 `class={expr}`
  现共享同一函数体。LLVM ICF 可跨组件合并相同的单态化实例，减少重复内联导致的代码膨胀。
- **常用 class match 分支 thread_local 缓存** - `generate_common_class_matches()` 在整个
  编译进程中只调用一次（通过 `thread_local! + RefCell<Option<Rc<…>>>`），后续调用通过
  `Rc` 共享，成本为 O(1)。

### ♻️ 代码质量改进
- **合并 `EVENT_HANDLERS` 和 `ATTRIBUTE_NAME_MAP`** 为单一 `lookup_attr_method()` match
  函数，移除冗余的第三元组字段（方法名始终等于 snake_case）。
- **通过 `parse_color_with_method(color, method)` 统一颜色解析** - 移除三个近乎相同的实现
  （text_color / bg / border_color），合并为一个函数。
- **`is_stateful_attr()` 改用 `starts_with` + `match`** - 替换了对 `NEEDS_ID_ATTRS` 和
  `EVENT_HANDLERS` 数组的两阶段线性扫描。
- **删除死代码** - 删除已废弃的 `class_dynamic_value_error()` 诊断函数。
- **修复 `generate_for_loop` 类型约束问题**（`element.rs`）- for 循环多子节点情况下原先使用
  定长数组 `[...]` 传入 `flat_map`，要求所有元素类型相同。改为 `vec![...]` 后可正确支持
  同一 for 循环体中出现不同类型元素（如 `div()` 与自定义组件混用）。
- **`parse_arbitrary_hex` 零堆分配 3 位 hex 展开**（`class.rs`）- 将 3 位 hex（`[#rgb]` →
  `[#rrggbb]`）的展开从字符串分配改为位运算。每个半字节通过 `d << 4 | d` 复制，
  不产生任何堆分配。
- **重构 `border_` 条件判断逻辑**（`class.rs`）- 将"空 if 体 + else if"模式替换为显式的
  `is_directional` 绑定和单一 `if !is_directional` 守卫，消除死分支，意图更直接。
- **`parse_condition_tuple` 更惯用的元组提取**（`parser.rs`）- 将"创建 iterator + 两次
  `.next().expect()`"替换为 `Punctuated::pop()`，直接按索引移除元素，无需构造中间迭代器。

### 📖 文档改进
- **动态 class 限制已明确文档化**（`runtime.rs`）- `generate_dynamic_class_code` 文档注释
  和 `_ => el` 通配分支现均明确说明：运行时仅识别约 58 个预编译的常用 class，不在列表中的
  class 会被**静默忽略**。文档中按优先级列出了推荐替代方案（字符串字面量、条件表达式）。
- **`auto_id` 计数器增量编译稳定性说明**（`element.rs`）- `AUTO_ID_COUNTER` 和 `next_auto_id`
  现已文档化已知限制：计数器在单次编译进程中单调递增，若增量编译改变了宏的展开顺序，相同元素的
  自动 ID 可能发生变化。依赖 ID 稳定性做焦点/状态追踪的元素应显式指定 `id` 属性。

### ✅ 测试
- 全部 236 个测试通过（203 宏测试 + 31 覆盖率测试 + 2 诊断测试）
- 优化改动零回归

### 🔒 安全
- **更新依赖** - 将不再维护的 `proc-macro-error` 替换为 `proc-macro-error2`
  - 解决 RUSTSEC-2024-0370 安全建议
  - 消除重复的 `syn 1.x` 依赖树
  - 完全兼容的 API（无缝替换）

---

## [0.2.0] - 2026-02-17

### ✨ 新增功能

#### 核心特性
- **For 循环语法糖** - 使用 `{for item in items { ... }}` 语法简化列表渲染
- **Styled 标志** - 根据标签名称应用合理的默认样式（h1-h6、button、input 等）
- **条件样式** - 使用 `when` 和 `whenSome` 属性实现动态样式
- **Fragment 支持** - 使用 `<>...</>` 语法返回多个根元素
- **完整 Tailwind 调色板** - 242 种内置颜色（slate、gray、red、blue 等）+ 任意十六进制颜色值
- **全面的事件处理** - 支持 onClick、onMouseDown、onKeyDown、onHover 等 15 种事件类型
- **属性映射** - camelCase 到 snake_case 自动转换（zIndex → z_index，fontSize → font_size 等）

#### 文档系统
- 完整文档体系：快速入门、API 参考、最佳实践、迁移指南、故障排除
- 架构文档（`ARCHITECTURE.md` / `ARCHITECTURE_CN.md`）
- 英文为主要 README；中文版移至 `README_CN.md`

#### 基础设施
- GitHub Actions CI、GPUI 兼容性测试和发布自动化工作流
- Codecov 集成和本地覆盖率脚本

#### 开发者体验
- 使用 `proc-macro-error` 依赖提供更好的错误信息
- 更新仓库 URL 为 `https://github.com/wsafight/gpui-rsx`
- 包含综合测试套件

### 🐛 修复
- **有状态事件的自动 ID 注入** - 补充了 `onHover`/`on_hover`、`onDrag`/`on_drag`、
  `onDrop`/`on_drop` 的有状态属性检测，修复了这些事件处理器在没有显式 `id` 属性时
  编译失败的问题。新增 6 个测试用例验证。

### ♻️ 重构
- **消除子节点解析重复** - 提取 `try_parse_child_node()`，消除 `parse_children()` 和
  `parse_for_loop()` 间的重复逻辑。
- **消除 for 循环代码生成重复** - 提取 `generate_for_loop()`，统一 `map`/`flat_map`
  生成逻辑。
- **简化颜色 class 解析** - 将冗余的 `starts_with()` + `strip_prefix()` 替换为单次
  `strip_prefix()` 调用。

### 🗑️ 移除
- `examples/` 目录（需要外部 GPUI 依赖；功能已由测试覆盖）
- `trybuild` 编译失败测试（简化测试结构）

---

## [0.1.2] - 2026-02-16

### 🔧 维护
- 更新仓库 URL
- 注释掉 todo_app 示例（等待依赖解决）

## [0.1.1] - 2026-02-15

### 📖 文档
- 切换为英文作为主要 README
- 添加中文 README（README_CN.md）
- 更新安装说明

## [0.1.0] - 2026-02-15

### 🎉 首次发布
- 基本 RSX 宏实现
- 支持嵌套元素
- 属性支持（布尔属性和值属性）
- 基本 class 属性解析
- 表达式插值
- 事件处理基础功能

---

[未发布]: https://github.com/wsafight/gpui-rsx/compare/v0.10.2...HEAD
[0.10.2]: https://github.com/wsafight/gpui-rsx/compare/v0.10.1...v0.10.2
[0.10.1]: https://github.com/wsafight/gpui-rsx/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/wsafight/gpui-rsx/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/wsafight/gpui-rsx/compare/v0.8.3...v0.9.0
[0.8.3]: https://github.com/wsafight/gpui-rsx/compare/v0.8.2...v0.8.3
[0.8.2]: https://github.com/wsafight/gpui-rsx/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/wsafight/gpui-rsx/compare/v0.8.0...v0.8.1
[v0.8.0 GitHub Release]: https://github.com/wsafight/gpui-rsx/releases/tag/v0.8.0
[0.8.0]: https://github.com/wsafight/gpui-rsx/releases/tag/v0.8.0
[0.7.0]: https://github.com/wsafight/gpui-rsx/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/wsafight/gpui-rsx/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/wsafight/gpui-rsx/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/wsafight/gpui-rsx/compare/v0.4.4...v0.5.0
[0.4.4]: https://github.com/wsafight/gpui-rsx/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/wsafight/gpui-rsx/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/wsafight/gpui-rsx/compare/v0.4.1...v0.4.2
[0.4.0]: https://github.com/wsafight/gpui-rsx/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/wsafight/gpui-rsx/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/wsafight/gpui-rsx/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/wsafight/gpui-rsx/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/wsafight/gpui-rsx/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/wsafight/gpui-rsx/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/wsafight/gpui-rsx/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/wsafight/gpui-rsx/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wsafight/gpui-rsx/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wsafight/gpui-rsx/releases/tag/v0.1.0
