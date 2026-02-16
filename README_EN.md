# GPUI-RSX

English | [简体中文](./README.md)

A Rust procedural macro that provides JSX-like syntax for GPUI, making UI development more concise and intuitive.

## ✨ Features

- 🎨 **HTML-like Syntax** - React JSX-like development experience
- 🚀 **Zero Runtime Overhead** - Expands to native GPUI code at compile time
- 📦 **Lightweight** - Only depends on `syn`, `quote`, `proc-macro2`
- 🔧 **Flexible** - Supports expressions, conditional rendering, component composition
- 💡 **Type Safe** - Full compile-time type checking

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gpui = "0.1"
gpui-rsx = { path = "../gpui-rsx" }
```

## 🚀 Quick Start

### Get Started in 5 Minutes

```rust
use gpui::*;
use gpui_rsx::rsx;

struct CounterView {
    count: i32,
}

impl Render for CounterView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        rsx! {
            <div class="flex flex-col gap-4 p-4">
                <h1>{format!("Count: {}", self.count)}</h1>
                <div class="flex gap-2">
                    <button
                        bg={rgb(0x3b82f6)}
                        text_color={rgb(0xffffff)}
                        px_4
                        py_2
                        rounded_md
                        onClick={cx.listener(|view, _, cx| {
                            view.count += 1;
                            cx.notify();
                        })}
                    >
                        {"Increment"}
                    </button>
                    <button
                        bg={rgb(0xef4444)}
                        text_color={rgb(0xffffff)}
                        px_4
                        py_2
                        rounded_md
                        onClick={cx.listener(|view, _, cx| {
                            view.count -= 1;
                            cx.notify();
                        })}
                    >
                        {"Decrement"}
                    </button>
                </div>
            </div>
        }
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(|_cx| CounterView { count: 0 })
        });
    });
}
```

### Before & After

#### ❌ Traditional GPUI (Verbose)

```rust
fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        .child(
            div()
                .text_xl()
                .font_bold()
                .child(format!("Count: {}", self.count))
        )
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    div()
                        .bg(rgb(0x3b82f6))
                        .text_color(rgb(0xffffff))
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .on_click(cx.listener(|view, _, cx| {
                            view.count += 1;
                            cx.notify();
                        }))
                        .child("Increment")
                )
                .child(
                    div()
                        .bg(rgb(0xef4444))
                        .text_color(rgb(0xffffff))
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .on_click(cx.listener(|view, _, cx| {
                            view.count -= 1;
                            cx.notify();
                        }))
                        .child("Decrement")
                )
        )
}
```

#### ✅ With GPUI-RSX (Concise)

See the Quick Start example above.

**Code Reduction: ~50%** ✨

## 📖 Syntax Guide

### 1. Basic Elements

```rust
rsx! {
    <div>{"Hello GPUI"}</div>
}
```

Expands to:
```rust
div().child("Hello GPUI")
```

### 2. Attributes

#### Boolean Attributes (Flags)

```rust
rsx! {
    <div flex flex_col />
}
```

Expands to:
```rust
div().flex().flex_col()
```

#### Value Attributes

```rust
rsx! {
    <div gap={px(16.0)} bg={rgb(0xffffff)} />
}
```

Expands to:
```rust
div().gap(px(16.0)).bg(rgb(0xffffff))
```

### 3. Class Attribute (Special Handling)

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4" />
}
```

Expands to:
```rust
div().flex().flex_col().gap(px(4.0)).p(px(4.0))
```

Supported class patterns:
- `gap-4` → `gap(px(4.0))`
- `p-4` → `p(px(4.0))`
- `px-4` → `px(px(4.0))`
- `py-4` → `py(px(4.0))`
- `flex` → `flex()`
- `flex-col` → `flex_col()`

### 4. Event Handling

```rust
rsx! {
    <button onClick={cx.listener(|view, _, cx| {
        println!("clicked");
    })}>
        {"Click me"}
    </button>
}
```

Supported events:
- `onClick` / `on_click`
- `onMouseDown` / `on_mouse_down`
- `onMouseUp` / `on_mouse_up`
- `onKeyDown` / `on_key_down`
- `onKeyUp` / `on_key_up`

### 5. Nested Elements

```rust
rsx! {
    <div>
        <h1>{"Title"}</h1>
        <p>{"Description"}</p>
        <div>
            <button>{"Action 1"}</button>
            <button>{"Action 2"}</button>
        </div>
    </div>
}
```

### 6. Expressions

```rust
rsx! {
    <div>
        {format!("Count: {}", self.count)}
        {self.render_child_component()}
        {if self.show {
            rsx! { <span>{"Visible"}</span> }
        } else {
            rsx! { <span>{"Hidden"}</span> }
        }}
    </div>
}
```

### 7. List Rendering

```rust
rsx! {
    <div>
        {self.items.iter().map(|item| {
            rsx! {
                <div key={item.id}>
                    {item.name.clone()}
                </div>
            }
        }).collect::<Vec<_>>()}
    </div>
}
```

## 🎯 Complete Example

### Todo App

