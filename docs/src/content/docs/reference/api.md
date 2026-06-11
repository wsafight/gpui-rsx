---
title: API Reference
description: Compact reference for GPUI-RSX macros, tags, attributes, and generated mappings.
---

## Macros

```rust
use gpui_rsx::{rsx, rsx_expand, rsx_permissive, rsx_strict};
```

| Macro | Behavior |
| --- | --- |
| `rsx!` | Default permissive RSX macro. |
| `rsx_permissive!` | Explicit permissive mode; unsupported classes are ignored when safe. |
| `rsx_strict!` | Unsupported static classes become compile errors; unsupported dynamic classes panic when evaluated. |
| `rsx_expand!` | Returns a string preview of generated GPUI builder code. |

## Element Constructors

| RSX tag | Generated constructor |
| --- | --- |
| `div`, `span`, `section`, `article`, `header`, `footer`, `main`, `nav`, `aside` | `div()` |
| `h1` through `h6`, `p`, `label`, `a` | `div()` |
| `button`, `input`, `textarea`, `select`, `form` | `div()` |
| `ul`, `ol`, `li` | `div()` |
| `svg` | `svg()` |
| `img src={source}` or `img source={source}` | `img(source)` |
| `canvas prepaint={a} paint={b}` | `canvas(a, b)` |
| `MyComponent` | `MyComponent()` |
| `ui::TaskCard` | `ui::TaskCard()` |
| any tag with `base={expr}` | `expr` |

## Attribute Rules

| RSX | Generated form |
| --- | --- |
| `<div flex />` | `.flex()` |
| `<div gap={px(4.0)} />` | `.gap(px(4.0))` |
| `<div class="flex gap-4" />` | compile-time class method expansion |
| `<div class={classes} />` | generated runtime class matcher |
| `<div visible={cond} />` | `.visible()` or `.invisible()` |
| `<img grayscale />` | `.grayscale(true)` |
| `<Button base={Button::new("id")} />` | starts method chain from `Button::new("id")` |
| `<button key={id} onClick={h} />` | generated composite `.id(...)` plus `.on_click(h)` |

`id`, `key`, and `base` are macro-level attributes. `id` generates `.id(...)`; `key` and `base` are consumed by the macro.

## CamelCase Mapping

| RSX attribute | GPUI method |
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

Multi-argument methods use tuple values:

```rust
rsx! {
    <div
        onMouseDown={(MouseButton::Left, handler)}
        onDrag={(drag_value, build_drag)}
    />
}
```

`groupDragOver` / `group_drag_over` is not available as an attribute because GPUI requires an
explicit drag data type. Use `when` or `base` and call `group_drag_over::<YourType>(...)` directly.

## Conditional Attributes

| Attribute | Shape | Generated idea |
| --- | --- | --- |
| `when` | `when={(condition, |el| ... )}` | `.when(condition, closure)` |
| `whenSome` | `whenSome={(option, |el, value| ... )}` | `.when_some(option, closure)` |
| `whenClass` | `whenClass={(condition, "class list")}` | `.when(condition, |el| el...)` |

`whenClass` requires a string literal and rejects stateful scroll classes.

## Class Support Summary

Static class strings are the broadest and fastest path. Dynamic class strings support the common runtime matcher subset.

| Area | Examples |
| --- | --- |
| Layout | `flex`, `flex-col`, `grid`, `hidden`, `absolute`, `relative` |
| Alignment | `items-center`, `justify-between`, `content-stretch`, `self-center` |
| Spacing | `gap-4`, `gap-x-2`, `p-4`, `px-2`, `m-4`, `mt-2` |
| Sizing | `w-full`, `w-64`, `w-[280px]`, `w-[37.5%]`, `w-6/24` |
| Colors | `bg-blue-500`, `text-white`, `border-gray-300`, `bg-[#ff0000]` |
| Typography | `text-sm`, `text-2xl`, `font-bold`, `text-center`, `truncate` |
| Borders | `border`, `border-t`, `border-2`, `rounded-md`, `rounded-full` |
| Misc | `cursor-pointer`, `debug-outline`, `shadow-md`, `opacity-50`, `col-span-full` |

Numeric utilities use direct pixel semantics: `w-64` means `w(px(64.0))`.

## Stateful ID Triggers

The macro injects IDs for attributes and static classes that require GPUI stateful identity:

| Category | Names |
| --- | --- |
| Events | `onClick`, `onHover`, `onDrag`, `onAuxClick`, `onA11yAction` |
| Interactive state | `active`, `activeClass`, `groupActive`, `focusable`, `tooltip`, `hoverableTooltip`, `tooltipShowDelay`, `anchorScroll`, `trackScroll`, `scrollbarWidth` |
| Accessibility | `role`, `ariaLabel`, `ariaSelected`, `ariaExpanded`, and other `aria*` attributes |
| Scroll classes | `overflow-scroll`, `overflow-x-scroll`, `overflow-y-scroll` |

Inside `{for ...}`, stateful elements must provide `id={...}` or `key={...}`. `groupHover`
does not require an ID; `groupActive` does.

## Return Shapes

| RSX body | Return expression |
| --- | --- |
| Single element | generated GPUI element expression |
| Fragment `<>...</>` | `vec![...]` |
| Child expression `{expr}` | `.child(expr)` |
| Spread `{...iter}` | `.children(iter)` |
| For loop `{for item in iter { ... }}` | `.children(iter.into_iter().map(...))` or `.flat_map(...)` |
