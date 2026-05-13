# API Reference

Complete reference for GPUI-RSX syntax and features.

## Macro Syntax

```rust
rsx! { /* RSX content */ }
rsx_strict! { /* RSX content */ }
rsx_permissive! { /* RSX content */ }
rsx_expand! { /* RSX content */ }
```

- `rsx!` is the default permissive macro.
- `rsx_strict!` rejects unsupported static classes and panics on unsupported dynamic classes when evaluated.
- `rsx_permissive!` is an explicit alias for the default permissive class handling.
- `rsx_expand!` returns a string preview of the generated GPUI method chain for debugging.

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

String literal with space-separated class names. GPUI-RSX implements a Tailwind-like subset that maps to GPUI APIs; it is not a full Tailwind CSS engine.

```rust
<div class="flex flex-col gap-4 p-4" />
```

Dynamic expressions use a generated runtime matcher:

```rust
let classes = if active { "flex gap-4" } else { "block" };
rsx! { <div class={classes} /> }
```

Runtime class support includes common layout/spacing/typography utilities, the full color palette, arbitrary colors, arbitrary spacing/sizing lengths, fraction sizing, and opacity. Unknown dynamic classes are ignored in permissive mode and panic in strict mode.

#### Supported Class Patterns

##### Layout

- `flex` → `.flex()`
- `flex-col` → `.flex_col()`
- `flex-row` → `.flex_row()`
- `flex-1`, `flex-auto`, `flex-none`, `flex-grow`, `flex-grow-0`, `flex-shrink`, `flex-shrink-0`
- `min-w-0`, `min-h-0`
- `grid` → `.grid()`
- `debug-outline` → debug border in debug builds, no-op in release builds

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
- `gap-[14px]` → `.gap(px(14.0))`
- `gap-x-[0.75rem]` → `.gap_x(rems(0.75))`
- `p-[18px]`, `mx-[1.25rem]`, etc.

Percentage spacing such as `gap-[10%]` and `p-[10%]` is rejected because GPUI spacing APIs use definite lengths.

##### Sizing

- `w-N` → `.w(px(N))`
- `h-N` → `.h(px(N))`
- `size-N` → `.size(px(N))`
- `w-full`, `h-full`, `size-full`
- `w-auto`, `h-auto`
- `w-[280px]` → `.w(px(280.0))`
- `w-[18rem]` → `.w(rems(18.0))`
- `w-[37.5%]` → `.w(relative(0.375))`
- `min-w-[280px]`, `max-w-[32rem]`, `min-h-[48px]`, `max-h-[80%]`
- `w-6/24`, `h-1/2`, `size-3/4` → relative fraction sizing

Numeric sizing keeps GPUI-RSX's established pixel semantics: `w-64` means `.w(px(64.0))`, not Tailwind's `16rem` spacing scale.

##### Colors

Complete Tailwind color palette:

- `bg-COLOR-SHADE` → `.bg(rgb(0xHEX))`
- `text-COLOR-SHADE` → `.text_color(rgb(0xHEX))`
- `border-COLOR-SHADE` → `.border_color(rgb(0xHEX))`

Examples:
- `bg-blue-500` → `.bg(rgb(0x3b82f6))`
- `text-red-600` → `.text_color(rgb(0xdc2626))`
- `border-gray-300` → `.border_color(rgb(0xd1d5db))`

Arbitrary colors:
- `bg-[#ff0000]` → `.bg(rgb(0xff0000))`
- `text-[#a1b]` → `.text_color(rgb(0xaa11bb))`
- `border-[#11223344]` → `.border_color(rgba(0x11223344))`
- `bg-[rgb(15,23,42)]` → `.bg(rgb(0x0f172a))`
- `text-[rgba(15,23,42,0.8)]` → `.text_color(rgba(0x0f172acc))`

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
- `font-thin` → `.font_weight(FontWeight::THIN)`
- `font-extralight` → `.font_weight(FontWeight::EXTRA_LIGHT)`
- `font-light` → `.font_weight(FontWeight::LIGHT)`
- `font-normal` → `.font_weight(FontWeight::NORMAL)`
- `font-medium` → `.font_weight(FontWeight::MEDIUM)`
- `font-semibold` → `.font_weight(FontWeight::SEMIBOLD)`
- `font-bold` → `.font_weight(FontWeight::BOLD)`
- `font-extrabold` → `.font_weight(FontWeight::EXTRA_BOLD)`
- `font-black` → `.font_weight(FontWeight::BLACK)`
- `text-center` → `.text_center()`