```rust
use gpui::*;
use gpui_rsx::rsx;

struct TodoApp {
    todos: Vec<Todo>,
    input: String,
}

struct Todo {
    id: usize,
    text: String,
    completed: bool,
}

impl Render for TodoApp {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        rsx! {
            <div class="flex flex-col gap-4 p-4">
                <h1 class="text-2xl font-bold">
                    {"Todo List"}
                </h1>

                <div class="flex gap-2">
                    <input
                        placeholder="Add a todo..."
                        value={self.input.clone()}
                    />
                    <button
                        bg={rgb(0x3b82f6)}
                        text_color={rgb(0xffffff)}
                        px_4
                        py_2
                        rounded_md
                        onClick={cx.listener(|view, _, cx| {
                            view.add_todo();
                            cx.notify();
                        })}
                    >
                        {"Add"}
                    </button>
                </div>

                <div class="flex flex-col gap-2">
                    {self.todos.iter().map(|todo| {
                        rsx! {
                            <div
                                key={todo.id}
                                class="flex gap-2 items-center p-2 rounded-md"
                                bg={if todo.completed {
                                    rgb(0xf3f4f6)
                                } else {
                                    rgb(0xffffff)
                                }}
                            >
                                <input
                                    type="checkbox"
                                    checked={todo.completed}
                                />
                                <span>
                                    {todo.text.clone()}
                                </span>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        }
    }
}

impl TodoApp {
    fn add_todo(&mut self) {
        if !self.input.is_empty() {
            self.todos.push(Todo {
                id: self.todos.len(),
                text: self.input.clone(),
                completed: false,
            });
            self.input.clear();
        }
    }
}
```

## 🔧 Advanced Usage

### Custom Components

```rust
fn render_card(&self, title: &str, content: &str) -> impl IntoElement {
    rsx! {
        <div class="bg-white rounded-lg shadow-md p-6">
            <h2 class="text-xl font-bold">
                {title}
            </h2>
            <p class="text-gray-600">
                {content}
            </p>
        </div>
    }
}

fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
    rsx! {
        <div>
            {self.render_card("Title 1", "Content 1")}
            {self.render_card("Title 2", "Content 2")}
        </div>
    }
}
```

### Conditional Rendering

```rust
rsx! {
    <div>
        {if self.loading {
            rsx! { <div>{"Loading..."}</div> }
        } else if let Some(error) = &self.error {
            rsx! { <div class="text-red-500">{error.clone()}</div> }
        } else {
            rsx! { <div>{self.render_content()}</div> }
        }}
    </div>
}
```

### Dynamic Styling

```rust
fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
    let bg_color = if self.is_active {
        rgb(0x3b82f6)
    } else {
        rgb(0x6b7280)
    };

    rsx! {
        <div bg={bg_color} class="px-4 py-2 rounded-md">
            {"Button"}
        </div>
    }
}
```

## 📊 Performance

GPUI-RSX is a **compile-time macro** that expands to the same code as hand-written GPUI, with **zero runtime overhead**.

| Metric | Traditional GPUI | GPUI-RSX |
|--------|------------------|----------|
| Code Size | 100 lines | 50 lines (-50%) |
| Runtime Performance | Baseline | Same |
| Type Safety | ✅ | ✅ |
| Compile-time Checking | ✅ | ✅ |

## 🛠️ Development

### Build

```bash
cd gpui-rsx
cargo build
```

### Test

```bash
cargo test
```

### Run Examples

```bash
# Counter example
cargo run --example counter

# Todo app example
cargo run --example todo_app
```

### Expand Macros (Debugging)

```bash
# Install cargo-expand
cargo install cargo-expand

# View expanded code
cargo expand --lib
```

## 💡 Best Practices

### 1. Component Splitting

Break complex UIs into small, reusable components:

```rust
// ✅ Recommended: Split into multiple methods
fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
    rsx! {
        <div>
            {self.render_header()}
            {self.render_content()}
            {self.render_footer()}
        </div>
    }
}

fn render_header(&self) -> impl IntoElement {
    rsx! { <header>{"Header"}</header> }
}
```

### 2. Use Constants

Extract repeated styles as constants:

```rust
const PRIMARY_BG: Rgb = rgb(0x3b82f6);
const PRIMARY_TEXT: Rgb = rgb(0xffffff);

rsx! {
    <button bg={PRIMARY_BG} text_color={PRIMARY_TEXT}>
        {"Button"}
    </button>
}
```

### 3. Avoid Over-nesting

```rust
// ❌ Not recommended: Over-nested
rsx! {
    <div>
        <div>
            <div>
                <div>
                    {"Content"}
                </div>
            </div>
        </div>
    </div>
}

// ✅ Recommended: Flatten structure
rsx! {
    <div class="container">
        {"Content"}
    </div>
}
```

## 🐛 FAQ

### Q1: How to use variables in RSX?

```rust
let title = "Hello";
rsx! {
    <div>{title}</div>
}
```

### Q2: How to handle Option types?

```rust
rsx! {
    <div>
        {if let Some(text) = &self.optional_text {
            rsx! { <span>{text.clone()}</span> }
        } else {
            rsx! { <span>{"No text"}</span> }
        }}
    </div>
}
```

### Q3: What does the expanded macro code look like?

Use `cargo expand` to view:

```bash
cargo expand --lib
```

### Q4: Which elements are supported?

All GPUI-supported elements can be used, such as `div`, `button`, `input`, `span`, etc.

## 🤝 Contributing

Contributions are welcome! Feel free to submit Issues or Pull Requests.

### Development Workflow

1. Fork the project
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push branch: `git push origin feature/amazing-feature`
5. Submit a Pull Request

### Code Standards

- Use `rustfmt` to format code
- Use `clippy` to check code quality
- Add tests for new features
- Update documentation

## 📝 License

MIT License

## 🙏 Acknowledgments

Inspired by:
- [Dioxus RSX](https://dioxuslabs.com/) - RSX syntax design
- [Yew html! macro](https://yew.rs/) - html! macro
- [React JSX](https://react.dev/) - JSX syntax
- [GPUI](https://www.gpui.rs/) - Underlying UI framework

---

**Make GPUI development more enjoyable!** 🎉
