# Troubleshooting

Common issues and solutions when using GPUI-RSX.

## Compilation Errors

### Error: "closing tag does not match opening tag"

**Error message:**
```
Closing tag `</span>` does not match opening tag `<div>`. Tags must be properly nested.
```

**Solution:**
Ensure all tags are properly matched:

```rust
// ✗ Wrong
<div>
    <span>"Text"</div>
</span>

// ✓ Correct
<div>
    <span>"Text"</span>
</div>
```

### Error: "unclosed tag"

**Error message:**
```
Unclosed tag `<div>`. Expected closing tag before end of input.
```

**Solution:**
Add the missing closing tag:

```rust
// ✗ Wrong
rsx! {
    <div>"Content"
}

// ✓ Correct
rsx! {
    <div>"Content"</div>
}
```

### Error: "unexpected token"

**Error message:**
```
Unexpected token in `<div>`. Expected one of: {expr}, "text", <child>, or </div>
```

**Solution:**
Wrap bare identifiers in braces:

```rust
// ✗ Wrong
<div>count</div>

// ✓ Correct
<div>{count}</div>
```

### Error: "class attribute only supports string literals"

**Error message:**
```
class attribute only supports string literals; use individual attributes for dynamic styling
```

**Solution:**
Use individual attributes for dynamic values:

```rust
// ✗ Wrong
<div class={format!("flex {}", self.extra_classes)} />

// ✓ Correct
<div
    flex
    bg={self.get_background()}
/>
```

### Error: "expected '{' after for-in expression"

**Error message:**
```
Expected '{' after for-in expression to start the loop body.
```

**Solution:**
Add braces around the for-loop body:

```rust
// ✗ Wrong
{for item in items
    <div>{item}</div>
}

// ✓ Correct
{for item in items {
    <div>{item}</div>
}}
```

## Type Errors

### Error: "trait `IntoElement` is not implemented"

**Error:**
```
the trait `IntoElement` is not implemented for `Option<String>`
```

**Solution:**
Unwrap or handle the Option:

```rust
// ✗ Wrong
<div>{self.optional_text}</div>

// ✓ Correct - unwrap with default
<div>{self.optional_text.as_deref().unwrap_or("")}</div>

// ✓ Correct - conditional rendering
{if let Some(text) = &self.optional_text {
    rsx! { <div>{text.as_str()}</div> }
}}
```

### Error: "mismatched types" in conditional

**Error:**
```
expected `Div`, found `Span`
```

**Solution:**
Both branches must return the same type. Use a common container:

```rust
// ✗ Wrong
fn render_content(&self) -> impl IntoElement {
    if self.use_div {
        rsx! { <div>"Content"</div> }
    } else {
        rsx! { <span>"Content"</span> }
    }
}

// ✓ Correct - wrap in common type
fn render_content(&self) -> impl IntoElement {
    rsx! {
        <div>
            {if self.use_bold {
                rsx! { <strong>"Content"</strong> }
            } else {
                rsx! { <span>"Content"</span> }
            }}
        </div>
    }
}
```

### Error: "cannot move out of `self`"

**Error:**
```
cannot move out of `self.items` which is behind a shared reference
```

**Solution:**
Use references in loops:

```rust
// ✗ Wrong
{for item in self.items {
    <div>{item.name}</div>
}}

// ✓ Correct
{for item in &self.items {
    <div>{item.name.as_str()}</div>
}}
```

## Runtime Issues

### Elements not appearing

**Symptoms:**
- Elements don't show up in the UI
- Layout is empty

**Possible causes and solutions:**

1. **Forgot to return from render:**
   ```rust
   // ✗ Wrong
   fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
       rsx! { <div>"Content"</div> };  // Note the semicolon
   }

   // ✓ Correct
   fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
       rsx! { <div>"Content"</div> }  // No semicolon
   }
   ```

2. **Missing `cx.notify()` after state changes:**
   ```rust
   // ✗ Wrong
   fn handle_click(&mut self, cx: &mut Context<Self>) {
       self.count += 1;
       // Missing cx.notify()
   }

   // ✓ Correct
   fn handle_click(&mut self, cx: &mut Context<Self>) {
       self.count += 1;
       cx.notify();  // Trigger re-render
   }
   ```

3. **Size constraints:**
   ```rust
   // Element might be zero-sized
   <div />

   // Add explicit size
   <div class="w-full h-full" />
   ```

### Events not firing

**Symptoms:**
- Clicks or other events don't trigger handlers

**Possible causes and solutions:**

1. **Element needs an ID for interactive events:**

   RSX automatically adds IDs for elements with event handlers, but if you're getting issues:

   ```rust
   // Add explicit ID if needed
   <button id="my-button" onClick={handler}>
       "Click"
   </button>
   ```

2. **Element is covered by another element:**

   Check z-index and layering:
   ```rust
   <button class="relative z-10" onClick={handler}>
       "Click"
   </button>
   ```

3. **Handler signature is incorrect:**
   ```rust
   // ✓ Correct
   onClick={cx.listener(|view, _event, _window, cx| {
       view.handle_click(cx);
   })}
   ```

### Styling not applied

**Symptoms:**
- Colors, sizes, or other styles don't appear

