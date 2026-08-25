---
title: 性能优化
description: GPUI-RSX 如何展开、哪些路径会产生运行时工作，以及如何保持 render 轻量。
---

GPUI-RSX 的设计目标是让常见路径在编译期生成普通 GPUI 代码。因此，性能优化的重点通常是：尽量保持标记静态，避免在 render 中产生不必要的分配和状态刷新。

## 心智模型

静态 RSX 会在宏执行期间展开：

```rust
rsx! {
    <div class="flex gap-4 bg-blue-500">
        {"Hello"}
    </div>
}
```

生成的表达式类似普通 GPUI builder 链：

```rust
div()
    .flex()
    .gap(px(4.0))
    .bg(rgb(0x3b82f6))
    .child("Hello")
```

静态标签、属性、class 字符串、颜色和很多布局工具类都会在应用运行前解析完成。

## 优先使用静态 Class 字符串

稳定 UI 结构使用字面量 `class`：

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4 bg-white border border-gray-200 rounded-md" />
}
```

字面量 class 没有运行时 parser 成本，也能让 strict 模式给出更有用的编译期诊断。

## 保持条件 Class 可编译期优化

如果 `class={...}` 内部的 `if` 或 `match` 每个分支都是字符串字面量，宏可以直接优化：

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

这个优化只作用于传给 `class` 的表达式。如果先把结果赋值给变量，宏看到的就是运行时表达式，会使用动态 matcher。

## 谨慎使用动态 Class

动态 class 适合 class 字符串确实必须运行时组装的情况：

```rust
let class_name = format!("w-[{}px] bg-{}", width, tone);

rsx! {
    <div class={class_name} />
}
```

这条路径会生成运行时 matcher 来处理 class token。它支持常见布局、间距、尺寸、颜色、透明度、排版、边框、任意长度和任意颜色，但这些仍然是在 render 期间执行的工作。

如果值本来就是 Rust 表达式，优先使用直接 GPUI 属性：

```rust
rsx! {
    <aside class="flex flex-col min-w-0" w={px(self.sidebar_width)} />
}
```

少量状态变体优先使用 `when` 或 `whenClass`：

```rust
rsx! {
    <button
        class="px-3 py-2 rounded-md"
        whenClass={(selected, "bg-blue-500 text-white")}
        whenClass={(!selected, "bg-gray-100 text-gray-900")}
    >
        {label.as_str()}
    </button>
}
```

## 避免 Render 循环里的分配

Render 可能频繁执行。每次 render 的分配都应当是有意的：

```rust
rsx! {
    <ul>
        {for item in &self.items {
            <li>{item.name.as_str()}</li>
        }}
    </ul>
}
```

除非确实需要，避免在循环里 clone 大字符串或重复构造格式化 class 名。

为可见文本做少量 `format!` 是正常的：

```rust
rsx! {
    <span>{format!("Count: {}", self.count)}</span>
}
```

重复标签优先使用借用字符串或预先计算好的显示文本。

## 合并状态通知

状态更新通常应只触发一次 render 通知：

```rust
onClick={cx.listener(|view, _, _window, cx| {
    view.count += 1;
    view.last_update = Instant::now();
    view.recompute_summary();
    cx.notify();
})}
```

避免每改一个字段就调用一次 `cx.notify()`。

## 理解 ID 和 Key 的成本

只有需要 GPUI 有状态身份的元素才会生成自动 ID。常见触发项包括 `onClick`、`onHover`、
`onDrag`、`onAuxClick`、`onA11yAction`、`active`、`activeClass`、`groupActive`、
`tooltip`、`hoverableTooltip`、`tooltipShowDelay`、`focusable`、`role`、`aria*`、
`trackScroll`、`overflowScroll`、`overflowXScroll`、`overflowYScroll` 和静态
`overflow-scroll` class 变体。

`hover`、`hoverClass`、`focus`、`focusClass`、`group`、`groupHover` 和 `scrollbarWidth`
走样式 refinement 路径，本身不会触发自动 ID 注入。

普通布局元素不会因为带了 `key={...}` 就变成有状态元素：

```rust
rsx! {
    <div key={task.id} class="flex items-center">
        {task.title.as_str()}
    </div>
}
```

上面的 `key` 会被忽略，因为元素不需要 ID。循环里的重复有状态元素才需要添加 `key`：

```rust
rsx! {
    <button key={task.id} onClick={handler}>
        {task.title.as_str()}
    </button>
}
```

字面量 key 使用 `concat!`；动态 key 使用格式化，因此任何实现 `Display` 的值都可以参与生成 ID。

## 保持 RSX Block 易读

很大的宏输入会增加编译期工作，也会让编译错误更难定位。按真实 UI 概念拆分视图：

```rust
fn render_sidebar(&self) -> impl IntoElement {
    rsx! {
        <aside class="flex flex-col w-[280px] min-w-[280px] border-r border-gray-200">
            {self.render_sidebar_header()}
            {self.render_project_list()}
        </aside>
    }
}
```

这主要是可维护性优化，也能让增量修改更小。

## 查看生成代码

使用 `rsx_expand!` 检查 class 是否保持静态，还是进入了动态 matcher：

```rust
let preview = gpui_rsx::rsx_expand! {
    <div class={if active { "flex gap-4" } else { "block" }} />
};
```

完整类型验证仍以 `cargo check` 为准。

## 验证性能敏感改动

仓库包含 class 解析/codegen 的 benchmark 目标：

```bash
cargo bench --bench class_performance --no-run
```

真实 GPUI 集成使用 demo crate 检查：

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
```

文档示例和导航用文档站构建验证：

```bash
cd docs
pnpm run build
```
