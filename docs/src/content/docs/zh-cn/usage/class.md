---
title: Class 处理
description: 静态和动态 class 处理、支持的工具类范围，以及性能建议。
---

`class` 属性把一部分 Tailwind 风格的工具类映射为 GPUI builder 方法。固定样式优先写成静态字符串；依赖应用状态的样式再使用动态表达式。

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

permissive 模式下，不能展开的静态 class 会给出编译期警告；未知标识符形式也会警告并透传为 GPUI 方法，疑似拼写错误附带建议。第三方扩展方法建议改用显式属性（如 `h_flex`）或 `base={...}`。无效的任意值仍会编译时报错。strict 模式下，不支持的静态 class 会直接变成编译错误：

CSS 专属类别不走透传：`z-*`、`space-x-*`、`space-y-*`、`divide-*`、`transition*`、
`duration-*`、`ease-*`、`delay-*` 和 `animate-*` 在宽松模式下会警告并忽略，即使扩展 trait
定义了对应方法。调用自定义实现时使用显式属性或 `base={...}`。

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

## 状态 Class 属性

状态样式可以用静态 class 表达时，使用 `hoverClass`、`focusClass` 和 `activeClass`：

```rust
rsx! {
    <button
        class="px-4 py-2 rounded-md bg-blue-500 text-white"
        hoverClass="bg-blue-600"
        focusClass="border-blue-500"
        activeClass="opacity-75"
    />
}
```

这些属性会展开为 GPUI `StyleRefinement` 闭包。它们只接受字符串字面量；`overflow-scroll` 或 `debug-outline` 这类需要元素级 stateful 行为的 class 会产生编译错误。由于当前 GPUI 的 `active` hook 是 stateful，`activeClass` 会自动注入 ID；`hoverClass` 和 `focusClass` 不会。

## 动态 Class

当 class 字符串必须在运行时组装时，再使用动态 class：

```rust
let class_name = format!("w-[{}px] bg-{}", width, tone);

rsx! {
    <div class={class_name} />
}
```

动态 class 使用生成的运行时 matcher。matcher 支持常用布局、间距、尺寸、颜色、透明度、排版和边框工具类，也支持文档列出的任意长度和颜色格式。

permissive 模式下，未知动态 class 会被忽略。debug 构建中，每个生成调用点的未知 class 警告只输出一次。strict 模式下，不支持的动态 class 默认 panic，可通过 `onUnknownClass={|class: &str| ...}` 自定义处理。开启 `dynamic-class-report` 后，debug 提示会在进程退出时汇总。

| 能力 | 静态 class | 动态 class |
| --- | --- | --- |
| 布局和对齐 | 支持 | 支持子集 |
| 数值间距和尺寸 | 支持 | 支持 |
| 任意长度 | 支持 | 支持 |
| 百分比和分数尺寸 | 支持 | 支持 |
| 完整 Tailwind 色板 | 支持 | 支持 |
| 任意 hex / RGB / RGBA 颜色 | 支持 | 支持 |
| 静态有状态滚动 class | 支持并自动生成 ID | 不支持 |
| `hover:` / `focus:` / `active:` / group 状态前缀 | 支持 | 不支持 |
| 响应式 / `dark:` 前缀 | 警告或 strict 报错 | debug 提示或 strict panic / 自定义回调 |

## 支持范围

| 范围 | 说明 |
| --- | --- |
| 布局 | `flex`、`flex-col`、`flex-row`、`flex-wrap`、`grid`、`hidden`、`absolute`、`relative`、grow/shrink 辅助类。 |
| 对齐 | GPUI 支持的 `items-*`、`justify-*`、`content-*`、`self-*`。 |
| 间距 | `gap-*`、`gap-x-*`、`gap-y-*`、`p-*`、`px-*`、`m-*` 以及方向变体。间距与尺寸家族支持 `-m-4`、`-mt-1.5` 这样的负值。 |
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

## 负值

GPUI 自带 `*_neg_*` 预设的间距与尺寸家族支持前导负号，字面量 class、状态前缀和动态
 class 表达式均可用：

```rust
rsx! {
    <div class="-m-2 -mt-1.5 -ml-[13px] -p-1 hover:-gap-1" />
}
```

负值 class 展开为取负的长度，例如 `-m-4` 是 `.m(px(-4.0))`（Tailwind 刻度下为 -16px），
预设值如 `-m-px` 则调用 GPUI 的 `m_neg_px()` 方法。`-auto` 与 `-full` 仍不受支持，
因为负的 `auto` / `full` 没有意义；`rounded-*` / `leading-*` 会被拒绝，因为 GPUI
没有为它们生成负值预设。未知的负值 class 会给出具体原因，而不是推荐符号相反的正值 class。

负值的实际效果由 GPUI 布局引擎决定。下表对应 `demo/tests/negative_layout.rs` 中的
固定尺寸 flex 布局；正最小尺寸、flex 增长和内容尺寸等其他约束可能改变最终边界。

| class | 布局结果 |
| --- | --- |
| `-m-*`、`-mx-*`、`-my-*`、`-mt-*`、`-ml-*` | 元素及其后续兄弟向负方向移动，与相邻元素重叠（`-mt-5` 使其上移 5px）。 |
| `-mb-*`、`-mr-*` | 沿布局方向把下一个兄弟拉近；在交叉轴上不产生位移。 |
| `-gap-*` | 兄弟元素按该数值相互靠拢。 |
| `-gap-x-*`、`-gap-y-*` | 同上，但只作用于对应轴（`gap-x` 用于横排，`gap-y` 用于竖排）。 |
| `-p-*`、`-px-*`、`-py-*`、`-pt-*`、`-pl-*` | 内容向外偏移，超出容器边缘。 |
| `-pb-*`、`-pr-*` | 只改变远端边缘，子元素不移动。 |
| `-w-*`、`-h-*`、`-size-*`、`-max-w-*`、`-max-h-*`（包括 `-w-1/2` 这类分数） | 被钳制为 `0`，元素塌缩。 |
| `-min-w-*`、`-min-h-*` | 没有效果。 |

