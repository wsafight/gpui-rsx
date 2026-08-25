# GPUI-RSX

[English](./README.md) | 简体中文

[![CI](https://github.com/wsafight/gpui-rsx/actions/workflows/ci.yml/badge.svg)](https://github.com/wsafight/gpui-rsx/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/wsafight/gpui-rsx/branch/main/graph/badge.svg)](https://codecov.io/gh/wsafight/gpui-rsx)
[![Crates.io](https://img.shields.io/crates/v/gpui-rsx.svg)](https://crates.io/crates/gpui-rsx)
[![Documentation](https://docs.rs/gpui-rsx/badge.svg)](https://docs.rs/gpui-rsx)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

一个为 GPUI 提供 JSX-like 语法的 Rust 过程宏，让 UI 开发更加简洁和直观。

## ✨ 特性

- 🎨 **HTML-like 语法** - 类似 React JSX 的开发体验
- 🚀 **静态标记无运行时解析** - 静态 RSX 编译为原生 GPUI builder 调用
- 📦 **轻量级** - 仅依赖 `syn`, `quote`, `proc-macro2`
- 🔧 **灵活** - 支持表达式、条件渲染、组件组合
- 💡 **类型安全** - 完全的编译时检查
- 🧩 **Fragment 支持** - 使用 `<>...</>` 返回多个根元素
- 🔁 **For 循环语法糖** - 使用 `{for item in iter { ... }}` 迭代
- 🔑 **循环安全 ID** - `key={expr}` 为每次迭代生成唯一 ID；缺少 key 时报编译错误
- 🎨 **完整 Tailwind 色板** - 22 个色系 × 11 个色阶 + black/white + 任意 hex/RGB/RGBA 值
- 📐 **桌面布局工具类** - 支持任意长度、百分比和分数尺寸，适合面板和分栏布局
- ⚡ **动态 Class** - 支持运行时 class 切换：颜色、尺寸、间距和数值前缀回退
- 🔍 **诊断与预览** - strict/permissive 宏、可读错误和 `rsx_expand!`

## 📚 文档资源

- **[文档站](https://wsafight.github.io/gpui-rsx/zh-cn/)** - Astro/Starlight 中英文文档
- **[架构指南](./ARCHITECTURE_CN.md)** - 详细的架构文档
  - 模块组织和数据流
  - 代码生成策略（match 查找表、单次扫描、Vec 复用等）
  - 设计模式和测试方法
  - 扩展点和调试指南
- **[快速开始](https://wsafight.github.io/gpui-rsx/zh-cn/getting-started/)** - 分步教程
- **[语法参考](https://wsafight.github.io/gpui-rsx/zh-cn/usage/syntax/)** - 元素、属性、子节点和条件渲染
- **[API 参考](https://wsafight.github.io/gpui-rsx/zh-cn/reference/api/)** - 宏和映射参考
- **[最佳实践](https://wsafight.github.io/gpui-rsx/zh-cn/guides/best-practices/)** - 推荐模式
- **[迁移指南](https://wsafight.github.io/gpui-rsx/zh-cn/guides/migration/)** - 升级说明
- **[问题排查](https://wsafight.github.io/gpui-rsx/zh-cn/guides/troubleshooting/)** - 常见问题和解决方案

本地运行文档站：

```bash
cd docs
bun install
bun run dev
```

## 📦 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit", "runtime_shaders", "wayland", "x11"] }
gpui-rsx = "0.7"
```

### GPUI 版本目标

`gpui-rsx` 0.6 及后续版本使用 Zed 仓库中的 GPUI，而不是 crates.io 发布的 `gpui = "0.2.2"` 包。这个 git 依赖仍然显示为 `gpui v0.2.2`，但 API 面与 crates.io 的版本不同，并包含这些版本使用的 helper 方法。

如果你从旧版 `gpui-rsx` 升级，请将：

```toml
gpui = "0.2.2"
```

替换为上方的 Zed git `gpui` 和 `gpui_platform` 依赖，并把应用启动方式更新为 `gpui_platform::application()`。

应用项目应提交 `Cargo.lock` 来固定实际解析到的 Zed revision。如果同时使用 `gpui-component`，请保持直接依赖的 `gpui` / `gpui_platform` source 与 `gpui-component` 使用的 GPUI source 完全一致；混用 bare git 和 `rev = "..."` 会生成两份 GPUI crate，导致组件类型不兼容。

## 🚀 快速开始

### 5 分钟上手

```rust
use gpui::*;
use gpui::prelude::*;
use gpui_platform::application;
use gpui_rsx::rsx;

struct CounterView {
    count: i32,
}

impl Render for CounterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="flex flex-col gap-4 p-4">
                <h1>{format!("Count: {}", self.count)}</h1>
                <div class="flex gap-2">
                    <button
                        bg={rgb(0x3b82f6)}
                        text_color={rgb(0xffffff)}
                        px_4
                        py_2
                        rounded_md
                        onClick={cx.listener(|view, _, _window, cx| {
                            view.count += 1;
                            cx.notify();
                        })}
                    >
                        {"Increment"}
                    </button>
                    <button
                        bg={rgb(0xef4444)}
                        text_color={rgb(0xffffff)}
                        px_4
                        py_2
                        rounded_md
                        onClick={cx.listener(|view, _, _window, cx| {
                            view.count -= 1;
                            cx.notify();
                        })}
                    >
                        {"Decrement"}
                    </button>
                </div>
            </div>
        }
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_cx| CounterView { count: 0 })
        }).unwrap();
        cx.activate(true);
    });
}
```

### 前后对比

#### ❌ 传统 GPUI 写法（繁琐）

```rust
fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        .child(
            div()
                .text_xl()
                .font_weight(FontWeight::BOLD)
                .child(format!("Count: {}", self.count))
        )
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    div()
                        .bg(rgb(0x3b82f6))
                        .text_color(rgb(0xffffff))
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .on_click(cx.listener(|view, _, _window, cx| {
                            view.count += 1;
                            cx.notify();
                        }))
                        .child("Increment")
                )
                .child(
                    div()
                        .bg(rgb(0xef4444))
                        .text_color(rgb(0xffffff))
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .on_click(cx.listener(|view, _, _window, cx| {
                            view.count -= 1;
                            cx.notify();
                        }))
                        .child("Decrement")
                )
        )
}
```

#### ✅ 使用 GPUI-RSX（简洁）

见上方快速开始示例。

**代码减少：~50%** ✨

## 📖 语法指南

### 1. 基本元素

```rust
rsx! {
    <div>{"Hello GPUI"}</div>
}
```

展开为：
```rust
div().child("Hello GPUI")
```

### 2. Fragment（多根节点）

当你需要返回多个元素而不需要外层包装时：

```rust
rsx! {
    <>
        <div>{"第一个"}</div>
        <div>{"第二个"}</div>
        <div>{"第三个"}</div>
    </>
}
```

展开为：
```rust
vec![
    div().child("第一个"),
    div().child("第二个"),
    div().child("第三个"),
]
```

### 3. 属性

#### 布尔属性（Flag）

```rust
rsx! {
    <div flex flex_col />
}
```

展开为：
```rust
div().flex().flex_col()
```

#### 值属性

```rust
rsx! {
    <div gap={px(16.0)} bg={rgb(0xffffff)} />
}
```

展开为：
```rust
div().gap(px(16.0)).bg(rgb(0xffffff))
```

### 4. Class 属性

`class` 属性接受类似 Tailwind 的字符串，展开为多个 GPUI 方法调用：

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4" />
}
```

展开为：
```rust
div().flex().flex_col().gap(px(4.0)).p(px(4.0))
```

> **注意：** `class` 同时支持静态字符串（编译时处理）和动态表达式（运行时解析）。GPUI-RSX 实现的是 Tailwind-like 子集，不是完整 Tailwind CSS 引擎。静态 class 会直接展开为 GPUI builder 调用；动态 class 表达式使用一个很小的运行时 matcher。详见 [FAQ Q5](#q5-可以使用动态-class-值吗)。

#### 支持的 class 模式

**布局：**
- `flex`, `flex-col`, `flex-row`, `flex-wrap`, `flex-1`, `flex-none`, `flex-auto`
- `flex-grow`, `flex-grow-0`, `flex-grow-1`, `flex-shrink`, `flex-shrink-0`, `flex-shrink-1`
- `min-w-0`, `min-h-0`, `items-center`, `items-start`, `items-end`, `items-stretch`
- `justify-center`, `justify-between`, `justify-around`, `justify-evenly`

**间距**（数值自动转为 `px(n)`）：
- `gap-4` → `.gap(px(4.0))`
- `p-4`, `px-4`, `py-4`, `pt-4`, `pb-4`, `pl-4`, `pr-4`
- `m-4`, `mx-4`, `my-4`, `mt-4`, `mb-4`, `ml-4`, `mr-4`
- 任意间距：`gap-[14px]`, `gap-x-[0.75rem]`, `p-[18px]`, `mx-[1.25rem]`
- `gap-[10%]` 这类百分比间距会报错，因为 GPUI 间距使用 definite length

**尺寸：**
- 数值尺寸保持项目原有语义：`w-64` → `.w(px(64.0))`, `h-32` → `.h(px(32.0))`
- `w-full`, `h-full`, `size-full`, `aspect-square`
- `w-px`, `h-px`, `w-auto`, `h-auto`, `w-1/2`, `h-1/3`, `size-1/2`
- 任意尺寸：`w-[280px]`, `w-[18rem]`, `w-[37.5%]`, `min-w-[280px]`, `max-w-[32rem]`
- 任意分母的分数尺寸：`w-6/24`, `min-w-1/3`, `size-3/4`

**文本：**
- `text-xs`, `text-sm`, `text-base`, `text-lg`, `text-xl`
- `text-2xl`, `text-3xl`
- `font-thin`, `font-extralight`, `font-light`, `font-normal`, `font-medium`, `font-semibold`, `font-bold`, `font-extrabold`, `font-black`
- `whitespace-normal`, `whitespace-nowrap`, `line-clamp-*`
- `text-ellipsis`, `truncate`, `no-underline`
- `text-decoration-solid`, `text-decoration-wavy`, `text-decoration-0/1/2/4/8`

**对齐：**
- `content-normal`, `content-center`, `content-start`, `content-end`, `content-between`, `content-around`, `content-evenly`, `content-stretch`
- `self-start`, `self-end`, `self-flex-start`, `self-flex-end`, `self-center`, `self-baseline`, `self-stretch`

**边框：**
- `border` → `.border_1()`
- `border-2` → `.border_2()`, `border-4` → `.border_4()`
- `rounded-sm`, `rounded-md`, `rounded-lg`, `rounded-xl`, `rounded-2xl`, `rounded-3xl`, `rounded-full`, `rounded-none`
- 方向圆角类，例如 `rounded-t-lg`, `rounded-b-lg`, `rounded-r-lg`, `rounded-l-lg`

**颜色**（完整 Tailwind 色板）：
- `text-red-500` → `.text_color(rgb(0xef4444))`
- `bg-blue-600` → `.bg(rgb(0x2563eb))`
- `border-green-500` → `.border_color(rgb(0x22c55e))`
- 任意颜色：`bg-[#ff0000]`, `text-[#333]`, `border-[#11223344]`, `bg-[rgb(15,23,42)]`, `text-[rgba(15,23,42,0.8)]`

**效果：**
- `shadow-none`, `shadow-2xs`, `shadow-xs`, `shadow-sm`, `shadow-md`, `shadow-lg`, `shadow-xl`, `shadow-2xl`
- `overflow-hidden`, `overflow-x-hidden`, `overflow-y-hidden`, `overflow-scroll`
- `cursor-pointer`, `cursor-default`, `cursor-text`, `cursor-move`, `cursor-grab`, `cursor-not-allowed`，以及 resize cursor 变体
- `debug-outline` 在 debug 构建中启用 GPUI 调试边框，在 release 构建中为空操作

**Grid 定位：**
- `col-span-*`, `col-start-*`, `col-end-*`, `row-span-*`, `row-start-*`, `row-end-*`
- `col-span-full`, `col-start-auto`, `col-end-auto`, `row-span-full`, `row-start-auto`, `row-end-auto`

**支持的颜色：** slate, gray, zinc, neutral, stone, red, orange, amber, yellow, lime, green, emerald, teal, cyan, sky, blue, indigo, violet, purple, fuchsia, pink, rose（色阶 50-950）+ white, black

### 5. 事件处理

```rust
rsx! {
    <button onClick={cx.listener(|view, _, _window, cx| {
        println!("clicked");
    })}>
        {"Click me"}
    </button>
}
```

支持的事件（camelCase / snake_case）：

| 事件 | 方法 |
|------|------|
| `onClick` / `on_click` | `.on_click(handler)` |
| `onMouseDown` / `on_mouse_down` | `.on_mouse_down(button, handler)` |
| `onMouseUp` / `on_mouse_up` | `.on_mouse_up(button, handler)` |
| `onMouseMove` / `on_mouse_move` | `.on_mouse_move(handler)` |
| `onMouseDownOut` / `on_mouse_down_out` | `.on_mouse_down_out(handler)` |
| `onMouseUpOut` / `on_mouse_up_out` | `.on_mouse_up_out(button, handler)` |
| `onAnyMouseDown` / `on_any_mouse_down` | `.on_any_mouse_down(handler)` |
| `onAnyMouseUp` / `on_any_mouse_up` | `.on_any_mouse_up(handler)` |
| `onKeyDown` / `on_key_down` | `.on_key_down(handler)` |
| `onKeyUp` / `on_key_up` | `.on_key_up(handler)` |
| `onModifiersChanged` / `on_modifiers_changed` | `.on_modifiers_changed(handler)` |
| `onHover` / `on_hover` | `.on_hover(handler)` |
| `onScrollWheel` / `on_scroll_wheel` | `.on_scroll_wheel(handler)` |
| `onDrag` / `on_drag` | `.on_drag(value, constructor)` |
| `onDragMove` / `on_drag_move` | `.on_drag_move(handler)` |
| `onDrop` / `on_drop` | `.on_drop(handler)` |
| `onAction` / `on_action` | `.on_action(handler)` |
| `onBoxedAction` / `on_boxed_action` | `.on_boxed_action(action, handler)` |
| `captureAnyMouseDown` / `capture_any_mouse_down` | `.capture_any_mouse_down(handler)` |
| `captureAnyMouseUp` / `capture_any_mouse_up` | `.capture_any_mouse_up(handler)` |
| `captureKeyDown` / `capture_key_down` | `.capture_key_down(handler)` |
| `captureKeyUp` / `capture_key_up` | `.capture_key_up(handler)` |
| `captureAction` / `capture_action` | `.capture_action(handler)` |

多参数 GPUI 方法在 RSX 中使用 tuple 语法：
`onMouseDown={(MouseButton::Left, handler)}`、`onDrag={(value, constructor)}`。

### 6. 嵌套元素

```rust
rsx! {
    <div>
        <h1>{"标题"}</h1>
        <p>{"描述"}</p>
        <div>
            <button>{"操作 1"}</button>
            <button>{"操作 2"}</button>
        </div>
    </div>
}
```

### 7. 表达式

```rust
rsx! {
    <div>
        {format!("Count: {}", self.count)}
        {self.render_child_component()}
        {if self.show {
            rsx! { <span>{"可见"}</span> }
        } else {
            rsx! { <span>{"隐藏"}</span> }
        }}
    </div>
}
```

### 8. 列表渲染

#### 使用迭代器（传统方式）

```rust
rsx! {
    <div>
        {self.items.iter().map(|item| {
            rsx! {
                <div key={item.id}>
                    {item.name.clone()}
                </div>
            }
        }).collect::<Vec<_>>()}
    </div>
}
```

#### 使用 for 循环语法糖

```rust
rsx! {
    <ul>
        {for item in &self.items {
            <li>{item.name.clone()}</li>
        }}
    </ul>
}
```

展开为：
```rust
div().children((&self.items).into_iter().map(|item| {
    div().child(item.name.clone())
}))
```

#### 循环安全 — `key` 属性

for 循环内有 stateful 属性（`onClick`、`onHover`、`onDrag`、`onAuxClick`、
`onA11yAction`、`active`、`activeClass`、`groupActive`、`tooltip`、`tooltipShowDelay`、
`focusable`、`role`、`ariaLabel`、`ariaDescription`、`accessibilityId`、`overflowScroll`、
`restrictScrollToAxis`、`trackScroll`、`externalDragPayload` 或 `overflow-scroll`）的元素**必须**提供
`id` 或 `key`，否则宏会报编译错误：

```rust
// ❌ 编译错误 — 所有 <li> 会共享相同的自动 ID
{for item in &self.items { <li onClick={handler}>{item}</li> }}

// ✅ key 使每次迭代获得唯一 ID
{for item in &self.items {
    <li key={item.id} onClick={handler}>{item.name.clone()}</li>
}}
// → div().id(format!("src/list.rs::__rsx_li_L42C8_{}", item.id)).on_click(handler)…
```

`key` 由宏在编译期消费，**不会**生成 `.key()` 方法调用。
可接受任何实现 `Display` 的类型。非 stateful 元素上的 `key` 会被静默忽略
（不会注入 `.id()`，不改变元素类型）。

for 循环也支持 range 和方法调用：

```rust
rsx! {
    <div>
        {for i in 0..5 {
            <span>{i}</span>
        }}
    </div>
}
```

### 9. 动态 Class

GPUI-RSX 同时支持静态和动态 `class` 属性。

#### 静态 Class（编译期——推荐）

```rust
// ✅ 最佳性能 - 编译时解析，支持文档列出的子集
rsx! {
    <div class="flex gap-4 bg-blue-500">
        {"静态样式"}
    </div>
}
```

#### 动态 Class（运行时）

```rust
let classes = if is_active { "flex gap-4" } else { "block" };
rsx! {
    <div class={classes}>
        {"动态样式"}
    </div>
}
```

> **运行时支持范围：** 常用布局/间距/文字排版工具类、完整 Tailwind 色板、
> 任意颜色（如 `bg-[#ff0000]`、`text-[#f00]`、`bg-[rgba(15,23,42,0.8)]`）、
> 任意间距和尺寸长度（如 `w-[280px]`、`h-[50%]`、`gap-[14px]`、`mx-[1.25rem]`）、
> 分数尺寸（如 `w-6/24`），以及数值前缀回退（如 `gap-7`、`p-5`、`opacity-33`）。
> 真正不受支持的 class（如 Tailwind variants 或未知工具类）在 release 构建中**静默忽略**，
> 在 debug 构建中打印警告。
>
> **推荐替代方案**（按优先级排序）：
> 1. **字符串字面量**（最佳）：`class="flex gap-4"` — 编译期，支持文档列出的子集
> 2. **条件 / match 字面量**：`class={if active { "flex gap-4" } else { "block" }}`
>    或 `class={match state { State::Active => "flex", _ => "block" }}` — 仍会编译期展开
> 3. **独立属性**：`<div flex gap_4 />` — 编译期，类型检查
> 4. **`when` 属性**：`when={(cond, |el| el.flex())}` — 编译期，完全灵活
> 5. **动态表达式**：`class={expr}` — 运行时解析器，覆盖范围比静态字面量窄

**常见模式：**

```rust
// ✅ 条件字面量（编译期，支持文档列出的子集）
let btn_class = if primary { "bg-blue-500 text-white" } else { "bg-gray-200 text-black" };

// ✅ when 属性（编译期，完全灵活）
rsx! { <div when={(primary, |el| el.bg(rgb(0x3b82f6)).text_color(rgb(0xffffff)))} /> }

// ✅ 含数值前缀和任意颜色的动态字符串
let classes = format!("flex gap-{} bg-[#ff0000]", spacing);  // gap-7、gap-32 等均生效
```

### 10. 宏模式与展开预览

`rsx!` 默认是 permissive 模式：无法安全解析的不支持静态 class 会被忽略，而非法 arbitrary value 会报编译错误。使用 `rsx_strict!` 可以拒绝不支持的静态 class：

```rust
use gpui_rsx::{rsx_expand, rsx_permissive, rsx_strict};

rsx_strict! { <div class="flex w-[280px]" /> }
rsx_permissive! { <div class="hover:bg-blue-500 flex" /> }

let preview = rsx_expand! {
    <div class="flex w-[280px] bg-[rgba(15,23,42,0.8)]" />
};
assert!(preview.contains("rgba"));
```

strict 模式下，动态 class 在运行时遇到不支持的 token 会 panic。`rsx_expand!` 返回字符串预览用于调试，不会对生成的 GPUI 表达式做类型检查。

动态 class 能力边界：

| 能力 | 静态 `class="..."` | 动态 `class={expr}` |
|------|--------------------|---------------------|
| 布局、间距、尺寸 | 支持 | 支持子集 |
| 颜色和透明度 | 支持 | 支持 |
| 任意长度/颜色 | 支持 | 支持 |
| 分数尺寸 | 支持 | 支持 |
| stateful scroll class | 支持并自动注入 ID | 不支持 |
| 未知 Tailwind variant | permissive 忽略，strict 报错 | permissive 忽略，strict panic |

### 11. 桌面三栏布局

```rust
rsx! {
    <div class="flex h-full w-full bg-zinc-100">
        <nav class="w-[72px] min-w-[72px] bg-zinc-950" />

        <aside class="w-[280px] min-w-[280px] border-r border-zinc-200 bg-white">
            {"项目与任务"}
        </aside>

        <main class="flex-1 min-w-0 p-[18px]">
            {"对话、计划、diff 和完成页"}
        </main>

        <aside class="w-6/24 min-w-[320px] max-w-[460px] border-l border-zinc-200 bg-white">
            {"执行轨迹"}
        </aside>
    </div>
}
```

`min-w-0` 对桌面分栏布局很重要，它允许中间区域收缩，而不是把固定侧栏挤出窗口。

### 12. 展开语法

```rust
rsx! {
    <div>
        {...items.iter().map(|item| rsx! { <span>{item}</span> })}
    </div>
}
```

### 13. 属性映射参考

大多数 camelCase 属性会映射为 GPUI 的 snake_case 方法；特殊标志行为见下表：

| RSX 属性 | 生成的 GPUI 代码 |
|----------|------------------|
| `opacity` | `.opacity()` |
| `visible` / `invisible` | `.visible()` / `.invisible()` |
| `width` / `height` | `.w()` / `.h()` |
| `minWidth` / `maxWidth` | `.min_w()` / `.max_w()` |
| `minHeight` / `maxHeight` | `.min_h()` / `.max_h()` |
| `gapX` / `gapY` | `.gap_x()` / `.gap_y()` |
| `flexBasis` | `.flex_basis()` |
| `flexGrow` / `flexShrink`（标志） | `.flex_grow_1()` / `.flex_shrink_1()` |
| `fontSize` | `.text_size()` |
| `lineHeight` | `.line_height()` |
| `fontWeight` | `.font_weight()` |
| `fontFamily` | `.font_family()` |
| `textAlign` | `.text_align()` |
| `textColor` | `.text_color()` |
| `backgroundColor` | `.bg()` |
| `borderColor` | `.border_color()` |
| `borderTop` / `borderBottom` | `.border_t(value)` / `.border_b(value)` |
| `borderLeft` / `borderRight` | `.border_l(value)` / `.border_r(value)` |
| `border_t` / `border_b` / `border_l` / `border_r`（标志） | `.border_t_1()` / `.border_b_1()` / `.border_l_1()` / `.border_r_1()` |
| `roundedTop` / `roundedBottom` | `.rounded_t()` / `.rounded_b()` |
| `roundedTopLeft` / `roundedTopRight` | `.rounded_tl()` / `.rounded_tr()` |
| `roundedBottomLeft` / `roundedBottomRight` | `.rounded_bl()` / `.rounded_br()` |
| `boxShadow` | `.shadow()` |
| `inset` | `.inset()` |

不在此表中的属性将原样透传（如 `bg={color}` → `.bg(color)`）。

> **关于 z 轴：** 最新 GPUI 没有 `z-index` 字段，也没有 `.z_index()` builder。
> 同一父节点下的叠放顺序由 child 的绘制顺序决定，后面的 sibling 会盖在前面的 sibling 上；
> 浮层通常用 `absolute`/`relative` 结构或 GPUI 的 overlay / popover / modal 等机制实现。
> 因此 `z-*` class 和 `zIndex` 属性不会被映射。

### 14. 使用 `when` 和 `whenSome` 进行条件样式

#### when - 根据条件应用样式

```rust
rsx! {
    <div
        flex
        when={(is_active, |this| {
            this.bg(rgb(0x3b82f6))
                .text_color(rgb(0xffffff))
        })}
    >
        {"按钮"}
    </div>
}
```

#### whenSome - 当 Option 有值时应用样式

```rust
let custom_width: Option<f32> = Some(200.0);

rsx! {
    <div
        flex
        whenSome={(custom_width, |this, w| this.w(px(w)))}
    >
        {"内容"}
    </div>
}
```

#### whenClass - 根据条件应用静态 class

```rust
rsx! {
    <div
        class="flex px-2"
        whenClass={(active, "bg-neutral-900 text-white")}
        whenClass={(!active, "text-neutral-600")}
    />
}
```

`whenClass` 只接受字符串字面量。`overflow-scroll` 这类需要 stateful ID 的 class 会被拒绝；需要 ID 敏感的 GPUI 方法时，请使用 `when={(cond, |el| el.overflow_scroll())}`。

#### 状态 class 属性 - hover/focus/active

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

这些属性会编译为 GPUI `StyleRefinement` 闭包。它们只接受字符串字面量；`overflow-scroll` 或 `debug-outline` 这类需要元素级 stateful 方法的 class 会被拒绝。`activeClass` 会触发自动 ID 注入；`hoverClass` 和 `focusClass` 不会。

`groupHover` 走 GPUI 的非 stateful 样式闭包路径。`groupActive` 需要 stateful ID。
`groupDragOver` 不作为属性暴露，因为 GPUI 需要显式拖拽数据类型；请使用 `when` 或 `base`
并直接调用 `group_drag_over::<YourType>(...)`。

#### 多个条件

```rust
rsx! {
    <button
        class="px-4 py-2 rounded-md"
        when={(is_selected, |this| this.bg(rgb(0x3b82f6)))}
        when={(is_disabled, |this| this.bg(rgb(0xe5e7eb)))}
        whenSome={(custom_color, |this, color| this.bg(rgb(color)))}
    >
        {"按钮"}
    </button>
}
```

### 15. styled 标志（默认标签样式）

`styled` 标志会根据标签名注入合理的默认样式：

```rust
rsx! {
    <h1 styled>{"标题"}</h1>
    // 展开为: div().text_3xl().font_weight(FontWeight::BOLD).child("标题")

    <button styled>{"点击"}</button>
    // 展开为: div().cursor_pointer().child("点击")
}
```

各标签默认样式：

| 标签 | 默认样式 |
|------|---------|
| `h1` | `text-3xl font-bold` |
| `h2` | `text-2xl font-bold` |
| `h3` | `text-xl font-bold` |
| `h4` | `text-lg font-bold` |
| `h5` | `text-base font-bold` |
| `h6` | `text-sm font-bold` |
| `button`, `a` | `cursor-pointer` |
| `input`, `textarea` | `px-2 py-1` |
| `ul`, `ol` | `flex flex-col` |
| `li` | `flex items-center` |
| `p` | `text-base` |
| `label` | `text-sm` |
| `form` | `flex flex-col gap-4` |

用户属性在默认样式之后应用，可以覆盖。

## 🎯 完整示例

### Todo 应用

```rust
use gpui::*;
use gpui::prelude::*;
use gpui_rsx::rsx;

struct TodoApp {
    todos: Vec<Todo>,
    input: String,
}

struct Todo {
    id: usize,
    text: String,
    completed: bool,
}

impl Render for TodoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="flex flex-col gap-4 p-4">
                <h1 class="text-2xl font-bold">
                    {"Todo List"}
                </h1>

                <div class="flex gap-2">
                    <input
                        placeholder="Add a todo..."
                        value={self.input.clone()}
                    />
                    <button
                        class="bg-blue-500 text-white px-4 py-2 rounded-md"
                        onClick={cx.listener(|view, _, _window, cx| {
                            view.add_todo();
                            cx.notify();
                        })}
                    >
                        {"Add"}
                    </button>
                </div>

                <div class="flex flex-col gap-2">
                    {for todo in self.todos.iter() {
                        <div
                            class="flex gap-2 items-center p-2 rounded-md"
                            bg={if todo.completed {
                                rgb(0xf3f4f6)
                            } else {
                                rgb(0xffffff)
                            }}
                        >
                            <span>{todo.text.clone()}</span>
                        </div>
                    }}
                </div>
            </div>
        }
    }
}

impl TodoApp {
    fn add_todo(&mut self) {
        if !self.input.is_empty() {
            self.todos.push(Todo {
                id: self.todos.len(),
                text: self.input.clone(),
                completed: false,
            });
            self.input.clear();
        }
    }
}
```

## 🔧 高级用法

### 自定义组件

```rust
fn render_card(&self, title: &str, content: &str) -> impl IntoElement {
    rsx! {
        <div class="rounded-lg shadow-md p-6">
            <h2 class="text-xl font-bold">
                {title}
            </h2>
            <p class="text-gray-600">
                {content}
            </p>
        </div>
    }
}

fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    rsx! {
        <div>
            {self.render_card("Title 1", "Content 1")}
            {self.render_card("Title 2", "Content 2")}
        </div>
    }
}
```

### Builder 构造组件

当组件需要自定义构造函数，而不是默认的 `<Tag />` 展开为 `Tag()` 时，使用 `base={expr}`：

```rust
rsx! {
    <Button
        base={Button::new("continue")}
        label={"继续"}
        small
        primary
    />
}
```

这会展开为 `Button::new("continue").label("继续").small().primary()`。`base` 属性由宏消费，不会生成 `.base(...)`。

支持路径型组件 tag，适合组件放在模块里的场景：

```rust
rsx! {
    <ui::TaskCard
        base={ui::TaskCard::new(task.id)}
        title={task.title.clone()}
    />
}
```

混用 `gpui-component` 时，保持 import 显式，并用 `base` 表达需要 ID 或自定义参数的构造函数：

```rust
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::Sizable as _;

rsx! {
    <Button
        base={Button::new("continue")}
        label={"继续"}
        small
        primary
    />
}
```

### 条件渲染

```rust
rsx! {
    <div>
        {if self.loading {
            rsx! { <div>{"Loading..."}</div> }
        } else if let Some(error) = &self.error {
            rsx! { <div class="text-red-500">{error.clone()}</div> }
        } else {
            rsx! { <div>{self.render_content()}</div> }
        }}
    </div>
}
```

### 动态样式

```rust
fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let bg_color = if self.is_active {
        rgb(0x3b82f6)
    } else {
        rgb(0x6b7280)
    };

    rsx! {
        <div bg={bg_color} class="px-4 py-2 rounded-md">
            {"Button"}
        </div>
    }
}
```

## 📊 性能

GPUI-RSX 是一个**编译时宏**，静态 RSX 会展开为直接的 GPUI builder 调用。静态标记没有运行时解析器；动态 `class={expr}` 会有意使用一个很小的运行时 matcher。

| 指标 | 传统 GPUI | GPUI-RSX |
|------|----------|----------|
| 代码量 | 100 行 | 50 行 (-50%) |
| 运行时性能 | 基准 | 相同 |
| 类型安全 | ✅ | ✅ |
| 编译时检查 | ✅ | ✅ |

### v0.3.2 修复与改进
- **修复** `parse_single_class` 在 Tailwind 变体语法（如 `hover:bg-blue-500`）上的 panic：
  含非法字符的 class 名现在静默跳过，而非触发 `syn::Ident::new` panic
- **新增** 7 个动态 class：`rounded-none`、`rounded-xl`、`cursor-default`、`cursor-text`、
  `shadow-sm`、`shadow-md`、`shadow-lg`
- **文档** styled 默认表新增 `li`、`p`、`label`、`form`；同步 GPUI 0.2.2 方法名；
  移除不存在的 `text-4xl`/`text-5xl`；更新动态 class 说明

### v0.3.1 修复与新增
- **修复** `is_stateful_attr`：`hover`/`active`/`focus`/`group` 是 `Styled` trait 方法，
  不再触发不必要的 `.id()` 注入
- **新增** `key={expr}` 属性：为 for 循环内 stateful 元素生成复合自动 ID
- **新增** for 循环内 stateful 元素缺少 `id` 或 `key` 时报编译错误
- 非 stateful 元素上的 `key` 静默忽略（不改变 `Div` → `Stateful<Div>` 类型）

### v0.3.0 重构亮点
- 消除 `tests/common/mod.rs` 中约 60 个重复方法定义（823 → 456 行）
- 简化 `runtime.rs` black/white 条目生成（方法名直接编码进数据）
- 在 `class.rs` 中提取 `is_directional_border()` 辅助函数，逻辑更清晰

### v0.2.2 优化亮点

**编译时性能：**
- class 解析改用 `split_ascii_whitespace`
- 统一 `text_` 前缀处理（单次 `strip_prefix` 调用）
- 空元素提前快速路径
- `Vec::with_capacity(attrs * 2 + children)` 减少重分配

**运行时性能：**
- `.children([...])` 聚合阈值 3 → 2

**二进制体积：**
- 应用可在自己的 release profile 中选择启用 `panic = "abort"` 以移除展开表

### v0.2.1 优化亮点

**编译时性能：**
- O(1) 颜色/属性/间距查找（`match` 跳转表，无线性扫描）
- `generate_element` 单次属性扫描
- 动态 class match 分支 thread_local 缓存（每进程只生成一次）

**内存分配减少：**
- `parse_class_string` 返回迭代器（无中间 `Vec`）
- `generate_attr_methods` 直接推送到调用方缓冲区
- `Cow<str>` 实现 class 名称转换（无连字符时零拷贝）
- 全面使用 `Vec::with_capacity` 预分配

**运行时性能：**
- 动态 class 字符串通过 `AsRef<str>` 零拷贝传递（`&str` 无需分配）

**二进制体积：**
- 动态 class match 表通过 `#[inline(never)]` + LLVM ICF 去重
- 同一组件内多个 `class={expr}` 共享同一函数体

## 🛠️ 开发

### 构建

```bash
cd gpui-rsx
cargo build
```

### 测试

```bash
cargo test --test macro_tests
```

### 展开宏（调试）

```bash
# 安装 cargo-expand
cargo install cargo-expand

# 查看展开后的代码
cargo expand --lib
```

## 💡 最佳实践

### 1. 组件拆分

将复杂的 UI 拆分为小的、可复用的组件：

```rust
// ✅ 推荐：拆分为多个方法
fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    rsx! {
        <div>
            {self.render_header()}
            {self.render_content()}
            {self.render_footer()}
        </div>
    }
}

fn render_header(&self) -> impl IntoElement {
    rsx! { <header>{"Header"}</header> }
}
```

### 2. 使用常量

将重复的样式提取为常量：

```rust
const PRIMARY_BG: Rgb = rgb(0x3b82f6);
const PRIMARY_TEXT: Rgb = rgb(0xffffff);

rsx! {
    <button bg={PRIMARY_BG} text_color={PRIMARY_TEXT}>
        {"Button"}
    </button>
}
```

### 3. 避免过度嵌套

```rust
// ❌ 不推荐：过度嵌套
rsx! {
    <div>
        <div>
            <div>
                <div>
                    {"Content"}
                </div>
            </div>
        </div>
    </div>
}

// ✅ 推荐：扁平化结构
rsx! {
    <div class="flex flex-col gap-4">
        {"Content"}
    </div>
}
```

## 🐛 常见问题

### Q1: 如何在 RSX 中使用变量？

```rust
let title = "Hello";
rsx! {
    <div>{title}</div>
}
```

### Q2: 如何处理 Option 类型？

```rust
rsx! {
    <div>
        {if let Some(text) = &self.optional_text {
            rsx! { <span>{text.clone()}</span> }
        } else {
            rsx! { <span>{"No text"}</span> }
        }}
    </div>
}
```

### Q3: 宏展开后的代码是什么样的？

使用 `rsx_expand!` 查看局部字符串预览，或用 `cargo expand` 查看完整 crate 展开：

```rust
let preview = gpui_rsx::rsx_expand! {
    <div class="flex w-[280px] bg-[rgba(15,23,42,0.8)]" />
};
```

```bash
cargo expand --lib
```

### Q4: 支持哪些元素？

所有 GPUI 支持的元素都可以使用，如 `div`, `button`, `input`, `span` 等。

### Q5: 可以使用动态 class 值吗？

可以，但有重要限制：

```rust
// ✅ 静态字面量（编译期，支持文档列出的子集——推荐）
rsx! { <div class="flex gap-4" /> }

// ✅ 独立属性（编译期，类型检查）
rsx! { <div bg={dynamic_color} flex /> }

// ✅ when 条件样式（编译期，完全灵活）
rsx! { <div when={(is_active, |this| this.bg(rgb(0x3b82f6)))} /> }

// ✅ 含数值、任意长度和任意颜色的动态表达式
let classes = if active { "flex gap-4" } else { "block" };
rsx! { <div class={classes} /> }
// Tailwind variants 或未知工具类在 permissive 模式中会被忽略。
```

**建议：** 需要动态样式时，优先使用 `when`/`whenSome` 或独立值属性（如 `bg={color}`）——它们是编译期处理，支持所有 GPUI 提供的功能。

### Q6: 如何在 Fragment 中混合不同元素类型？

`rsx!` 的 Fragment 返回 `Vec<impl IntoElement>`，因此多个根节点需要是同一个具体类型。多数场景更推荐用父元素包起来：

```rust
rsx! {
    <div>
        <div />
        {Button::new("save")}
    </div>
}
```

如果确实需要 Fragment，可以显式做类型擦除：

```rust
rsx! {
    <>
        {div().into_any_element()}
        {Button::new("save").into_any_element()}
    </>
}
```

## 🤝 贡献

欢迎贡献！请随时提交 Issue 或 Pull Request。

### 开发流程

1. Fork 项目
2. 创建功能分支: `git checkout -b feature/amazing-feature`
3. 提交更改: `git commit -m 'Add amazing feature'`
4. 推送分支: `git push origin feature/amazing-feature`
5. 提交 Pull Request

### 代码规范

- 使用 `rustfmt` 格式化代码
- 使用 `clippy` 检查代码质量
- 为新功能添加测试
- 更新文档

## 📝 许可

MIT License

## 🙏 致谢

灵感来源于：
- [Dioxus RSX](https://dioxuslabs.com/) - RSX 语法设计
- [Yew html! macro](https://yew.rs/) - html! 宏
- [React JSX](https://react.dev/) - JSX 语法
- [GPUI](https://www.gpui.rs/) - 底层 UI 框架

---

**让 GPUI 开发更加愉快！** 🎉
