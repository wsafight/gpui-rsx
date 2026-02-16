# Best Practices

Guidelines and patterns for effective GPUI-RSX development.

## Code Organization

### Component Extraction

Extract reusable UI into separate functions:

```rust
impl MyView {
    fn render_header(&self) -> impl IntoElement {
        rsx! {
            <header class="flex justify-between p-4 bg-gray-100">
                <h1 styled>{self.title.clone()}</h1>
                {self.render_nav()}
            </header>
        }
    }

    fn render_nav(&self) -> impl IntoElement {
        rsx! {
            <nav class="flex gap-2">
                {for item in &self.nav_items {
                    <a class="cursor-pointer">
                        {item.label.clone()}
                    </a>
                }}
            </nav>
        }
    }

    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        rsx! {
            <div>
                {self.render_header()}
                <main>{self.render_content()}</main>
            </div>
        }
    }
}
```

### Avoid Deep Nesting

Break down complex UIs into smaller, focused components:

**Don't:**

```rust
rsx! {
    <div>
        <div>
            <div>
                <div>
                    <div>
                        // Too deep!
                    </div>
                </div>
            </div>
        </div>
    </div>
}
```

**Do:**

```rust
// Split into multiple methods
fn render_card(&self) -> impl IntoElement {
    rsx! { <div>{self.render_card_content()}</div> }
}

fn render_card_content(&self) -> impl IntoElement {
    rsx! { <div>{ /* content */ }</div> }
}
```

## Performance

### Minimize Cloning

Avoid unnecessary clones in loops:

**Less efficient:**

```rust
{for item in &self.items {
    <div>{item.name.clone()}</div>
}}
```

**More efficient:**

```rust
{for item in &self.items {
    <div>{item.name.as_str()}</div>
}}
```

### Use References When Possible

```rust
// Good: pass by reference
fn render_list(&self, items: &[Item]) -> impl IntoElement {
    rsx! {
        <ul>
            {for item in items {
                <li>{item.display()}</li>
            }}
        </ul>
    }
}
```

### Batch Updates

Group related state changes:

```rust
onClick={cx.listener(|view, _, cx| {
    // Batch multiple updates
    view.count += 1;
    view.last_update = now();
    view.update_history();
    // Single notify for all changes
    cx.notify();
})}
```

## Styling

### Use Class Attribute for Static Styles

Prefer `class` attribute for unchanging styles:

**Preferred:**

```rust
<div class="flex flex-col gap-4 p-4 bg-blue-500" />
```

**Verbose:**

```rust
<div
    flex
    flex_col
    gap={px(16.0)}
    p={px(16.0)}
    bg={rgb(0x3b82f6)}
/>
```

### Use Individual Attributes for Dynamic Styles

For dynamic values, use individual attributes:

```rust
<div
    class="flex flex-col"
    bg={self.get_background_color()}
    h={px(self.height)}
/>
```

### Consistent Color Usage

Define color constants:

```rust
const PRIMARY: u32 = 0x3b82f6;
const DANGER: u32 = 0xef4444;
const SUCCESS: u32 = 0x10b981;

rsx! {
    <button class="px-4 py-2" bg={rgb(PRIMARY)}>
        "Primary Action"
    </button>
}
```

Or use Tailwind classes:

```rust
<button class="px-4 py-2 bg-blue-500">"Primary Action"</button>
```

## Event Handling

### Extract Event Handlers

For complex logic, extract handlers:

```rust
impl MyView {
    fn handle_submit(&mut self, cx: &mut ViewContext<Self>) {
        if self.validate() {
            self.submit_data(cx);
            cx.notify();
        }
    }
}

// In render:
rsx! {
    <button onClick={cx.listener(Self::handle_submit)}>
        "Submit"
    </button>
}
```

### Use Descriptive Handler Names

