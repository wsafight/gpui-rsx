---
title: CLI 与编辑器
description: 安装 Rust CLI，检查与格式化 RSX，转换 JSX，并接入 VS Code 或 Zed。
---

CLI 与编辑器语言服务共用宏的解析器和 class 配置。安装需要 Rust 1.98 或更新版本；宏库与 core 仍支持 Rust 1.88。

## 安装与环境检查

```sh
cargo install cargo-rsx --version 0.12.0 --locked
cargo rsx doctor --manifest-path demo/Cargo.toml
```

本页使用 `demo/Cargo.toml` 的命令在仓库根目录执行。接入自己的应用时，请换成应用的 manifest 路径。`doctor` 检查实际解析的 Cargo 依赖图，支持别名、Git 依赖和发布快照。

仓库开发版 CLI 还可检查热补丁环境。请传入应用实际使用的 Cargo feature：

```sh
cargo run -p cargo-rsx -- doctor --manifest-path tests/fixtures/hot-reload/Cargo.toml --features hotreload,kit --hot-reload --json
```

检查包括选中后端的 `hot-reload` feature、`dx serve` 是否支持 `--hot-patch`，以及可能
被 Dioxus CLI 0.7.10 拦截的 `rsx` 宏。可用 `RSX_DX` 指定独立的 CLI 安装路径。
补丁阶段仍标为 `unverified`，需要在运行中的应用里确认，步骤见[热更新排查](/gpui-rsx/zh-cn/guides/troubleshooting/)。

## 检查与格式化

```sh
cargo rsx lint --deny-warnings
cargo rsx lint --json
cargo rsx fmt --check
cargo rsx fmt src/main.rs --write
```

`lint` 扫描工作区成员，并读取各包的 `rsx.toml`。间距刻度与宏保持一致：先看 `rsx.toml`，再看 `gpui-rsx` 的 `tailwind-scale` feature；`--tailwind` 可强制使用 Tailwind 刻度。动态 class 的值在运行时检查，CLI 会将这一限制作为提示信息报告。

`fmt` 只改写 RSX 宏体，保留外围 Rust；含注释的宏体保持原样。CI 使用 `--check`，应用变更使用 `--write`。

开发版支持直接导入的宏别名，包括 `use gpui_rsx::{rsx as gpui_view, rsx_tw as tw};`。
lint、格式化、预览与 LSP 补全、悬停和诊断保留原宏的严格模式和间距刻度。
别名遵循块作用域，不会自动进入子模块。同名 `macro_rules!` 从定义处起遮蔽未限定的
RSX 宏调用，lint、格式化与迁移不会处理这些自定义宏；显式的 `gpui_rsx::rsx!` 调用
不受影响。暂不解析其他 crate 的重导出或
`gpui_rsx` 本身的 crate 别名。编辑器的静态语法注入不解析任意导入别名。

Rust 尚未写完时，工具仍处理完整、括号配对的 RSX 宏。lint 与 LSP 在受损位置报告
`rsx/rust-syntax`。恢复次数有上限；未闭合的字符串或注释会遮住文件剩余部分，
不会把其中的内容当作代码。格式化不改写外围未完成的 Rust。
批量写入、CI 格式检查和间距迁移仍先拒绝无法分词的 Rust 源码，再写入任何文件；恢复主要用于
编辑器请求和预览修改。

开发版 JSON 诊断增加可选的 `code`，例如 `rsx/missing-loop-key`、
`rsx/missing-required-attribute` 和 `rsx/missing-component-prop`。core 直接提供语义错误
的位置和修复建议，修改提示文案不会改变修复行为；LSP 也保留相同代码。
原有的 `start`、`end`、`severity`、`message` 和 `suggestion` 字段继续保留。

## 检查主题

```sh
cargo rsx theme check --manifest-path demo/Cargo.toml
cargo rsx theme list --manifest-path demo/Cargo.toml --json
cargo rsx theme explain bg-primary --manifest-path demo/Cargo.toml
```

`check` 验证配置；`list` 输出生成的 class 和 GPUI 方法；`explain` 追溯单个 class 的源令牌和展开结果。三个命令都支持 `--json`。配置示例与状态码见[主题与设计令牌](/gpui-rsx/zh-cn/guides/theming/)。

## 转换与预览

```sh
cargo rsx convert component.tsx
cargo rsx preview src/main.rs --watch
```

转换器使用 SWC，输出包含 RSX 与处理提示的 JSON，并列出不支持的 class、JavaScript 表达式和事件。使用结果前，需要在 Rust 中实现对应行为。

预览在 `http://127.0.0.1:9230` 提供近似静态布局，并标记未执行的行为。`--index N` 可选择指定宏。[在线体验](/gpui-rsx/zh-cn/playground/)在浏览器本地运行相同的解析器、转换器与预览；最终渲染和交互请在原生 GPUI 应用中验证。

两种预览都不执行任意 Rust 表达式或事件处理器。预览和展开成功，不代表生成代码通过
类型检查，也不代表焦点、滚动、无障碍及平台行为已验证。请编译应用并在 GPUI 中验证
这些行为；动态 class 匹配还受其运行时支持范围限制。

## 查看生成的 Rust

```sh
printf '<div class="flex gap-4" />' | cargo rsx expand -
cargo rsx expand body.rsx --manifest-path demo/Cargo.toml
cargo rsx preview demo --index 0 --watch
```

