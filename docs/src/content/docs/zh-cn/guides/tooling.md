---
title: CLI 与编辑器
description: 检查、格式化、迁移和预览 RSX，并接入 VS Code、Zed。
---

使用 Rust 1.98 或更新版本安装 CLI；宏库仍支持 Rust 1.88：

```sh
cargo install cargo-rsx --version 0.8.0
cargo rsx doctor --manifest-path demo/Cargo.toml
cargo rsx lint --deny-warnings
cargo rsx lint --json
cargo rsx fmt --check
cargo rsx fmt src/main.rs --write
cargo rsx migrate src/main.rs                 # 默认仅展示变更
cargo rsx convert component.tsx              # JSON：RSX 和人工处理提示
cargo rsx preview src/main.rs --watch        # http://127.0.0.1:9230
```

`doctor` 检查实际 Cargo 依赖图，支持别名、Git 依赖和发布快照。
`lint` 扫描工作区成员并读取各包的 `rsx.toml`；若刻度来自 Cargo feature，传入 `--tailwind`。
动态 class 的值在运行时检查，CLI 会提示这一限制。

`fmt` 只改 RSX 宏体，保留外围 Rust；含注释的宏体保持原样。CI 使用 `--check`，
应用变更使用 `--write`。

`migrate` 将数值 class 除以 4，以保留原先的像素尺寸。先审阅默认输出，
再用 `--write` 执行一次，随后启用 `tailwind-scale` 或在 `rsx.toml` 中设置
`[options] spacing-scale = "tailwind"`。`rsx_tw!`、显式单位和分数不会改写；
动态字符串和自行覆盖的刻度需要人工复核。

JSX/TSX 转换使用 SWC，输出 RSX 并列出不支持的 class、JavaScript 表达式和事件。
预览近似展示静态布局，并标记未执行的行为；`--index N` 选择文件中的第 N 个宏。
[Playground](../../playground/) 在浏览器本地运行相同的解析器、转换器与预览。

## 编辑器

从 GitHub Release 下载 VSIX，在 VS Code 中执行 **Extensions: Install from VSIX**。
若 PATH 中没有 `cargo-rsx`，设置 `gpuiRsx.serverPath`，再执行
**GPUI RSX: Restart language server**。

Zed 可将 `editors/zed` 安装为开发扩展。若自定义了 Rust 的语言服务列表，
将 `"gpui-rsx"` 加到 `"rust-analyzer"` 旁边。市场上架状态单独记录在发布说明中。

两个插件均通过 `cargo rsx lsp` 提供 class 补全、展开悬停、诊断、快速修复、色块和格式化。
补全适用于括号完整的 RSX 宏内静态 class 字符串，Rust 类型检查继续由 rust-analyzer 执行。
