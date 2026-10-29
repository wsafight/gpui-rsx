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

对于 `<ui::card onClick={handler} />` 这样的组件标签，是否自动注入 ID 取决于构造函数
的返回类型：同时实现 `gpui::InteractiveElement` 和 `gpui::Element` 的值会获得 ID，
因此返回 `Div` 的函数仍保持原有行为。其他值（包括 `#[component]` 构建器和
gpui-component 的 `Button`）保留自身的事件 setter，不会被自动 `.id()` 包装。
`base={...}` 表达式也使用同样的规则。

分派依据宏调用点可见的类型和 trait 约束。泛型或不透明返回类型需暴露上述两个约束才能
自动注入 ID。如果类型已支持 `.id()`，但隐藏了 `Element` 约束，可以显式设置 `id={...}`。
如果只返回 `impl IntoElement`，应在构造函数内部设置 ID 和有状态方法，此时具体类型仍可见。

常见触发 ID 的属性包括 `onClick`、`onHover`、`onDrag`、`onAuxClick`、`onA11yAction`、`active`、`activeClass`、`groupActive`、`tooltip`、`tooltipShowDelay`、`focusable`、`role`、`accessibilityId`、`ariaLabel`、`ariaDescription`、`ariaKeyShortcuts`、`ariaActiveDescendant`、`a11ySyntheticChildren`、`ariaNumericValueStep`、`ariaValue`、`ariaPlaceholder`、`overflowScroll`、`overflowXScroll`、`overflowYScroll`、`restrictScrollToAxis`、`trackScroll`、`externalDragPayload`，以及静态 `overflow-scroll` class 变体。

当前 GPUI 目标下，`hover`、`hoverClass`、`focus`、`focusClass`、`group`、`groupHover`、`scrollbarWidth`、`onMouseDown`、`onMouseExit`、`onMousePressure`、`captureMousePressure`、`onPinch`、`capturePinch`、`captureKeyDown` 等属性本身不会触发有状态 ID。

`accessibilityId` 设置的是暴露给辅助技术的作者标识，不是 `.id(...)` 创建的 GPUI 元素身份；
宏仍会先注入一份独立的 GPUI ID，再调用 `.accessibility_id(...)`。
`ariaActiveDescendant` 和 `a11ySyntheticChildren` 还需要配合合适的 `role`，才能生成有意义的无障碍节点。

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

循环中的组件标签只要带有状态属性名，就仍须提供 `key` 或 `id`，即使构建器最终不需要
注入 ID：宏先检查循环安全，Rust 再解析实际类型。构建器即使不使用生成的 ID，key 表达式
也仍会求值一次。

`key` 是宏专用属性，不会变成 `.key(...)` 方法调用。

## 字面量 Key 与动态 Key

字面量 key 可以使用 `concat!` 展开：

```rust
rsx! {
    <button key="toolbar" onClick={handler} />
}
```

字面量 key 使用 `concat!`。具体类型的动态整数（不超过 64 位，含 `usize` / `isize`）
使用 GPUI 的 `(&'static str, u64)` ID，无需格式化或分配字符串。其他 `Display` 值保留
字符串格式化路径，`u128` / `i128` 也走此路径以避免截断。整数引用和泛型 `T: Display`
也可能走格式化路径。表达式借用求值一次，不会移动 String key；大型有状态列表优先
使用 `key={task.id}` 这样的整数值。

示例：

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