```rust
impl MyView {
    fn handle_increment_click(&mut self, cx: &mut ViewContext<Self>) {
        self.count += 1;
        cx.notify();
    }

    fn handle_reset_click(&mut self, cx: &mut ViewContext<Self>) {
        self.count = 0;
        cx.notify();
    }
}
```

### Avoid Inline Complex Logic

**Don't:**

```rust
<button onClick={cx.listener(|view, _, cx| {
    // 20 lines of complex logic
    view.data.iter_mut().for_each(|item| {
        // more complex operations
    });
    // ...
})}>
    "Submit"
</button>
```

**Do:**

```rust
<button onClick={cx.listener(Self::handle_complex_submit)}>
    "Submit"
</button>
```

## Conditional Rendering

### Use `when` Attribute for Simple Cases

```rust
<div
    class="px-4 py-2"
    when={(self.is_active, |el| el.bg(rgb(0x00ff00)))}
>
    "Status"
</div>
```

### Use Rust `if` for Complex Conditions

```rust
{if self.loading {
    rsx! { <div>"Loading..."</div> }
} else if let Some(data) = &self.data {
    rsx! { <div>{data.render()}</div> }
} else {
    rsx! { <div>"No data"</div> }
}}
```

### Pattern Matching

```rust
{match &self.state {
    State::Loading => rsx! { <div>"Loading..."</div> },
    State::Success(data) => rsx! { <div>{data.display()}</div> },
    State::Error(err) => rsx! { <div class="text-red-600">{err.to_string()}</div> },
}}
```

## Lists and Iteration

### Use References in Loops

```rust
{for item in &self.items {
    <div>{item.name.as_str()}</div>
}}
```

### Add Keys for Dynamic Lists

When items can be reordered or removed, use unique IDs:

```rust
{for item in &self.items {
    <div id={&item.id}>
        {item.name.as_str()}
    </div>
}}
```

### Handle Empty Lists

```rust
{if self.items.is_empty() {
    rsx! { <div class="text-gray-500">"No items"</div> }
} else {
    rsx! {
        <ul>
            {for item in &self.items {
                <li>{item.name.as_str()}</li>
            }}
        </ul>
    }
}}
```

## Type Safety

### Leverage Type Inference

GPUI-RSX generates type-safe code. Let Rust's type system help:

```rust
// Compiler will catch type mismatches
<div gap={px(16.0)}>  // ✓ Correct type
    {self.render_child()}
</div>

<div gap={"16px"}>    // ✗ Compiler error - wrong type
    {self.render_child()}
</div>
```

### Return Consistent Types

Ensure all branches return compatible types:

```rust
fn render_content(&self) -> impl IntoElement {
    if self.show_text {
        rsx! { <div>"Text"</div> }
    } else {
        // Both branches return impl IntoElement
        rsx! { <span>"Other"</span> }
    }
}
```

## Error Handling

### Provide Meaningful Error Context

```rust
impl MyView {
    fn load_data(&mut self, cx: &mut ViewContext<Self>) {
        match self.try_load() {
            Ok(data) => {
                self.data = Some(data);
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load: {}", e));
                log::error!("Data load error: {:?}", e);
            }
        }
        cx.notify();
    }
}

// In render:
{if let Some(error) = &self.error {
    rsx! {
        <div class="p-4 bg-red-100 text-red-800 rounded">
            {error.clone()}
        </div>
    }
}}
```

## Testing

### Test Component Logic Separately

```rust
impl MyView {
    // Testable logic in methods
    pub fn increment(&mut self) -> usize {
        self.count += 1;
        self.count
    }

    pub fn reset(&mut self) {
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        let mut view = MyView { count: 0 };
        assert_eq!(view.increment(), 1);
        assert_eq!(view.increment(), 2);
    }

    #[test]
    fn test_reset() {
        let mut view = MyView { count: 5 };
        view.reset();
        assert_eq!(view.count, 0);
    }
}
```

## Documentation

### Document Complex Components

