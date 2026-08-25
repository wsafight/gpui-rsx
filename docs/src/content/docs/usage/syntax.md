---
title: Syntax Reference
description: Elements, attributes, children, fragments, conditionals, and special GPUI tags.
---

This page covers RSX syntax. For class utility details, see [Class Handling](/gpui-rsx/usage/class/). For generated identity rules, see [IDs and Keys](/gpui-rsx/usage/ids/).

## Elements

RSX elements use an HTML-like shape:

```rust
rsx! {
    <tag attr value={expr}>
        {child_expr}
    </tag>
}
```

Common HTML-like tags map to `div()` because GPUI does not expose browser DOM nodes:

| Group | Tags |
| --- | --- |
| Structure | `div`, `span`, `section`, `article`, `header`, `footer`, `main`, `nav`, `aside` |
| Text | `h1`, `h2`, `h3`, `h4`, `h5`, `h6`, `p`, `label`, `a` |
| Forms | `button`, `input`, `textarea`, `select`, `form` |
| Lists | `ul`, `ol`, `li` |

Self-closing and paired tags are equivalent when there are no children:

```rust
rsx! { <div flex /> }
rsx! { <div flex></div> }
```

## Special Tags

Some tags use GPUI constructors directly:

| RSX | Generated constructor |
| --- | --- |
| `<svg src={path} />` | `svg().path(path)` |
| `<img src={source} />` | `img(source)` |
| `<img source={source} />` | `img(source)` |
| `<canvas prepaint={prepaint} paint={paint} />` | `canvas(prepaint, paint)` |

`img` requires `src` or `source`. `canvas` requires both `prepaint` and `paint`.

## Components

Unknown single-segment tags are treated as constructor functions:

```rust
rsx! {
    <MyComponent title={"Inbox"} />
}
```

Path-qualified tags call the path:

```rust
rsx! {
    <ui::TaskCard title={task.title.clone()} />
}
```

Use `base={expr}` when the builder needs arguments or comes from a component library:

```rust
rsx! {
    <Button
        base={Button::new("save")}
        label={"Save"}
        small
    />
}
```

`base` is consumed by the macro. It does not generate a `.base(...)` method call.

## Attributes

Flag attributes become zero-argument methods:

```rust
rsx! { <div flex flex_col items_center /> }
```

Value attributes become one-argument methods:

```rust
rsx! {
    <div gap={px(16.0)} bg={rgb(0x3b82f6)} />
}
```

Multi-argument GPUI methods use tuple syntax:

```rust
rsx! {
    <div onMouseDown={(MouseButton::Left, handler)} />
}
```

CamelCase aliases map to GPUI snake_case methods where needed:

| RSX attribute | GPUI method |
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
| `accessibilityId` / `ariaDescription` | `accessibility_id` / `aria_description` |
| `ariaKeyShortcuts` / `ariaValue` | `aria_keyshortcuts` / `aria_value` |
| `ariaActiveDescendant` | `aria_active_descendant` |
| `a11ySyntheticChildren` | `a11y_synthetic_children` |
| `restrictScrollToAxis` | `restrict_scroll_to_axis` |
| `externalDragPayload` | `external_drag_payload` |
| `onMouseExit` / `onMousePressure` / `onPinch` | `on_mouse_exit` / `on_mouse_pressure` / `on_pinch` |
| `gridColsMinContent` / `gridRowsMaxContent` | `grid_cols_min_content` / `grid_rows_max_content` |

`externalDragPayload` must follow `onDrag` on the same element and use the same dragged value type,
matching GPUI's method contract. The macro preserves attribute order; it does not reorder these calls.

`whiteSpace` is intentionally rejected. Use GPUI's supported whitespace helpers through class utilities such as `whitespace-nowrap`, or direct GPUI methods when available.

## Children

String literals are valid children:

```rust
rsx! { <div>"Hello"</div> }
```

Rust expressions are wrapped in braces:

```rust
rsx! {
    <div>
        {format!("Count: {}", self.count)}
        {self.render_footer()}
    </div>
}
```

Nested elements become `.child(...)` calls:

```rust
rsx! {
    <section>
        <h1 styled>{"Title"}</h1>
        <p>{"Body"}</p>
    </section>
}
```

Use spread children for existing iterables:

```rust
rsx! {
    <div>
        {...rows}
    </div>
}
```

## For Loops

Use `{for pattern in iter { ... }}` to render an iterator as children:

```rust
rsx! {
    <ul>
        {for (index, item) in items.iter().enumerate() {
            <li>{format!("{}: {}", index, item.name)}</li>
        }}
    </ul>
}
```

If a loop body contains stateful elements, add `key={...}` or `id={...}`. See [IDs and Keys](/gpui-rsx/usage/ids/).

## Fragments

Fragments return multiple root elements:

```rust
rsx! {
    <>
        <div>{"First"}</div>
        <div>{"Second"}</div>
    </>
}
```

A fragment expands to `vec![...]`. Mixed concrete root types may need `into_any_element()` or a common wrapper element.

## Conditional Rendering

Rust expressions work inside children:

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

Use `when` to conditionally transform the element being built:

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

Use `whenSome` when the conditional value is an `Option`:

```rust
rsx! {
    <div whenSome={(custom_width, |el, width| el.w(px(width)))} />
}
```

Use `whenClass` for conditional static class strings:

```rust
rsx! {
    <div
        class="px-2"
        whenClass={(active, "bg-neutral-900 text-white")}
        whenClass={(!active, "text-neutral-600")}
    />
}
```

`whenClass` requires a string literal and rejects stateful classes such as `overflow-scroll`. Use `when` for ID-sensitive methods.

## Special Attributes

`visible={bool}` maps to `.visible()` or `.invisible()` while evaluating the expression once:

```rust
rsx! {
    <div visible={self.show_sidebar} />
}
```

`grayscale` maps to `.grayscale(true)`:

```rust
rsx! {
    <img src={source} grayscale />
}
```

`styled` injects default classes for semantic tags before user attributes:

| Tag | Default classes |
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

User attributes are applied after `styled`, so later values can override defaults:

```rust
rsx! {
    <h1 styled class="text-xl">{title.as_str()}</h1>
}
```
