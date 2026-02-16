# API Reference

Complete reference for GPUI-RSX syntax and features.

## Macro Syntax

```rust
rsx! { /* RSX content */ }
```

## Elements

### Element Syntax

```rust
<tag_name attributes...>children</tag_name>
```

### Supported HTML Tags

All standard HTML tags map to `div()` in GPUI:

- **Structure**: `div`, `span`, `section`, `article`, `header`, `footer`, `main`, `nav`, `aside`
- **Typography**: `h1`, `h2`, `h3`, `h4`, `h5`, `h6`, `p`, `label`
- **Interactive**: `button`, `a`
- **Forms**: `input`, `textarea`, `select`, `form`
- **Lists**: `ul`, `ol`, `li`

### Special Tags

- `svg` → `svg()`
- `img` → `img()`
- `canvas` → `canvas()`

### Self-Closing Tags

```rust
<div flex flex_col />
```

Equivalent to:

```rust
<div flex flex_col></div>
```

## Attributes

### Flag Attributes

Boolean attributes without values:

```rust
<div flex rounded_md cursor_pointer />
```

Generates:

```rust
div().flex().rounded_md().cursor_pointer()
```

### Value Attributes

Attributes with expressions:

```rust
<div
    gap={px(16.0)}
    bg={rgb(0x3b82f6)}
    w={px(200.0)}
/>
```

Generates:

```rust
div()
    .gap(px(16.0))
    .bg(rgb(0x3b82f6))
    .w(px(200.0))
```

### Class Attribute

String literal with space-separated class names:

```rust
<div class="flex flex-col gap-4 p-4" />
```

#### Supported Class Patterns

##### Layout

- `flex` → `.flex()`
- `flex-col` → `.flex_col()`
- `flex-row` → `.flex_row()`
- `grid` → `.grid()`

##### Spacing

- `gap-N` → `.gap(px(N))`
- `p-N` → `.p(px(N))`
- `px-N` → `.px(px(N))`
- `py-N` → `.py(px(N))`
- `pt-N` → `.pt(px(N))`
- `pb-N` → `.pb(px(N))`
- `pl-N` → `.pl(px(N))`
- `pr-N` → `.pr(px(N))`
- `m-N` → `.m(px(N))`
- `mx-N` → `.mx(px(N))`
- `my-N` → `.my(px(N))`
- `mt-N` → `.mt(px(N))`
- `mb-N` → `.mb(px(N))`
- `ml-N` → `.ml(px(N))`
- `mr-N` → `.mr(px(N))`

##### Sizing

- `w-N` → `.w(px(N))`
- `h-N` → `.h(px(N))`

##### Colors

Complete Tailwind color palette:

- `bg-COLOR-SHADE` → `.bg(rgb(0xHEX))`
- `text-COLOR-SHADE` → `.text_color(rgb(0xHEX))`
- `border-COLOR-SHADE` → `.border_color(rgb(0xHEX))`

Examples:
- `bg-blue-500` → `.bg(rgb(0x3b82f6))`
- `text-red-600` → `.text_color(rgb(0xdc2626))`
- `border-gray-300` → `.border_color(rgb(0xd1d5db))`

Arbitrary hex colors:
- `bg-[#ff0000]` → `.bg(rgb(0xff0000))`
- `text-[#a1b]` → `.text_color(rgb(0xaa11bb))`

Supported color families:
- `slate`, `gray`, `zinc`, `neutral`, `stone`
- `red`, `orange`, `amber`, `yellow`, `lime`, `green`
- `emerald`, `teal`, `cyan`, `sky`, `blue`
- `indigo`, `violet`, `purple`, `fuchsia`, `pink`, `rose`
- `white`, `black`

Shades: `50`, `100`, `200`, `300`, `400`, `500`, `600`, `700`, `800`, `900`, `950`

##### Typography

- `text-xs` → `.text_xs()`
- `text-sm` → `.text_sm()`
- `text-base` → `.text_base()`
- `text-lg` → `.text_lg()`
- `text-xl` → `.text_xl()`
- `text-2xl` → `.text_2xl()`
- `text-3xl` → `.text_3xl()`
- `text-4xl` → `.text_4xl()`
- `text-5xl` → `.text_5xl()`
- `font-bold` → `.font_bold()`
- `text-center` → `.text_center()`

##### Borders

- `border` → `.border_1()`
- `border-N` → `.border_N()`
- `rounded-md` → `.rounded_md()`
- `rounded-lg` → `.rounded_lg()`

### Attribute Name Mapping

CamelCase JSX-style names map to snake_case Rust methods:

| JSX Name | Rust Method |
|----------|-------------|
| `zIndex` | `z_index` |
| `minWidth` | `min_w` |
| `maxHeight` | `max_h` |
| `fontSize` | `font_size` |
| `lineHeight` | `line_height` |
| `borderRadius` | `border_radius` |
| `flexBasis` | `basis` |
| `flexGrow` | `flex_grow` |

### Event Handlers

#### Supported Events