上述结果由 `demo/tests/negative_layout.rs` 固定，它在 GPUI 真实布局引擎上逐个家族验证。
尺寸类之所以被接受，是因为 GPUI 提供了对应方法，但它们几乎没有实用价值，因此静态字面量
如 `-w-4`、`-min-w-4` 在编译时会给出说明结果的警告。strict 模式保留警告级别，
但 Rust 的 `#[deny(deprecated)]` / `-D warnings` 和 `cargo rsx lint --deny-warnings`
仍可使验证失败。`cargo rsx lint` 与语言服务器报告同样的警告。判断依据是解析后的长度：
`w-[-4px]` 会提示，而 `-w-[-4px]` 的结果是正的 4px，不会误报；零尺寸也不会触发
负尺寸警告。`-mt-px` 这类预设值需要使用静态字面量：动态运行时匹配器不解析
`px`、`0p5` 这类预设，无论正值还是负值。

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

错误处理与数值刻度分别选择：

| 需求 | 入口 | 行为 |
| --- | --- | --- |
| 常规 GPUI 布局 | `rsx!` | 宽松处理 class；未配置时使用像素刻度 |
| 拒绝不支持的 class | `rsx_strict!` | 静态 class 编译时报错；未处理的未知动态 token 会 panic |
| 显式允许不支持的 class | `rsx_permissive!` | class 策略与默认 `rsx!` 相同，保留已配置的刻度 |
| 使用 Tailwind 数值间距 | `rsx_tw!` | 宽松处理 class，数值单位为 4px；不会启用 strict 检查 |
| 检查生成代码 | `rsx_expand!` | 返回字符串，不做类型检查或渲染 |

同时需要 strict 检查与 Tailwind 刻度时，使用 `rsx_strict!`，并开启
`tailwind-scale` feature 或设置 `[options] spacing-scale = "tailwind"`。
`w-[8px]` 等显式长度和原生 GPUI 属性在各模式下保持原单位。

例如，`class="md:flex"` 在宽松模式下警告并忽略，在 strict 模式下编译报错；
`class={String::from("md:flex")}` 则遵循动态策略。`onUnknownClass` 只处理不支持的
**动态** token，不能屏蔽静态错误。`w-[oops]` 这样的非法值在两种静态模式下都会报错。
将字面量移入变量前，先核对[静态与动态能力表](#动态-class)。

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

关于 render 循环分配、ID/key 行为和验证命令，见[性能优化](/gpui-rsx/zh-cn/guides/performance/)。

## 刻度与主题

默认 `gap-4` 是 4px。`rsx_tw!`、`tailwind-scale` feature 或 `rsx.toml` 中的
`[options] spacing-scale = "tailwind"` 可将数值间距和尺寸改为每单位 4px。
任意值、分数和 GPUI 预设不受影响。已有界面的迁移步骤见
[迁移指南](/gpui-rsx/zh-cn/guides/migration/)。

静态字面量支持 `hover:`、`focus:`、`focus-visible:`、`active:`、`group-hover:` 和
`group-active:`，也可写具名组 `group-hover/menu:`。有状态的前缀会注入 ID，循环内要提供
稳定 key 或显式的唯一 ID。
主题令牌通过 `rsx.toml` 的 `[colors]`、`[radius]`、`[spacing]` 定义；还支持
`[text-size]`（`text-size-body`）、`[line-height]`（`leading-body`）、`[opacity]`
（`opacity-muted`）和只接受表达式的 `[shadow]`（`shadow-card`）。gpui-component
预设可将 `bg-primary` 等映射为 `cx.theme()` 调用。状态前缀和主题令牌不支持动态字符串。
可用 `cargo rsx theme check`、`cargo rsx theme list` 和
`cargo rsx theme explain bg-primary` 查看配置及生成的 GPUI 调用。

```toml
[text-size]
body = "14px"
[line-height]
body = "20px"
[opacity]
muted = "60%"
[shadow]
card = { expr = "vec![BoxShadow::new(px(0.0), px(1.0), hsla(0.0, 0.0, 0.0, 0.2))]" }
```

通过 `class="text-size-body leading-body opacity-muted shadow-card"` 使用。阴影表达式
需返回 GPUI `.shadow()` 接受的类型，其中的名称需要在宏调用处可见。静态令牌值
也可用于预览和颜色提示，任意 Rust 表达式继续显示为占位内容。

配置的完整示例、令牌映射与检查流程见[主题与设计令牌](/gpui-rsx/zh-cn/guides/theming/)。

## 常用 Tailwind 值

`bg-red-500/50` 应用 50% 透明度（文字与边框颜色同样支持），`rounded-[4px]` 设置圆角，
`text-[14px]` 设置字号，`leading-6` 设置行高（默认 6px，Tailwind 刻度下为 24px）。
透明度修饰符接受 0 到 100 的百分数，每种颜色最多一个 `/百分数` 修饰符。字面量与动态 class 均支持这些值；字面量还可使用
支持的状态变体。`z-*`、`space-x-*`、`space-y-*`、`divide-*`、过渡与动画需显式使用
GPUI 实现，诊断会说明限制。
