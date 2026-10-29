---
title: 主题与设计令牌
description: 在 rsx.toml 中统一定义颜色、间距、排版、透明度与阴影。
---

将共用的视觉约定放在 crate 的 `Cargo.toml` 同级目录下的 `rsx.toml` 中。宏会在展开静态 class 时解析这些令牌，生成普通的 GPUI builder 调用。

## 定义设计令牌

从一份小配置开始：

```toml title="rsx.toml"
[colors]
brand = "#2563eb"

[radius]
card = "12px"

[spacing]
gutter = "18px"

[text-size]
body = "14px"

[line-height]
body = "20px"

[opacity]
muted = "60%"
```

在静态 class 字面量中使用生成的类名：

```rust
rsx! {
    <div class="flex flex-col gap-gutter p-gutter rounded-card">
        <h1 class="text-brand font-bold">{"项目概览"}</h1>
        <p class="text-size-body leading-body opacity-muted">
            {"让不同视图共用一套视觉约定。"}
        </p>
    </div>
}
```

| 配置段 | 类名示例 | GPUI 方法 |
| --- | --- | --- |
| `colors` | `bg-brand`、`text-brand`、`border-brand` | `.bg()`、`.text_color()`、`.border_color()` |
| `radius` | `rounded-card` | `.rounded()` |
| `spacing` | `p-gutter`、`gap-gutter` | `.p()`、`.gap()` 及方向变体 |
| `text-size` | `text-size-body` | `.text_size()` |
| `line-height` | `leading-body` | `.line_height()` |
| `opacity` | `opacity-muted` | `.opacity()` |
| `shadow` | `shadow-card` | `.shadow()` |

长度值使用显式单位。透明度接受带引号的小数（`"0.6"`）或百分比（`"60%"`）。重复键、无效值与冲突的 `text-size-*` 类名会产生配置错误。

## 跟随应用主题

`gpui-component` 预设提供 `bg-background`、`text-foreground`、`bg-primary` 等语义颜色：

```toml title="rsx.toml"
[options]
preset = "gpui-component"
# context = "cx"  # 默认值；若渲染上下文使用其他名称，可在此修改
```

引入主题 trait，并让 `cx` 在宏展开的作用域内可用：

```rust
use gpui::*;
use gpui::prelude::*;
use gpui::component::ActiveTheme;
use gpui_rsx::rsx;

// 在带有 cx: &mut Context<Self> 的 Render 实现中：
rsx! {
    <div class="bg-background text-foreground p-[24px]">
        {"跟随当前应用主题。"}
    </div>
}
```

预设生成 `cx.theme().colors.background` 等表达式。十六进制字面量令牌保持固定颜色；同名自定义颜色会覆盖预设条目。

组件构造、初始化与应用启动方式见 [gpui-component 集成](/gpui-rsx/zh-cn/guides/gpui-component/)。

## Rust 表达式与阴影

令牌也可以使用在视图渲染时求值的 Rust 表达式：

```toml title="rsx.toml"
[colors]
surface = { expr = "cx.theme().colors.background" }

[shadow]
card = { expr = "vec![BoxShadow::new(px(0.0), px(1.0), hsla(0.0, 0.0, 0.0, 0.2))]" }
```

使用 `class="bg-surface shadow-card"` 应用它们。阴影令牌必须使用表达式，返回值需符合 GPUI `.shadow()` 方法的要求。`BoxShadow`、`px`、`hsla` 等所有引用名称，都要在宏展开的 Rust 作用域中可用。

:::note[静态 class 与预览]
主题令牌和状态变体适用于静态 class 字面量，也包括字面量条件分支；运行时生成的 class 字符串无法解析它们。静态令牌值会出现在预览和编辑器色块中，预览不会执行任意 Rust 表达式。
:::

## 检查配置

[安装 CLI](/gpui-rsx/zh-cn/guides/tooling/) 后，可以在应用目录运行，也可以指定包的 manifest：

```sh
cargo rsx theme check --manifest-path demo/Cargo.toml
cargo rsx theme list --manifest-path demo/Cargo.toml
cargo rsx theme explain hover:opacity-muted --manifest-path demo/Cargo.toml
```

三个命令都支持 `--json`。配置验证失败返回状态码 `1`，用法错误返回 `2`。编辑器语言服务使用相同的配置解析器。

## 选择间距刻度

默认数值间距以像素为单位：`gap-4` 是 4px。若要在当前 crate 启用每单位 4px 的 Tailwind 刻度：

```toml title="rsx.toml"
[options]
spacing-scale = "tailwind"
```

可以将此设置与 `preset` 写在同一个 `[options]` 段中。`gap-[14px]` 等显式长度与 `gap-gutter` 等命名令牌保持声明的值。修改已有布局前，请阅读[间距迁移说明](/gpui-rsx/zh-cn/guides/migration/)。

## 共享动态 class helper

`class={expr}` 调用点较多时，在 crate 根添加 `gpui_rsx::rsx_runtime!();`，
并在 `[options]` 设置 `dynamic-runtime = "rsx_runtime"`。该选项独立于
`spacing-scale`，不会让动态字符串支持主题 token。
目标放置方式、GPUI 别名和测量结果见[性能指南](/gpui-rsx/zh-cn/guides/performance/)。