| JSX Name | Rust Method |
|----------|-------------|
| `onClick` | `on_click` |
| `onMouseDown` | `on_mouse_down` |
| `onMouseUp` | `on_mouse_up` |
| `onMouseMove` | `on_mouse_move` |
| `onKeyDown` | `on_key_down` |
| `onKeyUp` | `on_key_up` |
| `onFocus` | `on_focus` |
| `onBlur` | `on_blur` |
| `onHover` | `on_hover` |
| `onScrollWheel` | `on_scroll_wheel` |
| `onDrag` | `on_drag` |
| `onDrop` | `on_drop` |

#### Event Handler Syntax

```rust
<button
    onClick={cx.listener(|view, event, cx| {
        // handle click
    })}
>
    "Click me"
</button>
```

### Conditional Attributes

#### `when` Attribute

Conditionally apply styling:

```rust
<div
    when={(condition, |el| el.bg(rgb(0x00ff00)))}
>
    "Content"
</div>
```

Generates:

```rust
div().when(condition, |el| el.bg(rgb(0x00ff00)))
```

#### `whenSome` Attribute

Apply styling when Option is Some:

```rust
<div
    whenSome={(option_value, |el, value| el.child(value))}
>
    "Content"
</div>
```

### Special Flags

#### `styled` Flag

Apply tag-specific default styles:

```rust
<h1 styled>"Title"</h1>
```

Default styles:

| Tag | Default Classes |
|-----|----------------|
| `h1` | `text-3xl font-bold` |
| `h2` | `text-2xl font-bold` |
| `h3` | `text-xl font-bold` |
| `h4` | `text-lg font-bold` |
| `h5` | `text-base font-bold` |
| `h6` | `text-sm font-bold` |
| `button` | `cursor-pointer` |
| `a` | `cursor-pointer` |
| `input` | `px-2 py-1` |
| `textarea` | `px-2 py-1` |
| `ul`, `ol` | `flex flex-col` |

## Children

### Text Children

String literals:

```rust
<div>"Hello, world!"</div>
```

### Expression Children

Any Rust expression in braces:

```rust
<div>
    {format!("Count: {}", count)}
    {self.render_child()}
</div>
```

### Element Children

Nested elements:

```rust
<div>
    <span>"Child 1"</span>
    <span>"Child 2"</span>
</div>
```

### Spread Children

Spread iterables:

```rust
<div>
    {...children_iter}
</div>
```

### For Loops

Iterate and render:

```rust
<ul>
    {for item in &items {
        <li>{item.name.clone()}</li>
    }}
</ul>
```

With pattern matching:

```rust
<div>
    {for (index, item) in items.iter().enumerate() {
        <div>{format!("{}: {}", index, item)}</div>
    }}
</div>
```

## Fragments

Render multiple root elements:

```rust
rsx! {
    <>
        <div>"First"</div>
        <div>"Second"</div>
        <div>"Third"</div>
    </>
}
```

Returns `Vec<impl IntoElement>`.

## Type System

### Return Types

- **Single element**: `impl IntoElement`
- **Fragment**: `Vec<impl IntoElement>`

### Auto ID Generation

Elements with interactive attributes automatically get IDs:

```rust
<div onClick={handler} />
```

Generates:

```rust
div().id("__rsx_div_HASH").on_click(handler)
```

Attributes triggering auto-ID:
- `onClick`, `on_click`
- `hover`, `active`, `focus`
- `tooltip`, `group`, `track_focus`

### Explicit IDs

Override auto-generated IDs:

```rust
<div id="my-custom-id" onClick={handler} />
```

## Examples

### Complex Layout

```rust
rsx! {
    <div class="flex h-screen">
        <aside class="w-64 bg-gray-800 text-white">
            <nav class="p-4">
                {for item in &nav_items {
                    <a class="block py-2 cursor-pointer">
                        {item.label.clone()}
                    </a>
                }}
            </nav>
        </aside>

        <main class="flex-1 p-8">
            <header class="mb-8">
                <h1 styled>"Dashboard"</h1>
            </header>

            <div class="grid gap-4">
                {...content_widgets}
            </div>
        </main>
    </div>
}
```

### Form with Events

```rust
rsx! {
    <form class="flex flex-col gap-4">
        <label class="flex flex-col">
            "Username"
            <input
                class="px-2 py-1 border rounded"
                value={self.username.clone()}
            />
        </label>

        <button
            class="px-4 py-2 bg-blue-500 text-white rounded cursor-pointer"
            when={(self.is_valid(), |el| el.bg(rgb(0x00aa00)))}
            onClick={cx.listener(|view, _, cx| {
                view.submit_form(cx);
            })}
        >
            "Submit"
        </button>
    </form>
}
```

### Conditional Rendering

```rust
rsx! {
    <div>
        {if let Some(user) = &self.user {
            rsx! {
                <div class="flex items-center gap-2">
                    <span>{user.name.clone()}</span>
                    <button onClick={logout_handler}>"Logout"</button>
                </div>
            }
        } else {
            rsx! {
                <button onClick={login_handler}>"Login"</button>
            }
        }}
    </div>
}
```