```rust
/// A reusable card component with header and body.
///
/// # Example
/// ```ignore
/// self.render_card("Title", rsx! { <div>"Content"</div> })
/// ```
fn render_card(&self, title: &str, content: impl IntoElement) -> impl IntoElement {
    rsx! {
        <div class="border rounded-lg p-4">
            <h3 class="font-bold mb-2">{title}</h3>
            <div>{content}</div>
        </div>
    }
}
```

### Add Examples to Complex Patterns

```rust
/// Renders a list with alternating background colors.
///
/// # Example
/// ```ignore
/// let items = vec!["A", "B", "C"];
/// self.render_striped_list(&items)
/// ```
fn render_striped_list(&self, items: &[String]) -> impl IntoElement {
    rsx! {
        <ul>
            {for (i, item) in items.iter().enumerate() {
                <li
                    class="p-2"
                    when={(i % 2 == 0, |el| el.bg(rgb(0xf3f4f6)))}
                >
                    {item.as_str()}
                </li>
            }}
        </ul>
    }
}
```

## Common Patterns

### Loading States

```rust
fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
    match &self.state {
        LoadState::Initial => rsx! {
            <button onClick={cx.listener(Self::start_load)}>
                "Load Data"
            </button>
        },
        LoadState::Loading => rsx! {
            <div class="flex items-center gap-2">
                <div class="spinner" />
                "Loading..."
            </div>
        },
        LoadState::Loaded(data) => rsx! {
            <div>{self.render_data(data)}</div>
        },
        LoadState::Error(err) => rsx! {
            <div class="text-red-600">
                "Error: "{err.as_str()}
            </div>
        },
    }
}
```

### Modal Dialogs

```rust
fn render_with_modal(&self) -> impl IntoElement {
    rsx! {
        <div>
            {self.render_main_content()}

            {if self.show_modal {
                rsx! {
                    <div class="fixed inset-0 flex items-center justify-center">
                        <div class="absolute inset-0 bg-black opacity-50" />
                        <div class="relative bg-white rounded-lg p-8">
                            {self.render_modal_content()}
                        </div>
                    </div>
                }
            }}
        </div>
    }
}
```

### Tooltips

```rust
<div
    class="relative"
    onHover={cx.listener(|view, _, cx| {
        view.show_tooltip = true;
        cx.notify();
    })}
>
    "Hover me"

    {if self.show_tooltip {
        rsx! {
            <div class="absolute top-full mt-2 p-2 bg-gray-800 text-white rounded">
                "Tooltip text"
            </div>
        }
    }}
</div>
```

## Migration from Manual GPUI

### Before (Manual GPUI)

```rust
div()
    .flex()
    .flex_col()
    .gap(px(16.0))
    .child("Hello")
    .child(
        button()
            .bg(rgb(0x3b82f6))
            .on_click(handler)
            .child("Click")
    )
```

### After (GPUI-RSX)

```rust
rsx! {
    <div class="flex flex-col gap-4">
        "Hello"
        <button
            class="bg-blue-500"
            onClick={handler}
        >
            "Click"
        </button>
    </div>
}
```

## Anti-Patterns

### Don't Use String Concatenation for Classes

**Don't:**

```rust
// This won't work!
<div class={format!("flex {}", if active { "bg-blue-500" } else { "" })} />
```

**Do:**

```rust
<div
    class="flex"
    when={(active, |el| el.bg(rgb(0x3b82f6)))}
/>
```

### Don't Mix Styles and Logic

Keep business logic separate from rendering:

**Don't:**

```rust
{for item in &items {
    {
        let processed = complex_transformation(item);
        rsx! { <div>{processed}</div> }
    }
}}
```

**Do:**

```rust
{for item in &self.get_processed_items() {
    <div>{item.display()}</div>
}}
```

### Don't Overuse `clone()`

Minimize cloning by using references and `as_str()`:

**Don't:**

```rust
<div>{self.text.clone()}</div>
```

**Do:**

```rust
<div>{self.text.as_str()}</div>
```
