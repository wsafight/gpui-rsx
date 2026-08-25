---
title: GPUI-RSX
description: 用 JSX-like 语法编写 GPUI 视图的编译期 RSX 宏。
---

GPUI-RSX 是一个 Rust 过程宏，会在编译期把 JSX-like 标记转换成原生 GPUI builder 调用。

它适合希望保留 Rust 类型安全和 GPUI 性能模型，同时让 UI 代码更接近界面结构的项目。

## 提供什么

- HTML-like 标签，展开为 GPUI 元素。
- 以属性语法调用 GPUI builder 方法。
- `class="..."` 支持 Tailwind-like 子集，并映射到 GPUI API。
- 在 `{ ... }` 中使用 Rust 表达式和条件渲染。
- 使用 `<>...</>` 返回 Fragment。
- 为重复的有状态元素提供 `key={...}` 循环安全机制。
- 提供 strict、permissive 和展开预览宏。

## 不是什么

GPUI-RSX 不是浏览器渲染器，也不是 Tailwind CSS 运行时。静态 RSX 会被编译成 Rust 代码。动态 class 会通过生成的运行时 matcher 处理，并只支持文档中列出的常用 GPUI 工具类子集。

## 主要宏

```rust
use gpui_rsx::{rsx, rsx_expand, rsx_permissive, rsx_strict};
```

| 宏 | 用途 |
| --- | --- |
| `rsx!` | 默认 permissive 模式 RSX 宏。 |
| `rsx_strict!` | 不支持的静态 class 编译时报错；不支持的动态 class 在求值时 panic。 |
| `rsx_permissive!` | 显式 permissive 模式。 |
| `rsx_expand!` | 返回生成 GPUI builder 代码的字符串预览，方便调试。 |

## 文档地图

Starlight 站点现在是唯一维护的文档集。原来的 `docs/*.md` 内容已合并到以下页面：

- [快速开始](/gpui-rsx/zh-cn/getting-started/)：安装和第一个视图。
- [兼容性](/gpui-rsx/zh-cn/compatibility/)：GPUI、gpui-component、Rust 和 demo lockfile 矩阵。
- [语法参考](/gpui-rsx/zh-cn/usage/syntax/)：元素、属性、子节点和条件渲染。
- [Class 处理](/gpui-rsx/zh-cn/usage/class/)：静态和动态工具类。
- [ID 与 Key](/gpui-rsx/zh-cn/usage/ids/)：自动 ID、循环安全和显式 ID。
- [性能优化](/gpui-rsx/zh-cn/guides/performance/)：静态展开、动态 class 成本、render 循环分配和验证命令。
- [最佳实践](/gpui-rsx/zh-cn/guides/best-practices/)：组织和日常模式建议。
- [迁移指南](/gpui-rsx/zh-cn/guides/migration/)：从手写 GPUI 迁移，以及 `0.6.x` 的 GPUI 目标变化。
- [问题排查](/gpui-rsx/zh-cn/guides/troubleshooting/)：解析、类型、运行时和依赖问题。
- [API 参考](/gpui-rsx/zh-cn/reference/api/)：宏和映射表速查。
- [发布检查清单](/gpui-rsx/zh-cn/reference/release-checklist/)：发布验证和包内容检查。