`expand` 接受独立 RSX 宏体，打印完整 Rust 展开；未配置共用 runtime 时会包含内联
动态 helper。`--strict`、`--tailwind` 可覆盖模式。目录预览会查找包中的 Rust 文件，
使用 `--index N` 选择宏；`--watch` 也会在配置变化后刷新。最终类型检查使用 `cargo check`。

## 迁移间距

```sh
cargo rsx migrate src/main.rs           # 先审阅默认输出
cargo rsx migrate src/main.rs --write   # 应用一次
```

迁移将数值 class 除以 4，以便在启用 Tailwind 刻度后保留原来的像素尺寸。应用一次后，启用 `tailwind-scale`，或在 `rsx.toml` 设置 `[options] spacing-scale = "tailwind"`。命令保留 `rsx_tw!`、显式单位和分数；动态字符串与刻度覆盖需要人工复核。已有应用请先阅读[迁移指南](/gpui-rsx/zh-cn/guides/migration/)。

## 接入编辑器

本地原生界面迭代还可使用仓库中的[构建/重启原型](/gpui-rsx/zh-cn/guides/performance/#测量开发重载)，
它提供构建失败恢复和耗时报告。

| 编辑器 | 安装方式 |
| --- | --- |
| VS Code | 从 [0.12.0 Release](https://github.com/wsafight/gpui-rsx/releases/tag/v0.12.0) 下载 `.vsix`，执行 **Extensions: Install from VSIX**。 |
| Zed | 将仓库中的 `editors/zed` 安装为开发扩展；若自定义 Rust 语言服务列表，将 `"gpui-rsx"` 加到 `"rust-analyzer"` 旁边。 |

两个插件均通过 `cargo rsx lsp` 提供：

- 静态 class 补全与展开悬停。
- 诊断、快速修复和主题色块。
- 支持的 class、主题令牌与无效 class 的语义高亮。
- 可选的 Rust 文件内 RSX 格式化。

RSX 格式化默认关闭：它只改写 RSX 宏体、不处理外围 Rust 代码，开启后编辑器会把它当作可替代
rustfmt 的格式化器。在 VS Code 中设置 `"gpuiRsx.format.enable": true`，或在 Zed 中配置
`{"lsp":{"gpui-rsx":{"initialization_options":{"formatting":true}}}}` 即可开启。`cargo rsx fmt`
始终可用。

语言服务增量应用编辑，并复用未变更宏体的分析结果。补全适用于括号完整的 RSX 宏内静态字符串；Rust 类型检查继续由 rust-analyzer 执行。

若 VS Code 找不到服务，设置 `gpuiRsx.serverPath` 为 `cargo-rsx` 可执行文件路径，再执行 **GPUI RSX: Restart language server**。扩展市场的上架状态单独记录在发布说明中。

仓库的隔离 VS Code 集成测试验证 RSX 诊断、补全、格式化与 rust-analyzer 原生 Rust
服务的共存。测试使用 GPUI 签名 stub，并关闭 rust-analyzer 的过程宏展开。
原生 GPUI 行为和完整应用检查单独验证，见[兼容性验证范围](/gpui-rsx/zh-cn/compatibility/#验证范围)。

## 诊断过滤与本文件组件提示

所有命令均支持 `--help`，包括 `theme check/list/explain`。stdin lint 同时接受完整 Rust
文件与裸 RSX 宏体：

```sh
printf '<div class={classes}/>' | cargo rsx lint - --min-severity warning
cargo rsx lint --quiet --deny-warnings
cargo rsx convert component.tsx --tailwind
```

`--min-severity error|warning|information` 过滤文本与 JSON 诊断，默认为 `information`；
`--quiet` / `-q` 仅显示错误。过滤不改变退出状态，`--deny-warnings` 仍会因隐藏的警告失败。
Tailwind 转换保留数值 class 并输出 `rsx_tw! { ... }` 表达式，默认转换仍输出像素刻度宏体。
简单标识符与成员访问会保留；JavaScript 调用、运算符、可选链和事件处理需人工改成 Rust。

`{items}` 等保留的表达式不会收到需要手动转换的诊断。转换器不解析 Rust 类型或所有权；
集合子节点可能需要 `{...items}` 或 `{for ...}`，字段访问也可能需要借用或克隆。

LSP 支持标签、GPUI 方法和当前 Rust 文件内 `#[component]` props 补全（含内联模块），
并为裸文本、闭合标签不匹配、缺必填 prop 和循环缺 key 提供修复。prop / key 修复插入
可编辑的 `todo!` 表达式，运行应用前需填好。该目录不解析导入组件、外部模块、宏或 Rust
类型；请保留 rust-analyzer 完成类型检查。

支持动态文件监听注册的 VS Code / Zed 客户端会收到服务端对 `rsx.toml` 与 `Cargo.toml`
的监听，配置修改后会重新分析已打开的文档。

Playground 可输入可选的 `rsx.toml` 主题（最多 20,000 字符），分享链接包含主题，配置错误
与源码诊断一起展示。静态 token 参与预览，任意 Rust 表达式不会执行。主题中的间距选项
优先于刻度复选框；转换操作按最终有效刻度选择 `rsx_tw!` 输出。
