# Getting Started with GPUI-RSX

GPUI-RSX is a JSX-like macro for GPUI that makes UI development more intuitive and concise.

## Installation

Add GPUI-RSX to your `Cargo.toml`:

```toml
[dependencies]
gpui = "0.1"  # or your GPUI version
gpui-rsx = "0.1"
```

## Basic Usage

### Importing the Macro

```rust
use gpui::*;
use gpui_rsx::rsx;
```

### Your First Element

The simplest RSX creates a div:

```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        rsx! {
            <div>"Hello, GPUI!"</div>
        }
    }
}
```

### Adding Attributes

RSX supports both flag attributes and value attributes:

```rust
rsx! {
    <div
        flex
        flex_col
        gap={px(16.0)}
        bg={rgb(0x3b82f6)}
    >
        "Styled content"
    </div>
}
```

#### Flag Attributes

Flag attributes are method calls without arguments:

- `flex` → `.flex()`
- `rounded_md` → `.rounded_md()`
- `cursor_pointer` → `.cursor_pointer()`

#### Value Attributes

Value attributes take expressions:

- `gap={px(16.0)}` → `.gap(px(16.0))`
- `bg={rgb(0xff0000)}` → `.bg(rgb(0xff0000))`
- `onClick={handler}` → `.on_click(handler)`

### Class Attribute

The `class` attribute provides Tailwind-like syntax:

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4 bg-blue-500">
        "Content"
    </div>
}
```

Supported class patterns:

- **Layout**: `flex`, `flex-col`, `flex-row`, `grid`
- **Spacing**: `gap-4`, `p-4`, `px-2`, `py-1`, `m-4`, `mx-auto`
- **Sizing**: `w-full`, `h-screen`, `min-w-0`, `max-h-full`
- **Colors**: `bg-blue-500`, `text-red-600`, `border-gray-300`
- **Typography**: `text-xl`, `font-bold`, `text-center`
- **Borders**: `border`, `border-2`, `rounded-md`, `rounded-lg`

### Nesting Elements

Elements can be nested naturally:

```rust
rsx! {
    <div>
        <header>
            <h1>"Title"</h1>
        </header>
        <main>
            <p>"Content"</p>
        </main>
    </div>
}
```

### Expressions

Use `{...}` to embed Rust expressions:

```rust
rsx! {
    <div>
        {format!("Count: {}", self.count)}
        {self.render_child()}
    </div>
}
```

### Event Handlers

Event handlers use camelCase or snake_case:

```rust
rsx! {
    <button
        onClick={cx.listener(|view, _event, cx| {
            view.count += 1;
            cx.notify();
        })}
    >
        "Click me"
    </button>
}
```

Common event handlers:

- `onClick` / `on_click`
- `onMouseDown` / `on_mouse_down`
- `onKeyDown` / `on_key_down`
- `onHover` / `on_hover`

### Conditional Rendering

Use regular Rust `if` expressions:

```rust
rsx! {
    <div>
        {if self.show_message {
            rsx! { <span>"Visible"</span> }
        } else {
            rsx! { <span>"Hidden"</span> }
        }}
    </div>
}
```

Or use the `when` attribute:

```rust
rsx! {
    <div
        when={(self.is_active, |el| el.bg(rgb(0x00ff00)))}
    >
        "Conditionally styled"
    </div>
}
```

### Lists and Iteration

Use `for` loops to render lists:

```rust
rsx! {
    <ul>
        {for item in &self.items {
            <li>{item.clone()}</li>
        }}
    </ul>
}
```

### Fragments

Render multiple root elements with fragments:

```rust
rsx! {
    <>
        <div>"First"</div>
        <div>"Second"</div>
    </>
}
```

### Self-Closing Tags

Elements without children can use self-closing syntax:

```rust
rsx! {
    <div flex flex_col />
}
```

## Complete Example

```rust
use gpui::*;
use gpui_rsx::rsx;

struct CounterView {
    count: usize,
}

impl Render for CounterView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        rsx! {
            <div class="flex flex-col gap-4 p-8 bg-gray-100">
                <h1 class="text-3xl font-bold">"Counter App"</h1>

                <div class="text-xl">
                    {format!("Count: {}", self.count)}
                </div>

                <div class="flex gap-2">
                    <button
                        class="px-4 py-2 bg-blue-500 text-white rounded-md cursor-pointer"
                        onClick={cx.listener(|view, _, cx| {
                            view.count += 1;
                            cx.notify();
                        })}
                    >
                        "Increment"
                    </button>

                    <button
                        class="px-4 py-2 bg-red-500 text-white rounded-md cursor-pointer"
                        onClick={cx.listener(|view, _, cx| {
                            view.count = view.count.saturating_sub(1);
                            cx.notify();
                        })}
                    >
                        "Decrement"
                    </button>
                </div>
            </div>
        }
    }
}
```

## Next Steps

- Check out the [API Reference](api-reference.md) for complete documentation
- Read [Best Practices](best-practices.md) for tips and patterns
- See [Troubleshooting](troubleshooting.md) if you encounter issues
