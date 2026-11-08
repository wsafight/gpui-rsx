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
| `0.12.1` | `0.9.0` | `gpui-pre@0.3.11` | `gpui-component@0.9.0` |

此矩阵来自 demo/Cargo.lock。Kit 统一提供 GPUI、组件和资源；以下 Cargo 别名保留 RSX 展开所需的 gpui:: 路径。请提交应用的 lockfile。

```toml
[dependencies]
gpui = { package = "gpui-kit", version = "=0.9.0" }
gpui-rsx = "0.12.1"
```
<!-- gpui-rsx:compat:end -->

## 验证范围

| 目标 | 验证方式 | 覆盖边界 |
| --- | --- | --- |
| 矩阵中锁定的 Kit 快照 | Linux/macOS CI 中的生成 API 合约、demo 构建、测试和 doctest | 已映射 GPUI 方法及文档中 Kit 集成方式的主要目标 |
| 改名后的 Kit 依赖 | demo CI 中的独立 fixture，不声明 `gpui` crate 别名 | 配置 GPUI 路径、组件、ID、Fragment、内联动态 class 与共享运行时 |
| 官方 `gpui = "=0.2.2"` | 独立 macOS fixture，共用回归用例 | key、组件默认值、Fragment、事件、身份标识与状态更新，不覆盖完整生成 API 合约 |
| 其他版本、来源或后端 | 本矩阵未覆盖 | 接入前独立验证应用依赖图、生成代码和实际行为 |

宏不依赖 GPUI 运行时，不代表兼容任意 GPUI 版本。需要保留默认的 `gpui` 依赖名或配置 `gpui-path`，
导入相关 trait，并采用对应组件的构造方式。`doctor` 和 `check-gpui` 检查解析后的依赖图；
检查通过不代表 RSX 已通过类型检查，也不能代替应用测试。

先用锁定组合建立可工作的基线，再逐项调整依赖。报告未验证组合的兼容问题时，请提供
manifest、lockfile、最小失败 RSX 和编译器输出。

## 已有依赖别名

优先采用上面的 `gpui = { package = "gpui-kit", ... }` 写法。应用已有名为 `engine`
的依赖时，可以在 crate 根部提供生成代码所需的名称：

```toml
[dependencies]
engine = { package = "gpui-kit", version = "=0.9.0" }
```

```rust
extern crate engine as gpui;
```

默认配置下，使用 RSX 的每个 binary、example 或集成测试 target 都需在自己的 crate
根部声明该别名。也可以在项目的 `rsx.toml` 中配置绝对 GPUI 路径：

```toml
[options]
gpui-path = "::engine"
```

该选项覆盖标记、组件、自动 ID、动态 key、`rsx_listener!`、`rsx_update!` 和
`rsx_runtime!()` 中的生成路径。crate 根部的重新导出也可使用 `crate::ui`。
路径不能包含泛型参数或以 `self`、`super` 开头；采用 crate 内重新导出时，每个
target 都应提供对应导出。显式的 `rsx_runtime!(::other_engine)` 优先于此配置。

配置路径后，`#[component]` 函数通过 `ViewElement` 实现 `IntoElement`，因此路径必须指向提供它的
GPUI。使用 `Component` 的 GPUI 0.2.x 仅在默认 `gpui` 依赖名下支持这些生成的函数组件。

宏展开会追踪已经存在的 `rsx.toml`。若要自动识别首次编译后新增的配置，请在使用宏的
包中添加以下 `build.rs`，已有构建脚本时则合并这条指令：

```rust
fn main() {
    println!("cargo:rerun-if-changed=rsx.toml");
}
```

Demo 已接入该脚本，无需修改 Rust 源码即可识别配置的新增、修改、删除和重新创建。
被追踪的文件缺失时，Cargo 会重复构建；暂时禁用配置时可保留空的 `rsx.toml`，避免
重复构建。未接入构建脚本的项目，首次新增配置后需要修改一次 Rust 源码以触发宏展开。

用户表达式、显式 prop 类型和 GPUI 自身的 derive 宏仍需各自的导入。
例如上游 `#[derive(engine::IntoElement)]` 可能仍生成 `gpui::` 引用，因而需要用户别名。
`#[component]` 生成的函数组件直接使用配置路径。以下 fixture 无需 `gpui` 别名：

```bash
cargo +1.98.1 test --manifest-path tests/fixtures/gpui-renamed/Cargo.toml --target-dir demo/target
```

## 当前 Demo 矩阵

| 项目 | 值 |
| --- | --- |
| 渲染后端 | Kit 默认的 `gpui-pre`，通过 crates.io 获取 |
| GPUI-RSX 根 crate MSRV | `1.88.0` |
| Demo Rust toolchain | `1.98.1` |
| Demo 检查命令 | `cargo +1.98.1 check --manifest-path demo/Cargo.toml --bins --locked` |

根 crate 的 MSRV 与 demo 的工具链分开管理；编写宏无需安装 GPUI 平台依赖。

## 官方 GPUI 0.2.2 回归检查

独立的 macOS fixture 使用 `gpui = "=0.2.2"`，启用 `font-kit`、`runtime_shaders`
与 `test-support`，以及 GPUI-RSX 的 `tailwind-scale`，编译并运行动态字符串 key、组件
数值默认值、混合根节点 Fragment、监听器签名、身份标识和显式状态更新回归。
它与 Kit demo 测试共用回归源码：

```bash
cargo test --manifest-path tests/fixtures/gpui-0.2.2/Cargo.toml
```

此检查覆盖上述反馈中的宏行为；完整生成的 GPUI API 仍以上方矩阵中的 Kit 快照为目标。

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