##### Borders

- `border` → `.border_1()`
- `border-N` → `.border_N()`
- `border-t`, `border-b`, `border-l`, `border-r` → `.border_t_1()`, `.border_b_1()`, `.border_l_1()`, `.border_r_1()`
- `border-x`, `border-y`, `border-t-2`, etc. → directional GPUI border width methods
- `rounded-md` → `.rounded_md()`
- `rounded-lg` → `.rounded_lg()`

##### Alignment

- `items-start`, `items-center`, `items-end`, `items-baseline`, `items-stretch`
- `justify-start`, `justify-center`, `justify-end`, `justify-between`, `justify-around`, `justify-evenly`
- `content-normal`, `content-start`, `content-center`, `content-end`, `content-between`, `content-around`, `content-evenly`, `content-stretch`
- `self-start`, `self-end`, `self-flex-start`, `self-flex-end`, `self-center`, `self-baseline`, `self-stretch`

### Attribute Name Mapping

CamelCase JSX-style names map to snake_case Rust methods:

| JSX Name | Rust Method |
|----------|-------------|
| `minWidth` | `min_w` |
| `maxHeight` | `max_h` |
| `fontSize` | `text_size` |
| `lineHeight` | `line_height` |
| `trackFocus` | `track_focus` |
| `flexBasis` | `flex_basis` |
| `flexGrow` | `flex_grow` |
| `overflowScroll` | `overflow_scroll` |
| `trackScroll` | `track_scroll` |

`zIndex` is not mapped because GPUI 0.2 has no `z_index` builder; sibling paint order and overlay/popover APIs should be used instead.

### Event Handlers

#### Supported Events

| JSX Name | Rust Method |
|----------|-------------|
| `onClick` | `on_click` |
| `onMouseDown` | `on_mouse_down(button, handler)` |
| `onMouseUp` | `on_mouse_up(button, handler)` |
| `onMouseMove` | `on_mouse_move` |
| `onMouseDownOut` | `on_mouse_down_out` |
| `onMouseUpOut` | `on_mouse_up_out(button, handler)` |
| `onAnyMouseDown` | `on_any_mouse_down` |
| `onAnyMouseUp` | `on_any_mouse_up` |
| `onKeyDown` | `on_key_down` |
| `onKeyUp` | `on_key_up` |
| `onModifiersChanged` | `on_modifiers_changed` |
| `onHover` | `on_hover` |
| `onScrollWheel` | `on_scroll_wheel` |
| `onDrag` | `on_drag(value, constructor)` |
| `onDragMove` | `on_drag_move` |
| `onDrop` | `on_drop` |
| `onAction` | `on_action` |
| `onBoxedAction` | `on_boxed_action(action, handler)` |
| `captureAnyMouseDown` | `capture_any_mouse_down` |
| `captureAnyMouseUp` | `capture_any_mouse_up` |
| `captureKeyDown` | `capture_key_down` |
| `captureKeyUp` | `capture_key_up` |
| `captureAction` | `capture_action` |

Multi-argument GPUI methods use tuple syntax in RSX:
`onMouseDown={(MouseButton::Left, handler)}` and `onDrag={(value, constructor)}`.

#### Event Handler Syntax

```rust
<button
    onClick={cx.listener(|view, event, _window, cx| {
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
- `onHover`, `onDrag`
- `tooltip`, `focusable`, `active`, `groupActive`
- `overflowScroll`, `overflowXScroll`, `overflowYScroll`, `trackScroll`
- Static classes `overflow-scroll`, `overflow-x-scroll`, `overflow-y-scroll`

### Explicit IDs

Override auto-generated IDs:

```rust
<div id="my-custom-id" onClick={handler} />
```

## Examples

### Desktop Three-Column Layout

```rust
rsx! {
    <div class="flex h-full w-full bg-zinc-100">
        <nav class="w-[72px] min-w-[72px] bg-zinc-950" />

        <aside class="w-[280px] min-w-[280px] border-r border-zinc-200 bg-white">
            "Projects"
        </aside>

        <main class="flex-1 min-w-0 p-[18px]">
            "Conversation, plans, diff, and results"
        </main>

        <aside class="w-6/24 min-w-[320px] max-w-[460px] border-l border-zinc-200 bg-white">
            "Execution trace"
        </aside>
    </div>
}
```

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
            onClick={cx.listener(|view, _, _window, cx| {
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