**Possible causes and solutions:**

1. **Typo in class name:**
   ```rust
   // ✗ Wrong
   <div class="flex-collumn" />  // Typo

   // ✓ Correct
   <div class="flex-col" />
   ```

2. **Invalid color name:**
   ```rust
   // ✗ Wrong - invalid shade
   <div class="bg-blue-550" />

   // ✓ Correct - valid shade
   <div class="bg-blue-500" />
   ```

3. **Style override order:**
   ```rust
   // Later attributes override earlier ones
   <div
       bg={rgb(0xff0000)}  // Red
       class="bg-blue-500"  // Blue overrides red
   />
   ```

### Performance issues

**Symptoms:**
- Slow rendering
- High CPU usage
- Laggy interactions

**Solutions:**

1. **Minimize cloning in loops:**
   ```rust
   // ✗ Slow
   {for item in &self.items {
       <div>{item.description.clone()}</div>
   }}

   // ✓ Faster
   {for item in &self.items {
       <div>{item.description.as_str()}</div>
   }}
   ```

2. **Extract static elements:**
   ```rust
   // ✗ Re-creates header on every render
   rsx! {
       <div>
           <header class="flex p-4">"Static Header"</header>
           {self.dynamic_content()}
       </div>
   }

   // ✓ Extract to separate method
   fn render_static_header() -> impl IntoElement {
       rsx! { <header class="flex p-4">"Static Header"</header> }
   }
   ```

3. **Batch state updates:**
   ```rust
   // ✗ Multiple re-renders
   self.value1 = new_value1;
   cx.notify();
   self.value2 = new_value2;
   cx.notify();

   // ✓ Single re-render
   self.value1 = new_value1;
   self.value2 = new_value2;
   cx.notify();
   ```

## IDE Issues

### No syntax highlighting

**Solution:**
RSX syntax is valid Rust syntax, so standard Rust syntax highlighting should work. If you're experiencing issues:

1. **Restart your IDE/language server**
2. **Ensure rust-analyzer is up to date:**
   ```bash
   rustup update
   ```

### No autocomplete in RSX

**Solution:**
Autocomplete inside `rsx!` macros is limited. For better IDE support:

1. **Extract complex logic to methods**
2. **Use explicit types:**
   ```rust
   let items: &[Item] = &self.items;
   rsx! {
       <div>
           {for item in items {
               <div>{item.name.as_str()}</div>
           }}
       </div>
   }
   ```

### Macro expansion errors

**Solution:**
View expanded macro output:

```bash
cargo expand --lib
```

Or for a specific module:

```bash
cargo expand --lib module_name
```

## Build Issues

### Long compile times

**Symptoms:**
- Slow incremental builds
- High CPU usage during compilation

**Solutions:**

1. **Use release mode for final builds only:**
   ```bash
   cargo build  # Fast debug builds for development
   ```

2. **Split large RSX blocks:**
   ```rust
   // ✗ One huge RSX block
   rsx! {
       <div>
           // 500 lines of RSX
       </div>
   }

   // ✓ Split into methods
   fn render(&self) -> impl IntoElement {
       rsx! {
           <div>
               {self.render_header()}
               {self.render_body()}
               {self.render_footer()}
           </div>
       }
   }
   ```

3. **Update dependencies:**
   ```bash
   cargo update
   ```

### Dependency conflicts

**Error:**
```
failed to select a version for `syn`
```

**Solution:**
Ensure compatible versions:

```toml
[dependencies]
gpui-rsx = "0.4"
syn = { version = "2.0", features = ["full"] }
```

## Getting Help

If you're still stuck:

1. **Enable debug logging:**
   ```rust
   env_logger::init();
   log::debug!("State: {:?}", self);
   ```

3. **Simplify to minimal reproduction:**
   ```rust
   // Remove features until it works
   // Then add them back one by one
   ```

4. **Search existing issues:**
   - [GPUI-RSX Issues](https://github.com/wsafight/gpui-rsx/issues)
   - [GPUI Issues](https://github.com/zed-industries/gpui/issues)

5. **Create a new issue:**
   Include:
   - Minimal code example
   - Error messages
   - Expected vs actual behavior
   - Rust version (`rustc --version`)
   - GPUI version
   - GPUI-RSX version

## Common Pitfalls

### Semicolon after rsx! macro

```rust
// ✗ Returns ()
rsx! { <div>"Content"</div> };

// ✓ Returns impl IntoElement
rsx! { <div>"Content"</div> }
```

### Forgetting braces around expressions

```rust
// ✗ Syntax error
<div>self.count</div>

// ✓ Correct
<div>{self.count}</div>
```

### Using `clone()` excessively

```rust
// ✗ Expensive
{for item in &self.items {
    <div>{item.name.clone()}</div>
}}

// ✓ Cheaper
{for item in &self.items {
    <div>{item.name.as_str()}</div>
}}
```

### Missing return type bounds

```rust
// ✗ May cause inference issues
fn render_item(&self, item: &Item) {
    rsx! { <div>{item.name.as_str()}</div> }
}

// ✓ Explicit return type
fn render_item(&self, item: &Item) -> impl IntoElement {
    rsx! { <div>{item.name.as_str()}</div> }
}
```
