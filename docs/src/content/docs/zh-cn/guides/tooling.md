---
title: CLI 与编辑器
description: 安装 Rust CLI，检查与格式化 RSX，转换 JSX，并接入 VS Code 或 Zed。
---

CLI 与编辑器语言服务共用宏的解析器和 class 配置。安装需要 Rust 1.98 或更新版本；宏库与 core 仍支持 Rust 1.88。

## 安装与环境检查

```sh
cargo install cargo-rsx --version 0.9.0 --locked
cargo rsx doctor --manifest-path demo/Cargo.toml
```

本页使用 `demo/Cargo.toml` 的命令在仓库根目录执行。接入自己的应用时，请换成应用的 manifest 路径。`doctor` 检查实际解析的 Cargo 依赖图，支持别名、Git 依赖和发布快照。

## 检查与格式化

```sh
cargo rsx lint --deny-warnings
cargo rsx lint --json
cargo rsx fmt --check
cargo rsx fmt src/main.rs --write
```

`lint` 扫描工作区成员，并读取各包的 `rsx.toml`。若间距刻度来自 Cargo feature，传入 `--tailwind`。动态 class 的值在运行时检查，CLI 会将这一限制作为提示信息报告。

`fmt` 只改写 RSX 宏体，保留外围 Rust；含注释的宏体保持原样。CI 使用 `--check`，应用变更使用 `--write`。

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

## 迁移间距

```sh
cargo rsx migrate src/main.rs           # 先审阅默认输出
cargo rsx migrate src/main.rs --write   # 应用一次
```

迁移将数值 class 除以 4，以便在启用 Tailwind 刻度后保留原来的像素尺寸。应用一次后，启用 `tailwind-scale`，或在 `rsx.toml` 设置 `[options] spacing-scale = "tailwind"`。命令保留 `rsx_tw!`、显式单位和分数；动态字符串与刻度覆盖需要人工复核。已有应用请先阅读[迁移指南](/gpui-rsx/zh-cn/guides/migration/)。

## 接入编辑器

| 编辑器 | 安装方式 |
| --- | --- |
| VS Code | 从 [0.9.0 Release](https://github.com/wsafight/gpui-rsx/releases/tag/v0.9.0) 下载 `.vsix`，执行 **Extensions: Install from VSIX**。 |
| Zed | 将仓库中的 `editors/zed` 安装为开发扩展；若自定义 Rust 语言服务列表，将 `"gpui-rsx"` 加到 `"rust-analyzer"` 旁边。 |

两个插件均通过 `cargo rsx lsp` 提供：

- 静态 class 补全与展开悬停。
- 诊断、快速修复和主题色块。
- 支持的 class、主题令牌与无效 class 的语义高亮。
- Rust 文件内的 RSX 格式化。

语言服务增量应用编辑，并复用未变更宏体的分析结果。补全适用于括号完整的 RSX 宏内静态字符串；Rust 类型检查继续由 rust-analyzer 执行。

若 VS Code 找不到服务，设置 `gpuiRsx.serverPath` 为 `cargo-rsx` 可执行文件路径，再执行 **GPUI RSX: Restart language server**。扩展市场的上架状态单独记录在发布说明中。
