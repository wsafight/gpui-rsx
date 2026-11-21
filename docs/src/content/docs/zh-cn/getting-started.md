---
title: 快速开始
description: 安装 GPUI-RSX 并渲染第一个 GPUI 视图。
---

用 RSX 标签和 Rust 表达式编写 GPUI 视图，宏会在编译期将静态标记展开为 GPUI builder 调用。本页从依赖配置开始，带你打开第一个窗口。

:::note[GPUI 元素与 HTML-like 名称]
`button`、`input` 等名称是 GPUI `div()` 的别名，不提供 HTML 输入和表单行为。使用组件库控件时，请显式指定构造方式，例如 `<Button("save") label={"Save"} />` 或 `<Button base={Button::new("save")} label={"Save"} />`；导入和构造规则见[组件](/gpui-rsx/zh-cn/usage/syntax/#组件)。
:::

## 安装

在应用的 `Cargo.toml` 中添加 GPUI Kit 和 GPUI-RSX；将 Kit 命名为 `gpui`，供宏展开使用：

```toml
[dependencies]
gpui = { package = "gpui-kit", version = "=0.10.0" }
gpui-rsx = "0.13.0"
```

Kit 固定配套的 `gpui-pre` 快照，并导出 GPUI、组件和资源。应用应提交 `Cargo.lock`，固定完整依赖图。

如果你从 Zed Git 或官方 `gpui = "0.2.2"` 迁移，请采用[迁移指南](/gpui-rsx/zh-cn/guides/migration/)中的当前 Kit 迁移章节。

## 完整视图

创建 `src/main.rs`，写入下面的完整视图。需要同时引入 GPUI 和 prelude，让宏生成的 builder 方法在作用域内可用。

```rust
use gpui::*;
use gpui::prelude::*;
use gpui_rsx::rsx;

struct HelloView;

impl Render for HelloView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="size-full flex flex-col gap-[12px] p-[24px]">
                <h1 class="text-xl font-bold">{"Hello GPUI-RSX"}</h1>
                <p class="text-gray-600">{"This markup expands to GPUI builder calls."}</p>
            </div>
        }
    }
}

fn main() {
    application().with_assets(assets::Assets).run(|cx: &mut App| {
        gpui::init(cx);
        gpui::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_cx| HelloView)
        }).unwrap();
        cx.activate(true);
    });
}
```

## 运行应用

在应用目录执行：

```sh
cargo run
```

GPUI 需要对应平台的原生构建依赖。请参考[兼容性矩阵](/gpui-rsx/zh-cn/compatibility/)对齐 GPUI Kit、Rust 和组件版本。

想先试用现成应用，可以从[可运行示例](/gpui-rsx/zh-cn/examples/demos/)选择：`counter` 适合了解事件，`component` 适合查看原生控件。

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

:::note[默认间距以像素为单位]
`p-4` 和 `gap-4` 表示 4px。若需要每单位 4px 的 Tailwind 刻度，请使用 `rsx_tw!` 或[项目配置](/gpui-rsx/zh-cn/guides/theming/)。`gap_4` 等具名属性继续使用 GPUI 原生单位，不受 class 刻度影响。
:::

静态 class 字符串以及条件字面量数组都会在编译期直接展开，无运行时解析开销：

```rust
rsx! {
    <div
        class={[
            "flex items-center p-8 rounded-md",
            if selected { "bg-blue-500 text-white" } else { "bg-gray-100" },
            disabled && "opacity-50",
        ]}
    >
        {label.as_str()}
    </div>
}
```

如果尺寸或颜色本来就是计算好的 Rust 表达式，直接写成具名属性最直观：

```rust
rsx! {
    <div class="flex" w={px(self.sidebar_width)} bg={cx.theme().background} />
}
```

对于必须在运行时外部拼装的字符串（`class={row_class}`），宏会生成基于 `StyleRefinement` 的轻量匹配器。动态字符串不支持状态前缀（`hover:`、`active:`）和有状态滚动类；完整能力边界见 [Class 处理](/gpui-rsx/zh-cn/usage/class/)。

## 控件、事件与列表

使用 `gpui-component` 控件时，可以直接用 `<Button("id")>` 传入构造参数（展开为 `Button::new("id")`），配合 `rsx_listener!(cx, notify, ...)` 更新状态并自动通知重绘：

```rust
use gpui::component::button::{Button, ButtonVariants as _};
use gpui_rsx::{rsx, rsx_listener};

rsx! {
    <Button("increment")
        label="增加"
        primary
        onClick={rsx_listener!(cx, notify, |view, _, _, _| {
            view.count += 1;
        })}
    />
}
```

原生 GPUI 容器同样可以直接接收 `cx.listener(...)`：

```rust
rsx! {
    <button
        class="px-8 py-4 bg-blue-500 text-white rounded-md cursor-pointer"
        onClick={cx.listener(|view, _, _window, cx| {
            view.count += 1;
            cx.notify();
        })}
    >
        {"Increment"}
    </button>
}
```

在 `{for ...}` 中渲染重复的可交互元素时，添加 `key={...}` 让每行保持稳定的元素身份（若等高列表超过几百条，可改用虚拟列表 `<uniform_list>`）：

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

## 检查 Demo

仓库内置 demo crate，用来验证真实 GPUI 集成：

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
```

demo lockfile 会固定 Kit、GPUI 快照和 `gpui-component`，方便复现兼容性问题。

可以直接运行完整的多面板示例：

```bash
cargo run --manifest-path demo/Cargo.toml --bin incident_console --locked
```

## 下一步

- 用[主题与设计令牌](/gpui-rsx/zh-cn/guides/theming/)统一颜色、间距与排版。
- 阅读 [CLI 与编辑器](/gpui-rsx/zh-cn/guides/tooling/)配置补全、诊断与格式化。
- 阅读[语法参考](/gpui-rsx/zh-cn/usage/syntax/)了解元素、属性、子节点和 Fragment。
- 在依赖动态 class 前阅读 [Class 处理](/gpui-rsx/zh-cn/usage/class/)。
- 在循环中渲染可交互行前阅读 [ID 与 Key](/gpui-rsx/zh-cn/usage/ids/)。
- 阅读[事故响应控制台](/gpui-rsx/zh-cn/examples/incident-console/)查看模块化应用示例。
- 对齐 GPUI、gpui-component、Rust 和 demo lockfile 时阅读 [兼容性](/gpui-rsx/zh-cn/compatibility/)。
