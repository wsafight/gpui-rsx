# Migration Guide

Guide for upgrading between versions and migrating from manual GPUI code.

## Migrating from Manual GPUI

### Benefits of GPUI-RSX

- **More concise**: Less boilerplate code
- **Better readability**: HTML-like structure is easier to understand
- **Familiar syntax**: JSX-like for those coming from React
- **Type safe**: Full Rust type checking
- **No runtime overhead**: Compiles to the same GPUI code

### Basic Element Migration

**Before (Manual GPUI):**
```rust
div()
    .flex()
    .flex_col()
    .gap(px(16.0))
    .bg(rgb(0x3b82f6))
```

**After (GPUI-RSX):**
```rust
rsx! {
    <div class="flex flex-col gap-4 bg-blue-500" />
}
```

### Children Migration

**Before (Manual GPUI):**
```rust
div()
    .child("Hello")
    .child(span().child("World"))
    .child(button().child("Click"))
```

**After (GPUI-RSX):**
```rust
rsx! {
    <div>
        "Hello"
        <span>"World"</span>
        <button>"Click"</button>
    </div>
}
```

### Event Handlers

**Before (Manual GPUI):**
```rust
button()
    .on_click(cx.listener(|view, _event, _window, cx| {
        view.count += 1;
        cx.notify();
    }))
    .child("Click")
```

**After (GPUI-RSX):**
```rust
rsx! {
    <button onClick={cx.listener(|view, _event, _window, cx| {
        view.count += 1;
        cx.notify();
    })}>
        "Click"
    </button>
}
```

### Conditional Rendering

**Before (Manual GPUI):**
```rust
let mut element = div();

if show_message {
    element = element.child("Message visible");
}

element
```

**After (GPUI-RSX):**
```rust
rsx! {
    <div>
        {if show_message {
            rsx! { <span>"Message visible"</span> }
        }}
    </div>
}
```

Or use `when` attribute:

```rust
rsx! {
    <div
        when={(show_message, |el| el.child("Message visible"))}
    />
}
```

### Lists

**Before (Manual GPUI):**
```rust
let mut list = div().flex().flex_col();

for item in &self.items {
    list = list.child(
        div()
            .child(item.name.clone())
    );
}

list
```

**After (GPUI-RSX):**
```rust
rsx! {
    <div class="flex flex-col">
        {for item in &self.items {
            <div>{item.name.as_str()}</div>
        }}
    </div>
}
```

### Complex Nested Structures

**Before (Manual GPUI):**
```rust
div()
    .flex()
    .child(
        div()
            .w(px(200.0))
            .bg(rgb(0xf3f4f6))
            .child(
                div()
                    .p(px(16.0))
                    .child("Sidebar")
            )
    )
    .child(
        div()
            .flex_1()
            .child(
                div()
                    .p(px(16.0))
                    .child("Main content")
            )
    )
```

**After (GPUI-RSX):**
```rust
rsx! {
    <div flex>
        <div class="w-48 bg-gray-100">
            <div class="p-4">"Sidebar"</div>
        </div>
        <div flex_1>
            <div class="p-4">"Main content"</div>
        </div>
    </div>
}
```

### Method Chaining

Both styles can coexist:

```rust
rsx! {
    <div class="flex">
        {div()
            .flex_1()
            .bg(self.compute_color())
            .child("Mixed style")}
    </div>
}
```

## Version Migrations

### 0.1.0 → 0.1.1

**Changes:**
- Added `when` and `whenSome` attributes
- Added `styled` flag for semantic HTML tags
- Improved error messages

**Migration steps:**

1. **No breaking changes** - all 0.1.0 code continues to work

2. **Optional: Use new `when` attribute:**

   Before:
   ```rust
   <div>
       {if condition {
           rsx! { <div bg={color} /> }
       } else {
           rsx! { <div /> }
       }}
   </div>
   ```

   After:
   ```rust
   <div when={(condition, |el| el.bg(color))} />
   ```

3. **Optional: Use `styled` flag:**

   Before:
   ```rust
   <h1 class="text-3xl font-bold">"Title"</h1>
   ```

   After:
   ```rust
   <h1 styled>"Title"</h1>
   ```

### 0.1.1 → 0.1.2

**Changes:**
- Improved compile-time performance
- Better type inference
- Enhanced error messages with helpful hints

