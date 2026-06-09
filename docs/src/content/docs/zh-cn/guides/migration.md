---
title: 迁移指南
description: 从手写 GPUI builder 链迁移，并升级到 GPUI-RSX 0.6 的 GPUI 目标。
---

## 从手写 GPUI 迁移到 RSX

GPUI-RSX 可以渐进采用。手写 GPUI builder 和 RSX 输出都是普通 GPUI 元素，可以共存。

### 元素和属性

迁移前：

```rust
div()
    .flex()
    .flex_col()
    .gap(px(16.0))
    .bg(rgb(0x3b82f6))
```

迁移后：

```rust
rsx! {
    <div class="flex flex-col gap-4 bg-blue-500" />
}
```

如果源码里已经有类型化的 Rust 值，使用直接属性：

```rust
rsx! {
    <div w={px(width)} bg={self.background_color()} />
}
```

### 子节点

迁移前：

```rust
div()
    .child("Hello")
    .child(div().child("World"))
    .child(button().child("Click"))
```

迁移后：

```rust
rsx! {
    <div>
        "Hello"
        <span>{"World"}</span>
        <button>{"Click"}</button>
    </div>
}
```

### 事件

迁移前：

```rust
button()
    .on_click(cx.listener(|view, _event, _window, cx| {
        view.count += 1;
        cx.notify();
    }))
    .child("Click")
```

迁移后：

```rust
rsx! {
    <button onClick={cx.listener(|view, _event, _window, cx| {
        view.count += 1;
        cx.notify();
    })}>
        {"Click"}
    </button>
}
```

### 列表

迁移前：

```rust
let mut list = div().flex().flex_col();

for item in &self.items {
    list = list.child(div().child(item.name.clone()));
}

list
```

迁移后：

```rust
rsx! {
    <div class="flex flex-col">
        {for item in &self.items {
            <div>{item.name.as_str()}</div>
        }}
    </div>
}
```

如果行是有状态元素，添加 `key={...}`：

```rust
rsx! {
    <div class="flex flex-col">
        {for item in &self.items {
            <button key={item.id} onClick={handler}>
                {item.name.as_str()}
            </button>
        }}
    </div>
}
```

### 条件渲染

子节点不同，用 Rust 条件：

```rust
rsx! {
    <div>
        {if show_message {
            rsx! { <span>{"Message visible"}</span> }
        } else {
            rsx! { <span>{"Message hidden"}</span> }
        }}
    </div>
}
```

同一个元素只需要可选 builder 调用时，用 `when`：

```rust
rsx! {
    <div when={(show_message, |el| el.child("Message visible"))} />
}
```

## 0.5.x / crates.io GPUI 0.2.2 到 0.6.0 / Zed Git GPUI

GPUI-RSX `0.6.x` 目标是 Zed 仓库中的 GPUI，而不是 crates.io 的 `gpui = "0.2.2"` 包。

Zed git 依赖仍会把 package version 显示为 `gpui v0.2.2`，但 API 面与 crates.io release 不同。这会影响 `flex_grow_1`、`flex_shrink_1`、对齐 helper、桌面尺寸工具类和新的启动路径等生成调用。

### 替换依赖

迁移前：

```toml
[dependencies]
gpui = "0.2.2"
gpui-rsx = "0.5"
```

迁移后：

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit", "runtime_shaders", "wayland", "x11"] }
gpui-rsx = "0.6"
```

应用项目应提交 `Cargo.lock`，固定实际解析到的 Zed revision。

如果应用也依赖 `gpui-component`，直接依赖的 `gpui` 和 `gpui_platform` 应与 `gpui-component` 解析到的 source 和 revision 保持一致。混用 bare Zed git 和另一份 pinned `rev` 容易生成重复 GPUI crate，导致组件类型不兼容。

### 更新应用启动方式

迁移前：

```rust
Application::new().run(|cx: &mut App| {
    cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| MyView));
});
```

迁移后：

```rust
use gpui_platform::application;

application().run(|cx: &mut App| {
    cx.open_window(WindowOptions::default(), |_, cx| {
        cx.new(|_| MyView)
    }).unwrap();
    cx.activate(true);
});
```

### 验证依赖图

仓库 demo crate 是一个已知可用的集成目标：

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui
```

如果 `cargo tree` 显示 `git+https://github.com/zed-industries/zed#...`，即使 package version 是 `0.2.2`，项目也在使用 Zed git GPUI。

## 渐进迁移计划

1. 在一个 view module 中添加 `gpui_rsx::rsx` import。
2. 先转换小的静态子树。
3. 静态结构通过编译后，再转换列表和重复行。
4. 只给需要有状态 ID 的重复元素添加 `key={...}`。
5. 每个有意义的步骤后运行 `cargo check`。

混合写法是可以接受的：

```rust
rsx! {
    <div>
        {self.legacy_gpui_builder()}
        <section class="flex flex-col gap-4">
            {self.render_new_rsx_panel()}
        </section>
    </div>
}
```

## 常见迁移模式

| 手写 GPUI | GPUI-RSX |
| --- | --- |
| `div().child("Hello")` | `rsx! { <div>{"Hello"}</div> }` |
| `div().child(format!("Count: {count}"))` | `rsx! { <div>{format!("Count: {count}")}</div> }` |
| `.w(px(200.0)).h(px(100.0))` | `<div w={px(200.0)} h={px(100.0)} />` |
| `.flex().gap(px(4.0))` | `<div class="flex gap-4" />` |
| 带参数的 builder 构造器 | `<Button base={Button::new("id")} />` |

## 检查清单

- [ ] `gpui` 和 `gpui_platform` 依赖使用 Zed git GPUI。
- [ ] 应用启动方式使用 `gpui_platform::application()`。
- [ ] 应用项目提交了 `Cargo.lock`。
- [ ] 如果使用 `gpui-component`，它解析到同一份 GPUI source。
- [ ] 新 RSX 模块引入了 `gpui_rsx::rsx`。
- [ ] 重复的有状态元素有 `key={...}` 或显式 `id={...}`。
- [ ] 迁移后的视图通过 `cargo check`。
- [ ] 状态变化后仍然调用 `cx.notify()`。
