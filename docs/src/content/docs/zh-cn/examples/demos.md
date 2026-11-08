---
title: 可运行示例
description: 选择原生 GPUI 示例，运行交互，并查找对应 API。
---

`demo/` crate 提供九个桌面示例，使用本地 GPUI-RSX 0.11 源码和锁定的 GPUI Kit 依赖。可以先用小示例了解单项 API，再用控制台观察多个面板之间的状态变化。

## 选择示例

在仓库根目录运行下面的命令，将 `counter` 换成表中的名称：

```sh
cargo run --manifest-path demo/Cargo.toml --bin counter --locked
```

该 crate 固定使用 Rust 1.98.1。构建前请安装对应平台的原生依赖，版本组合见[兼容性](/gpui-rsx/zh-cn/compatibility/)。

| 二进制名称 | 可以尝试的操作 | 对应写法 |
| --- | --- | --- |
| `hello` | 打开最小视图。 | 静态 RSX、语义主题色、显式像素间距。 |
| `counter` | 增加、减少，再重置数值。 | `rsx_listener!`、`rsx_update!`、条件 class、原生 Button 禁用状态。 |
| `palette` | 对照色块、class 名称和颜色值。 | 动态 class 表达式。 |
| `task_list` | 完成或重新打开指定任务。 | 重复有状态控件的 key、捕获任务 ID、派生完成数量。 |
| `component` | 在输入框打字，再打开 Popover。 | Kit Button/Input/Popover、函数组件、上下文注入、children prop。 |
| `syntax_gallery` | 切换条件、反转列表，再滚动查看其他语法。 | Fragment、spread、带 key 的循环、组件、状态 variant、负边距、SVG/图片/canvas。 |
| `api_surface` | 切换条件样式、悬停查看 group 状态。 | Flex 别名、无障碍属性、tooltip、group 状态。 |
| `project_dashboard` | 筛选任务、切换密度、完成任务、添加后续工作。 | 共享 token、选择同步、条件详情 Fragment、空状态恢复。 |
| `incident_console` | 筛选、认领、推进事故状态并注入信号。 | 多面板拆分、共享动态 class runtime、生命周期方法、时间线、空状态。 |

## 当前版本能力的用法

计数器的增减操作使用 `rsx_listener!(cx, notify, ...)`；重置操作使用由 handler 负责通知的 listener，再调用 `rsx_update!`。这样每次操作只通知一次。这些辅助宏是可选写法，直接使用 `cx.listener` 仍然有效。

语法示例通过 `rsx_fragment!` 和 spread 添加子节点，并用重叠头像展示 0.11 的负边距 class。负尺寸有不同的布局规则：GPUI 会将负宽高、负最大尺寸限制为零，并忽略负最小尺寸。支持范围见 [Class 处理](/gpui-rsx/zh-cn/usage/class/)。

`demo/rsx.toml` 启用了 Kit 主题预设和共享动态 runtime。基础示例使用语义主题色，两个控制台使用同一组项目配色 token。固定布局尽量使用静态 class，动态字符串支持的工具类范围更小。

## 不开窗口检查行为

```sh
cargo +1.98.1 test --manifest-path demo/Cargo.toml --lib --bins --tests --locked
cargo +1.98.1 test --manifest-path demo/Cargo.toml --doc --locked
```

测试覆盖模型状态转换、真实 GPUI 布局、ID、回调和语法拒绝行为，不比较截图。`api_contract`、`generated_api_contract`、`semantics_contract` 是编译契约，不属于可交互示例。

需要修改后自动重启应用时，可使用[开发重载流程](/gpui-rsx/zh-cn/guides/performance/#测量开发重载)。每次重启都会重置应用状态。
