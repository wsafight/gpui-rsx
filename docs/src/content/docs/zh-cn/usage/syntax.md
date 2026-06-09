---
title: 语法参考
description: 元素、属性、子节点、Fragment、条件渲染和特殊 GPUI 标签。
---

本页说明 RSX 语法。Class 工具类范围见 [Class 处理](/gpui-rsx/zh-cn/usage/class/)，生成 ID 的规则见 [ID 与 Key](/gpui-rsx/zh-cn/usage/ids/)。

## 元素

RSX 元素使用 HTML-like 结构：

```rust
rsx! {
    <tag attr value={expr}>
        {child_expr}
    </tag>
}
```

常见 HTML-like 标签会映射为 `div()`，因为 GPUI 并没有浏览器 DOM 节点：

| 分组 | 标签 |
| --- | --- |
| 结构 | `div`, `span`, `section`, `article`, `header`, `footer`, `main`, `nav`, `aside` |
| 文本 | `h1`, `h2`, `h3`, `h4`, `h5`, `h6`, `p`, `label`, `a` |
| 表单 | `button`, `input`, `textarea`, `select`, `form` |
| 列表 | `ul`, `ol`, `li` |

无子节点时，自闭合和成对标签等价：

```rust
rsx! { <div flex /> }
rsx! { <div flex></div> }
```

## 特殊标签

部分标签会直接使用 GPUI 构造器：

| RSX | 生成的构造器 |
| --- | --- |
| `<svg src={path} />` | `svg().path(path)` |
| `<img src={source} />` | `img(source)` |
| `<img source={source} />` | `img(source)` |
| `<canvas prepaint={prepaint} paint={paint} />` | `canvas(prepaint, paint)` |

`img` 必须提供 `src` 或 `source`。`canvas` 必须同时提供 `prepaint` 和 `paint`。

## 组件

未知的单段标签会被视为构造函数：

```rust
rsx! {
    <MyComponent title={"Inbox"} />
}
```

路径标签会调用对应路径：

```rust
rsx! {
    <ui::TaskCard title={task.title.clone()} />
}
```

当 builder 需要参数，或来自组件库时，使用 `base={expr}`：

```rust
rsx! {
    <Button
        base={Button::new("save")}
        label={"Save"}
        small
    />
}
```

`base` 会被宏消费，不会生成 `.base(...)` 方法调用。

## 属性

Flag 属性会生成无参方法：

```rust
rsx! { <div flex flex_col items_center /> }
```

Value 属性会生成单参方法：

```rust
rsx! {
    <div gap={px(16.0)} bg={rgb(0x3b82f6)} />
}
```

多参数 GPUI 方法使用 tuple 语法：

```rust
rsx! {
    <div onMouseDown={(MouseButton::Left, handler)} />
}
```

常见 CamelCase alias 会映射到 GPUI 的 snake_case 方法：

| RSX 属性 | GPUI 方法 |
| --- | --- |
| `onClick` | `on_click` |
| `onKeyDown` | `on_key_down` |
| `onMouseDown` | `on_mouse_down` |
| `textColor` | `text_color` |
| `backgroundColor` | `bg` |
| `fontSize` | `text_size` |
| `minWidth` / `maxHeight` | `min_w` / `max_h` |
| `overflowScroll` | `overflow_scroll` |
| `trackScroll` | `track_scroll` |

`whiteSpace` 会被显式拒绝。可使用 `whitespace-nowrap` 等 class，或在 GPUI 可用时直接调用对应方法。

## 子节点

字符串字面量可以直接作为子节点：

```rust
rsx! { <div>"Hello"</div> }
```

Rust 表达式需要放在花括号中：

```rust
rsx! {
    <div>
        {format!("Count: {}", self.count)}
        {self.render_footer()}
    </div>
}
```

嵌套元素会生成 `.child(...)`：

```rust
rsx! {
    <section>
        <h1 styled>{"Title"}</h1>
        <p>{"Body"}</p>
    </section>
}
```

已有 iterable 可以用 spread 子节点：

```rust
rsx! {
    <div>
        {...rows}
    </div>
}
```

## For 循环

使用 `{for pattern in iter { ... }}` 渲染迭代器：

```rust
rsx! {
    <ul>
        {for (index, item) in items.iter().enumerate() {
            <li>{format!("{}: {}", index, item.name)}</li>
        }}
    </ul>
}
```

如果循环体里包含有状态元素，需要添加 `key={...}` 或 `id={...}`。详见 [ID 与 Key](/gpui-rsx/zh-cn/usage/ids/)。

## Fragment

Fragment 可以返回多个根元素：

```rust
rsx! {
    <>
        <div>{"First"}</div>
        <div>{"Second"}</div>
    </>
}
```

Fragment 会展开为 `vec![...]`。如果根元素具体类型不同，通常需要 `into_any_element()` 或包一层共同父元素。

## 条件渲染

子节点中可以使用 Rust 表达式：

```rust
rsx! {
    <div>
        {if let Some(user) = &self.user {
            rsx! { <span>{user.name.as_str()}</span> }
        } else {
            rsx! { <span>{"Guest"}</span> }
        }}
    </div>
}
```

使用 `when` 条件转换当前正在构建的元素：

```rust
rsx! {
    <div
        class="px-4 py-2"
        when={(active, |el| el.bg(rgb(0x3b82f6)).text_color(rgb(0xffffff)))}
    >
        {"Status"}
    </div>
}
```

当条件值是 `Option` 时使用 `whenSome`：

```rust
rsx! {
    <div whenSome={(custom_width, |el, width| el.w(px(width)))} />
}
```

使用 `whenClass` 应用条件静态 class：

```rust
rsx! {
    <div
        class="px-2"
        whenClass={(active, "bg-neutral-900 text-white")}
        whenClass={(!active, "text-neutral-600")}
    />
}
```

`whenClass` 的 class 必须是字符串字面量，并且会拒绝 `overflow-scroll` 这类有状态 class。涉及 ID 语义时请使用 `when`。

## 特殊属性

`visible={bool}` 会映射到 `.visible()` 或 `.invisible()`，且表达式只求值一次：

```rust
rsx! {
    <div visible={self.show_sidebar} />
}
```

`grayscale` 会映射为 `.grayscale(true)`：

```rust
rsx! {
    <img src={source} grayscale />
}
```

`styled` 会在用户属性前注入标签默认 class：

| 标签 | 默认 class |
| --- | --- |
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

用户属性会在 `styled` 之后应用，因此后续值可以覆盖默认值：

```rust
rsx! {
    <h1 styled class="text-xl">{title.as_str()}</h1>
}
```
