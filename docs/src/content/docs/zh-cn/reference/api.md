---
title: API 参考
description: GPUI-RSX 宏、标签、属性和生成映射的速查。
---

## 宏

```rust
use gpui_rsx::{rsx, rsx_expand, rsx_permissive, rsx_strict};
```

| 宏 | 行为 |
| --- | --- |
| `rsx!` | 默认 permissive RSX 宏。 |
| `rsx_permissive!` | 显式 permissive 模式；安全情况下忽略不支持的 class。 |
| `rsx_strict!` | 不支持的静态 class 编译时报错；不支持的动态 class 求值时 panic。 |
| `rsx_expand!` | 返回生成 GPUI builder 代码的字符串预览。 |

## 元素构造器

| RSX 标签 | 生成的构造器 |
| --- | --- |
| `div`, `span`, `section`, `article`, `header`, `footer`, `main`, `nav`, `aside` | `div()` |
| `h1` 到 `h6`, `p`, `label`, `a` | `div()` |
| `button`, `input`, `textarea`, `select`, `form` | `div()` |
| `ul`, `ol`, `li` | `div()` |
| `svg` | `svg()` |
| `img src={source}` 或 `img source={source}` | `img(source)` |
| `canvas prepaint={a} paint={b}` | `canvas(a, b)` |
| `MyComponent` | `MyComponent()` |
| `ui::TaskCard` | `ui::TaskCard()` |
| 任意带 `base={expr}` 的标签 | `expr` |

## 属性规则

| RSX | 生成形式 |
| --- | --- |
| `<div flex />` | `.flex()` |
| `<div gap={px(4.0)} />` | `.gap(px(4.0))` |
| `<div class="flex gap-4" />` | 编译期 class 方法展开 |
| `<div class={classes} />` | 生成运行时 class matcher |
| `<div visible={cond} />` | `.visible()` 或 `.invisible()` |
| `<img grayscale />` | `.grayscale(true)` |
| `<Button base={Button::new("id")} />` | 从 `Button::new("id")` 开始方法链 |
| `<button key={id} onClick={h} />` | 生成复合 `.id(...)` 并调用 `.on_click(h)` |

`id`、`key`、`base` 是宏级属性。`id` 会生成 `.id(...)`；`key` 和 `base` 会被宏消费。

## CamelCase 映射

| RSX 属性 | GPUI 方法 |
| --- | --- |
| `onClick` | `on_click` |
| `onMouseDown` | `on_mouse_down` |
| `onMouseUp` | `on_mouse_up` |
| `onMouseMove` | `on_mouse_move` |
| `onHover` | `on_hover` |
| `onDrag` | `on_drag` |
| `onDrop` | `on_drop` |
| `onKeyDown` / `onKeyUp` | `on_key_down` / `on_key_up` |
| `captureKeyDown` / `captureKeyUp` | `capture_key_down` / `capture_key_up` |
| `backgroundColor` | `bg` |
| `textColor` | `text_color` |
| `fontSize` | `text_size` |
| `fontWeight` | `font_weight` |
| `lineHeight` | `line_height` |
| `minWidth` / `maxWidth` | `min_w` / `max_w` |
| `minHeight` / `maxHeight` | `min_h` / `max_h` |
| `gapX` / `gapY` | `gap_x` / `gap_y` |
| `overflowScroll` | `overflow_scroll` |
| `overflowXScroll` / `overflowYScroll` | `overflow_x_scroll` / `overflow_y_scroll` |
| `trackScroll` | `track_scroll` |
| `groupHover` / `groupActive` | `group_hover` / `group_active` |
| `tabIndex` / `tabStop` | `tab_index` / `tab_stop` |

多参数方法使用 tuple 值：

```rust
rsx! {
    <div
        onMouseDown={(MouseButton::Left, handler)}
        onDrag={(drag_value, build_drag)}
    />
}
```

`groupDragOver` / `group_drag_over` 不作为属性提供，因为 GPUI 需要显式拖拽数据类型。
请使用 `when` 或 `base` 并直接调用 `group_drag_over::<YourType>(...)`。

## 条件属性

| 属性 | 形状 | 生成思路 |
| --- | --- | --- |
| `when` | `when={(condition, |el| ... )}` | `.when(condition, closure)` |
| `whenSome` | `whenSome={(option, |el, value| ... )}` | `.when_some(option, closure)` |
| `whenClass` | `whenClass={(condition, "class list")}` | `.when(condition, |el| el...)` |

`whenClass` 需要字符串字面量，并拒绝有状态滚动 class。

## Class 支持概览

静态 class 字符串覆盖范围最广、性能最好。动态 class 字符串使用常用运行时 matcher 子集。

| 范围 | 示例 |
| --- | --- |
| 布局 | `flex`, `flex-col`, `grid`, `hidden`, `absolute`, `relative` |
| 对齐 | `items-center`, `justify-between`, `content-stretch`, `self-center` |
| 间距 | `gap-4`, `gap-x-2`, `p-4`, `px-2`, `m-4`, `mt-2` |
| 尺寸 | `w-full`, `w-64`, `w-[280px]`, `w-[37.5%]`, `w-6/24` |
| 颜色 | `bg-blue-500`, `text-white`, `border-gray-300`, `bg-[#ff0000]` |
| 排版 | `text-sm`, `text-2xl`, `font-bold`, `text-center`, `truncate` |
| 边框 | `border`, `border-t`, `border-2`, `rounded-md`, `rounded-full` |
| 其他 | `cursor-pointer`, `debug-outline`, `shadow-md`, `opacity-50`, `col-span-full` |

数值工具类使用直接像素语义：`w-64` 表示 `w(px(64.0))`。

## 有状态 ID 触发项

宏会为需要 GPUI 有状态身份的属性和静态 class 注入 ID：

| 分类 | 名称 |
| --- | --- |
| 事件 | `onClick`, `onHover`, `onDrag`, `onAuxClick`, `onA11yAction` |
| 交互状态 | `active`, `activeClass`, `groupActive`, `focusable`, `tooltip`, `hoverableTooltip`, `tooltipShowDelay`, `anchorScroll`, `trackScroll`, `scrollbarWidth` |
| 无障碍 | `role`, `ariaLabel`, `ariaSelected`, `ariaExpanded` 以及其他 `aria*` 属性 |
| 滚动 class | `overflow-scroll`, `overflow-x-scroll`, `overflow-y-scroll` |

在 `{for ...}` 中，有状态元素必须提供 `id={...}` 或 `key={...}`。`groupHover` 不需要
ID；`groupActive` 需要。

## 返回形状

| RSX body | 返回表达式 |
| --- | --- |
| 单个元素 | 生成 GPUI 元素表达式 |
| Fragment `<>...</>` | `vec![...]` |
| 子表达式 `{expr}` | `.child(expr)` |
| Spread `{...iter}` | `.children(iter)` |
| For 循环 `{for item in iter { ... }}` | `.children(iter.into_iter().map(...))` 或 `.flat_map(...)` |
