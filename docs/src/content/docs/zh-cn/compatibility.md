---
title: 兼容性
description: GPUI、gpui-component、Rust 和 demo lockfile 的兼容性说明。
---

GPUI-RSX `0.6.x` 目标是 Zed 仓库中的 GPUI，而不是 crates.io 上发布的 `gpui = "0.2.2"` 包。

Zed git 依赖当前仍会把 crate 包版本显示为 `gpui v0.2.2`，但它的 API 面与 crates.io release 不同。所以即使 `cargo tree` 显示 `gpui v0.2.2`，只要 source 是 Zed git，项目也可能是正确配置。

## 当前 Demo 矩阵

| 项目 | 值 |
| --- | --- |
| GPUI-RSX | `0.6.0` |
| GPUI source | `https://github.com/zed-industries/zed` |
| GPUI resolved revision | `e973593455af18719be22b0455c3f928c6ccc24d` |
| `gpui-component` revision | `7885c41663c7a6cc68ad0c99b1ba33550f807ff0` |
| `gpui-base` revision | `7885c41663c7a6cc68ad0c99b1ba33550f807ff0` |
| GPUI-RSX 根 crate MSRV | `1.88.0` |
| Demo Rust toolchain | `1.95.0` |
| Demo 检查命令 | `cargo +1.95.0 check --manifest-path demo/Cargo.toml --bins --locked` |

根 crate 的 MSRV 是最低版本，不是日常开发必须固定的工具链。可以使用更新的 stable Rust；
当前锁定的 demo revision 也已使用 Rust `1.98.0` 验证。Rust `1.95.0` 继续作为 demo 的显式
可复现验证通道。

## 避免重复 GPUI Crate

使用 `gpui-component` 时，应保证应用中所有直接 GPUI 依赖与 `gpui-component` 解析到同一个 source 和 revision。

常见错误依赖图通常是：

- 一个 `gpui` 来自 bare Zed git 依赖；
- 另一个 `gpui` 来自带 `rev` 的 pinned 依赖；
- 组件类型来自不同 crate 实例，最终类型不兼容。

提交应用的 lockfile，并参考 demo 的依赖形态，可以让依赖图保持确定。

当前 gpui-component 主线已将底层控件拆到 `gpui-base`。应用通常仍只依赖
`gpui-component` facade，并通过其公开 re-export 使用受支持的基础类型。只有 facade
没有导出所需 API 时才直接依赖 `gpui-base`，且必须保持相同 source revision。

## 查看实际解析到的 GPUI

```bash
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui
```

如果 source 是 `git+https://github.com/zed-industries/zed#...`，即使显示包版本为 `0.2.2`，demo 使用的也是 Zed git GPUI。
