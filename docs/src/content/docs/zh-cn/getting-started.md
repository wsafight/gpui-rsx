---
title: 快速开始
description: 安装 GPUI-RSX 并渲染第一个 GPUI 视图。
---

GPUI-RSX 是一个用于编写 GPUI 视图的过程宏。静态标记会在编译期展开为普通 GPUI builder 调用，因此最终 UI 仍是常规 Rust 代码。

## 安装

在应用的 `Cargo.toml` 中添加 Zed 仓库的 GPUI 和 GPUI-RSX：

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit", "runtime_shaders", "wayland", "x11"] }
gpui-rsx = "0.7"
```

应用项目应提交 `Cargo.lock`，以固定实际解析到的 GPUI git revision。

如果你从 crates.io 的 `gpui = "0.2.2"` 迁移，请先阅读[迁移指南](/gpui-rsx/zh-cn/guides/migration/#05x--cratesio-gpui-022--060--zed-git-gpui)。

## 引入

```rust
use gpui::*;
use gpui::prelude::*;
use gpui_platform::application;
use gpui_rsx::rsx;
```

## 第一个元素

最小的 RSX 表达式会创建一个 GPUI `div()`：

```rust
rsx! {
    <div>{"Hello, GPUI!"}</div>
}
```

文本子节点既可以写成字符串字面量，也可以写成 Rust 表达式：

```rust
rsx! {
    <div>
        "Count: "
        {self.count.to_string()}
    </div>
}
```

## 属性

Flag 属性会变成无参方法调用：

```rust
rsx! {
    <div flex flex_col rounded_md />
}
```

Value 属性会把 Rust 表达式传给 builder 方法：

```rust
rsx! {
    <div gap={px(16.0)} bg={rgb(0x3b82f6)} />
}
```

GPUI 使用 snake_case 的地方，可以用 CamelCase alias：

```rust
rsx! {
    <button onClick={handler} textColor={rgb(0xffffff)}>
        {"Save"}
    </button>
}
```

## Class 属性

稳定的 Tailwind-like 工具类列表建议使用 `class`：

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4 bg-blue-500 text-white">
        {"Content"}
    </div>
}
```

静态 class 字符串由宏在编译期解析。动态字符串由生成的运行时 matcher 处理，适合必须运行时组装的场景：

```rust
let row_class = if selected { "bg-blue-500 text-white" } else { "bg-gray-100" };

rsx! {
    <div class={row_class}>{label.as_str()}</div>
}
```

如果值本来就是 Rust 表达式，优先使用直接属性：

```rust
rsx! {
    <div class="flex" w={px(self.sidebar_width)} />
}
```

## 事件

事件属性接受普通 GPUI listener：

```rust
rsx! {
    <button
        class="px-4 py-2 bg-blue-500 text-white rounded-md cursor-pointer"
        onClick={cx.listener(|view, _, _window, cx| {
            view.count += 1;
            cx.notify();
        })}
    >
        {"Increment"}
    </button>
}
```

在 `{for ...}` 中渲染重复的有状态元素时，添加 `key={...}`，让每次迭代获得唯一身份：

```rust
rsx! {
    <ul>
        {for task in &self.tasks {
            <li key={task.id} onClick={handler}>
                {task.title.as_str()}
            </li>
        }}
    </ul>
}
```

## 完整视图

```rust
use gpui::*;
use gpui::prelude::*;
use gpui_platform::application;
use gpui_rsx::rsx;

struct HelloView;

impl Render for HelloView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="flex flex-col gap-3 p-4">
                <h1 class="text-xl font-bold">{"Hello GPUI-RSX"}</h1>
                <p class="text-gray-600">{"This markup expands to GPUI builder calls."}</p>
            </div>
        }
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_cx| HelloView)
        }).unwrap();
        cx.activate(true);
    });
}
```

## 检查 Demo

仓库内置 demo crate，用来验证真实 GPUI 集成：

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
```

demo lockfile 会固定 GPUI 和 `gpui-component`，方便复现兼容性问题。

## 下一步

- 阅读[语法参考](/gpui-rsx/zh-cn/usage/syntax/)了解元素、属性、子节点和 Fragment。
- 在依赖动态 class 前阅读 [Class 处理](/gpui-rsx/zh-cn/usage/class/)。
- 在循环中渲染可交互行前阅读 [ID 与 Key](/gpui-rsx/zh-cn/usage/ids/)。
- 对齐 GPUI、gpui-component、Rust 和 demo lockfile 时阅读 [兼容性](/gpui-rsx/zh-cn/compatibility/)。
