---
title: Class 处理
description: 静态和动态 class 处理、支持的工具类范围，以及性能建议。
---

`class` 属性会把 Tailwind-like 工具类子集映射为 GPUI builder 方法。

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4 bg-blue-500 text-white">
        {"Content"}
    </div>
}
```

## 静态 Class

class 列表在编译期已知时，优先使用字符串字面量。它们会由宏解析，并直接展开为 GPUI 方法调用：

```rust
rsx! {
    <div class="flex items-center gap-2 px-4 py-2 rounded-md" />
}
```

permissive 模式下，无法安全生成的未知静态 class 会被忽略；无效的任意值仍会编译时报错。strict 模式下，不支持的静态 class 会直接变成编译错误：

```rust
use gpui_rsx::rsx_strict;

rsx_strict! {
    <div class="hover:bg-blue-500" />
}
```

## 编译期条件 Class

如果 `if` 和 `match` 的分支都是字符串字面量，宏也可以在编译期展开：

```rust
rsx! {
    <div class={if active { "bg-blue-500 text-white" } else { "bg-gray-100 text-gray-900" }} />
}

rsx! {
    <div class={match state {
        State::Ready => "border-green-500",
        State::Error => "border-red-500",
        _ => "border-gray-300",
    }} />
}
```

这种写法可以保留状态样式的可读性，同时避免简单字面量分支走动态 class matcher。优化只作用于 `class={...}` 内部的表达式；如果先赋值给变量，再把变量传给 `class`，就会走普通动态 class 路径。

## 动态 Class

当 class 字符串必须在运行时组装时，再使用动态 class：

```rust
let class_name = format!("w-[{}px] bg-{}", width, tone);

rsx! {
    <div class={class_name} />
}
```

动态 class 使用生成的运行时 matcher。matcher 支持常用布局、间距、尺寸、颜色、透明度、排版和边框工具类，也支持文档列出的任意长度和颜色格式。

permissive 模式下，未知动态 class 会被忽略。debug 构建中，每个生成调用点的未知 class 警告只输出一次。strict 模式下，不支持的动态 class 在求值时会 panic。

| 能力 | 静态 class | 动态 class |
| --- | --- | --- |
| 布局和对齐 | 支持 | 支持子集 |
| 数值间距和尺寸 | 支持 | 支持 |
| 任意长度 | 支持 | 支持 |
| 百分比和分数尺寸 | 支持 | 支持 |
| 完整 Tailwind 色板 | 支持 | 支持 |
| 任意 hex / RGB / RGBA 颜色 | 支持 | 支持 |
| 静态有状态滚动 class | 支持并自动生成 ID | 不支持 |
| `hover:*` 等 Tailwind variant | 忽略或 strict 报错 | 忽略或 strict panic |

## 支持范围

| 范围 | 说明 |
| --- | --- |
| 布局 | `flex`、`flex-col`、`flex-row`、`flex-wrap`、`grid`、`hidden`、`absolute`、`relative`、grow/shrink 辅助类。 |
| 对齐 | GPUI 支持的 `items-*`、`justify-*`、`content-*`、`self-*`。 |
| 间距 | `gap-*`、`gap-x-*`、`gap-y-*`、`p-*`、`px-*`、`m-*` 以及方向变体。 |
| 尺寸 | `w-*`、`h-*`、`size-*`、`min-*`、`max-*`、`*-full`、`*-auto`、百分比、分数和任意长度。 |
| 颜色 | 完整 Tailwind 色名，以及任意 `hex`、`rgb`、`rgba` 形式。 |
| 排版 | 字号、字重、文字对齐、装饰、截断、whitespace 和 line clamp。 |
| 边框 | 宽度、方向宽度、颜色、圆角和 dashed border。 |
| 其他 | cursor helper、`debug-outline`、阴影、透明度、grid placement alias。 |

## 长度语义

GPUI-RSX 的数值工具类使用直接像素语义：

```rust
rsx! {
    <div class="w-64 gap-4" />
}
```

它会展开为类似 `w(px(64.0)).gap(px(4.0))`，不会使用 Tailwind CSS 的 rem 间距比例。

任意长度示例：

```rust
rsx! {
    <div class="w-[280px] h-[18rem] max-w-[37.5%] gap-[14px]" />
}
```

百分比和分数支持在尺寸类中使用：

```rust
rsx! {
    <aside class="w-6/24 max-w-[40%]" />
}
```

`gap-[10%]` 这类百分比间距会被拒绝，因为 GPUI 间距 API 需要确定长度。

## 颜色格式

命名 Tailwind 颜色使用 `family-shade`：

```rust
rsx! {
    <div class="bg-blue-500 text-white border-gray-300" />
}
```

支持的色系包括 `slate`、`gray`、`zinc`、`neutral`、`stone`、`red`、`orange`、`amber`、`yellow`、`lime`、`green`、`emerald`、`teal`、`cyan`、`sky`、`blue`、`indigo`、`violet`、`purple`、`fuchsia`、`pink`、`rose`，色阶为 `50` 到 `950`，另有 `black` 和 `white`。

任意颜色支持 hex、短 hex、RGB 和 RGBA：

```rust
rsx! {
    <div class="bg-[#ff0000] text-[#f00] border-[rgba(15,23,42,0.35)]" />
}
```

## Strict 与预览宏

需要让未知静态 class 直接失败时，使用 `rsx_strict!`：

```rust
use gpui_rsx::{rsx_expand, rsx_permissive, rsx_strict};

rsx_strict! { <div class="flex w-[280px]" /> }
rsx_permissive! { <div class="hover:bg-blue-500 flex" /> }

let preview = rsx_expand! {
    <div class="flex gap-4 bg-blue-500" />
};
```

`rsx_expand!` 会返回生成的 GPUI builder 链字符串，适合调试宏展开；它不会对生成的 GPUI 表达式做类型检查。

## 性能建议

- 稳定 UI 结构使用字面量 class。
- 少量状态分支使用字面量 `if` 或 `match`。
- 只有尺寸或颜色确实运行时变化时再生成动态 class。
- 热路径中，如果值本来就是 Rust 表达式，优先使用直接 GPUI 属性，例如 `w={px(width)}`。
- 少量条件变体优先使用 `when` 或 `whenClass`，不要拼接很长的运行时 class 字符串。
