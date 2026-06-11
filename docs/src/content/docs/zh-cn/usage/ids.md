---
title: ID 与 Key
description: GPUI-RSX 如何生成 ID，以及循环中如何使用 key。
---

GPUI-RSX 会在 GPUI API 需要元素身份时生成稳定 ID。

大多数布局和样式调用不需要 ID。只有使用有状态 GPUI 方法时，宏才会注入 ID。

## 自动 ID

循环外的静态 RSX 可以由宏根据调用点生成 ID，常见标记因此可以保持简洁：

```rust
rsx! {
    <button onClick={handler}>
        {"Click"}
    </button>
}
```

静态有状态滚动 class 也会触发自动 ID：

```rust
rsx! {
    <div class="overflow-y-scroll">
        {"Scrollable content"}
    </div>
}
```

生成的 ID 对同一个宏展开调用点是确定的。

常见触发 ID 的属性包括 `onClick`、`onHover`、`onDrag`、`onAuxClick`、`onA11yAction`、`active`、`activeClass`、`groupActive`、`tooltip`、`tooltipShowDelay`、`focusable`、`role`、`ariaLabel`、`overflowScroll`、`overflowXScroll`、`overflowYScroll`、`trackScroll`、`scrollbarWidth`，以及静态 `overflow-scroll` class 变体。

当前 GPUI 目标下，`hover`、`hoverClass`、`focus`、`focusClass`、`group`、`groupHover`、`onMouseDown`、`captureKeyDown` 等属性本身不会触发有状态 ID。

## 循环中的 Key

渲染重复的有状态元素时，需要提供 `key={...}`：

```rust
rsx! {
    <div>
        {for task in &self.tasks {
            <button key={task.id} onClick={handler} class="flex items-center gap-2">
                {task.title.clone()}
            </button>
        }}
    </div>
}
```

如果循环中需要有状态 ID 但缺少 key，宏会给出编译错误，避免多行错误共享同一个生成 ID。

`key` 是宏专用属性，不会变成 `.key(...)` 方法调用。

## 字面量 Key 与动态 Key

字面量 key 可以使用 `concat!` 展开：

```rust
rsx! {
    <button key="toolbar" onClick={handler} />
}
```

动态 key 使用格式化，以保证每行或每个有状态实例都有唯一身份。表达式需要实现 `Display`：

```rust
rsx! {
    <button key={task.id} onClick={handler} />
}
```

如果元素不需要 ID，`key` 会被忽略：

```rust
rsx! {
    <div key={task.id} class="flex items-center">
        {task.title.as_str()}
    </div>
}
```

上面的例子不会因为 `key` 变成有状态元素，因为没有使用有状态方法。

## 显式 ID

显式 `id` 总是优先生效：

```rust
rsx! {
    <button id="save-button" key={task.id} onClick={handler}>
        {"Save"}
    </button>
}
```

当身份需要在源码行号移动或重构后仍保持稳定时，使用显式 ID。

## 什么时候添加 Key

以下情况建议添加 key：

- 元素出现在 `{for ...}` 中；
- class 或属性使用了 GPUI 的有状态行为；
- 元素身份需要在重排或更新中保持稳定。

简单、非重复元素通常不需要手写 key，交给宏在需要时生成 ID 即可。
