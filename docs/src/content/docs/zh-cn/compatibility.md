---
title: 兼容性
description: GPUI Kit、gpui-pre、gpui-component、Rust 和 demo lockfile 的兼容性说明。
---

当前验证目标是 GPUI Kit 及其固定的 `gpui-pre` 快照。`gpui-pre-*` 是 Longbridge
发布的 Zed GPUI 快照包；其版本号与官方 `gpui` crate 独立。

宏与 core crate 不依赖 GPUI 运行时。demo 将 `gpui-kit` 命名为 `gpui`，保留
RSX 展开与 GPUI derive 使用的 `gpui::` 路径。

<!-- gpui-rsx:compat:start -->
### 已验证的兼容版本

| GPUI-RSX | GPUI Kit | GPUI snapshot | Component |
| --- | --- | --- | --- |
| `0.8.3` | `0.8.0` | `gpui-pre@0.3.9` | `gpui-component@0.8.0` |

此矩阵来自 demo/Cargo.lock。Kit 统一提供 GPUI、组件和资源；以下 Cargo 别名保留 RSX 展开所需的 gpui:: 路径。请提交应用的 lockfile。

```toml
[dependencies]
gpui = { package = "gpui-kit", version = "=0.8.0" }
gpui-rsx = "0.8.3"
```
<!-- gpui-rsx:compat:end -->

## 当前 Demo 矩阵

| 项目 | 值 |
| --- | --- |
| 渲染后端 | Kit 默认的 `gpui-pre`，通过 crates.io 获取 |
| GPUI-RSX 根 crate MSRV | `1.88.0` |
| Demo Rust toolchain | `1.98.1` |
| Demo 检查命令 | `cargo +1.98.1 check --manifest-path demo/Cargo.toml --bins --locked` |

根 crate 的 MSRV 与 demo 的工具链分开管理；编写宏无需安装 GPUI 平台依赖。

## 保持同一份 GPUI 类型

通过 `gpui::component`、`gpui::base` 和 `gpui::platform` 使用 Kit 的导出。
在打开窗口前调用 `gpui::init(cx)`，使用 `gpui::open_window(...)` 挂载根视图；
图标资源可通过 `application().with_assets(gpui::assets::Assets)` 加载。

应用无需再直接添加旧 Zed Git 的 `gpui`、`gpui_platform` 或旧 Git 组件库。
每个 crate 实例拥有独立的 Rust 类型；同名类型来自不同 GPUI 实例时无法互换。

```bash
cargo xtask check-gpui
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui-pre
```

`check-gpui` 检查唯一的渲染引擎、配套 platform/macros 快照版本，以及 Kit、base、
component 是否引用同一份 GPUI。它输出已验证的包版本 JSON，供 CI 使用。

## 更新配套版本

随 Kit 的正式发布更新依赖。兼容性 workflow 选择 crates.io 中最新的非撤回正式版本，
修改 demo 的精确 Kit 版本，再更新配套 lockfile、API 表、快照、规范及本矩阵。
Linux 和 macOS 对同一份补丁验证通过后才创建修复 PR；失败时更新单一追踪 Issue。

GPUI Fast 是可选后端，当前 demo 和兼容性矩阵验证 Kit 默认的 gpui-pre 后端。
