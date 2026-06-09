---
title: 最佳实践
description: 编写可维护、类型安全且高效的 GPUI-RSX 视图的实用模式。
---

## 按工作流组织视图

当一个 render block 不再容易扫读时，把可复用 UI 提取到方法或小 builder 中：

```rust
impl MyView {
    fn render_header(&self) -> impl IntoElement {
        rsx! {
            <header class="flex items-center justify-between p-4 border-b border-gray-200">
                <h1 styled>{self.title.as_str()}</h1>
                {self.render_actions()}
            </header>
        }
    }

    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="flex flex-col h-full">
                {self.render_header()}
                <main class="flex-1 min-h-0">{self.render_content()}</main>
            </div>
        }
    }
}
```

优先按真实概念拆分，例如 header、sidebar、row、editor、status panel。不要把每个小元素都拆成方法。

## 保持类型可预测

GPUI-RSX 生成的是普通 Rust 表达式，所以 Rust 类型规则仍然生效。渲染辅助方法建议返回 `impl IntoElement`：

```rust
fn render_item(&self, item: &Item) -> impl IntoElement {
    rsx! {
        <li class="flex items-center gap-2">
            {item.name.as_str()}
        </li>
    }
}
```

当分支类型不好统一时，用共同父元素包住分支输出：

```rust
fn render_body(&self) -> impl IntoElement {
    rsx! {
        <div>
            {match &self.state {
                State::Loading => rsx! { <span>{"Loading"}</span> },
                State::Ready(data) => rsx! { <section>{data.title.as_str()}</section> },
                State::Error(error) => rsx! { <p class="text-red-600">{error.as_str()}</p> },
            }}
        </div>
    }
}
```

## 列表里优先用引用

避免在 render 循环里做不必要的 clone：

```rust
rsx! {
    <ul>
        {for item in &self.items {
            <li>{item.name.as_str()}</li>
        }}
    </ul>
}
```

如果重复的行是可交互的，添加 `key`：

```rust
rsx! {
    <ul>
        {for item in &self.items {
            <li key={item.id} onClick={cx.listener(Self::select_item)}>
                {item.name.as_str()}
            </li>
        }}
    </ul>
}
```

## 按稳定性选择样式方式

稳定结构使用字面量 class：

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4 bg-white border border-gray-200 rounded-md" />
}
```

值本来就是 Rust 表达式时，用直接属性：

```rust
rsx! {
    <aside class="min-w-0 border-r border-gray-200" w={px(self.sidebar_width)} />
}
```

少量静态变体使用 `whenClass`：

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

只有 class 字符串确实需要运行时组装时，再使用动态 `class={expr}`。动态字符串应保持短且可预测。

## 不把复杂事件逻辑塞进标记

很小的 listener 可以内联。真正的业务逻辑建议移到命名方法里：

```rust
impl MyView {
    fn handle_submit(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.validate() {
            self.submit(cx);
            cx.notify();
        }
    }
}

rsx! {
    <button onClick={cx.listener(Self::handle_submit)}>
        {"Submit"}
    </button>
}
```

相关状态修改应合并后只调用一次 `cx.notify()`：

```rust
onClick={cx.listener(|view, _, _window, cx| {
    view.count += 1;
    view.last_update = Instant::now();
    view.recompute_summary();
    cx.notify();
})}
```

## 有意识地选择条件形式

子节点结构不同，用 Rust `if` 或 `match`：

```rust
rsx! {
    <div>
        {if self.items.is_empty() {
            rsx! { <p class="text-gray-500">{"No items"}</p> }
        } else {
            self.render_items()
        }}
    </div>
}
```

同一个元素只需要可选 builder 调用时，用 `when`：

```rust
rsx! {
    <div
        class="px-4 py-2"
        when={(self.has_error, |el| el.bg(rgb(0xfef2f2)).text_color(rgb(0xb91c1c)))}
    >
        {message.as_str()}
    </div>
}
```

条件值是 `Option` 时使用 `whenSome`：

```rust
rsx! {
    <div whenSome={(self.max_width, |el, width| el.max_w(px(width)))} />
}
```

## 调试宏展开

本地预览可以使用 `rsx_expand!`：

```rust
let preview = gpui_rsx::rsx_expand! {
    <div class="flex gap-4 bg-blue-500" />
};
```

完整类型检查和编译器诊断仍然以 `cargo check` 或仓库 demo 检查为准：

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
```

## 避免这些模式

避免单个超大的 RSX block。扫读困难时，按 UI 概念拆分。

避免只为了子节点传值而 clone。优先使用 `as_str()`、引用或小的渲染辅助方法。

避免把 `key` 当作通用列表标记。只有元素需要生成有状态 ID 时，`key` 才会生效。

避免依赖 `hover:bg-blue-500` 等不支持的 Tailwind variant。使用 GPUI 方法、`when` 或显式状态。