**Migration steps:**

1. **No breaking changes** - all 0.1.1 code continues to work

2. **Optional: Update Cargo.toml:**
   ```toml
   [dependencies]
   gpui-rsx = "0.4"
   ```

3. **Optional: Simplify code with better type inference:**

   The macro now generates more efficient code automatically - no changes needed.

## Best Practices for Migration

### Incremental Migration

Don't migrate everything at once. Start with:

1. **New components** - Use RSX for all new code
2. **Simple components** - Migrate straightforward components first
3. **Complex components** - Migrate as you gain confidence

### Hybrid Approach

You can mix RSX and manual GPUI:

```rust
impl Render for MyView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div>
                {self.legacy_method()}  // Manual GPUI
                <div>"New RSX content"</div>
            </div>
        }
    }
}

fn legacy_method(&self) -> impl IntoElement {
    div().child("Legacy code")
}
```

### Testing Migration

1. **Visual testing**: Ensure UI looks the same
2. **Behavior testing**: Verify interactions work
3. **Performance testing**: Check for regressions

### Common Migration Patterns

#### Pattern 1: Simple Replacement

Before:
```rust
div().child("Hello")
```

After:
```rust
rsx! { <div>"Hello"</div> }
```

#### Pattern 2: Variable Interpolation

Before:
```rust
div().child(format!("Count: {}", count))
```

After:
```rust
rsx! { <div>{format!("Count: {}", count)}</div> }
```

#### Pattern 3: Builder Pattern to Attributes

Before:
```rust
div()
    .w(px(200.0))
    .h(px(100.0))
    .bg(rgb(0xff0000))
```

After:
```rust
rsx! {
    <div
        w={px(200.0)}
        h={px(100.0)}
        bg={rgb(0xff0000)}
    />
}
```

#### Pattern 4: Dynamic Children

Before:
```rust
let mut container = div();
for item in items {
    container = container.child(div().child(item));
}
container
```

After:
```rust
rsx! {
    <div>
        {for item in items {
            <div>{item}</div>
        }}
    </div>
}
```

## Troubleshooting Migration

### Type Mismatches

If you get type errors after migration:

```rust
// Add explicit type annotations
let items: &[Item] = &self.items;

rsx! {
    <div>
        {for item in items {
            <div>{item.render()}</div>
        }}
    </div>
}
```

### Performance Regressions

If performance degrades:

1. **Use references instead of clones:**
   ```rust
   {for item in &items {
       <div>{item.name.as_str()}</div>
   }}
   ```

2. **Extract static content:**
   ```rust
   fn static_header() -> impl IntoElement {
       rsx! { <header>"Static"</header> }
   }
   ```

3. **Batch state updates:**
   ```rust
   self.field1 = value1;
   self.field2 = value2;
   cx.notify(); // Single update
   ```

### Compile Time Increases

If compile times increase significantly:

1. **Split large RSX blocks** into smaller methods
2. **Use incremental compilation:** `cargo build` for development
3. **Consider caching:** Results are cached between builds

## GPUI Version Compatibility

GPUI-RSX aims to stay compatible with the latest stable GPUI release.

### Checking Compatibility

```bash
cargo tree | grep gpui
```

### Updating Dependencies

```bash
cargo update
cargo build
```

### Version Matrix

| GPUI-RSX Version | Compatible GPUI Versions |
|------------------|-------------------------|
| 0.1.0 - 0.1.2   | 0.1.x                  |
| Future releases | Will maintain compatibility |

## Getting Help

If you encounter issues during migration:

1. **Review documentation:**
   - [Getting Started](getting-started.md)
   - [API Reference](api-reference.md)
   - [Best Practices](best-practices.md)

3. **Ask for help:**
   - [GitHub Issues](https://github.com/wsafight/gpui-rsx/issues)
   - Include: version numbers, minimal code example, error messages

## Migration Checklist

- [ ] Updated `Cargo.toml` with `gpui-rsx` dependency
- [ ] Imported `rsx` macro: `use gpui_rsx::rsx;`
- [ ] Tested simple component migration
- [ ] Migrated event handlers
- [ ] Migrated conditional rendering
- [ ] Migrated list rendering
- [ ] Tested UI appearance
- [ ] Tested interactions
- [ ] Verified performance
- [ ] Updated tests
- [ ] Updated documentation
