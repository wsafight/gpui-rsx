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
| GPUI resolved revision | `f164afda46188939f76c24aba4099d04423bc356` |
| `gpui-component` revision | `8752104289424b7f35045b68a2d394018da48e7e` |
| Demo Rust toolchain | `1.95.0` |
| Demo 检查命令 | `cargo check --manifest-path demo/Cargo.toml --bins --locked` |

## 避免重复 GPUI Crate

使用 `gpui-component` 时，应保证应用中所有直接 GPUI 依赖与 `gpui-component` 解析到同一个 source 和 revision。

常见错误依赖图通常是：

- 一个 `gpui` 来自 bare Zed git 依赖；
- 另一个 `gpui` 来自带 `rev` 的 pinned 依赖；
- 组件类型来自不同 crate 实例，最终类型不兼容。

提交应用的 lockfile，并参考 demo 的依赖形态，可以让依赖图保持确定。

## 查看实际解析到的 GPUI

```bash
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui
```

如果 source 是 `git+https://github.com/zed-industries/zed#...`，即使显示包版本为 `0.2.2`，demo 使用的也是 Zed git GPUI。
