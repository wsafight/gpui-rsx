# GPUI-RSX

[English](./README.md) | 简体中文

一个为 GPUI 提供 JSX-like 语法的 Rust 过程宏，让 UI 开发更加简洁和直观。

## ✨ 特性

- 🎨 **HTML-like 语法** - 类似 React JSX 的开发体验
- 🚀 **零运行时开销** - 编译时展开为原生 GPUI 代码
- 📦 **轻量级** - 仅依赖 `syn`, `quote`, `proc-macro2`
- 🔧 **灵活** - 支持表达式、条件渲染、组件组合
- 💡 **类型安全** - 完全的编译时检查

## 📦 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
gpui = "0.1"
gpui-rsx = "0.1"
```

## 🚀 快速开始

### 5 分钟上手

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

### 前后对比

#### ❌ 传统 GPUI 写法（繁琐）

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

#### ✅ 使用 GPUI-RSX（简洁）

见上方快速开始示例。

**代码减少：~50%** ✨

## 📖 语法指南

### 1. 基本元素

```rust
rsx! {
    <div>{"Hello GPUI"}</div>
}
```

展开为：
```rust
div().child("Hello GPUI")
```

### 2. 属性

#### 布尔属性（Flag）

```rust
rsx! {
    <div flex flex_col />
}
```

展开为：
```rust
div().flex().flex_col()
```

#### 值属性

```rust
rsx! {
    <div gap={px(16.0)} bg={rgb(0xffffff)} />
}
```

展开为：
```rust
div().gap(px(16.0)).bg(rgb(0xffffff))
```

### 3. Class 属性（特殊处理）

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4" />
}
```

展开为：
```rust
div().flex().flex_col().gap(px(4.0)).p(px(4.0))
```

支持的 class 模式：
- `gap-4` → `gap(px(4.0))`
- `p-4` → `p(px(4.0))`
- `px-4` → `px(px(4.0))`
- `py-4` → `py(px(4.0))`
- `flex` → `flex()`
- `flex-col` → `flex_col()`

### 4. 事件处理

```rust
rsx! {
    <button onClick={cx.listener(|view, _, cx| {
        println!("clicked");
    })}>
        {"Click me"}
    </button>
}
```

支持的事件：
- `onClick` / `on_click`
- `onMouseDown` / `on_mouse_down`
- `onMouseUp` / `on_mouse_up`
- `onKeyDown` / `on_key_down`
- `onKeyUp` / `on_key_up`

### 5. 嵌套元素

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

### 6. 表达式

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

### 7. 列表渲染

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

## 🎯 完整示例

### Todo 应用

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

## 🔧 高级用法

### 自定义组件

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

### 条件渲染

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

### 动态样式

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

## 📊 性能

GPUI-RSX 是一个**编译时宏**，展开后的代码与手写的 GPUI 代码完全相同，**零运行时开销**。

| 指标 | 传统 GPUI | GPUI-RSX |
|------|----------|----------|
| 代码量 | 100 行 | 50 行 (-50%) |
| 运行时性能 | 基准 | 相同 |
| 类型安全 | ✅ | ✅ |
| 编译时检查 | ✅ | ✅ |

## 🛠️ 开发

### 构建

```bash
cd gpui-rsx
cargo build
```

### 测试

```bash
cargo test
```

### 运行示例

```bash
# 计数器示例
cargo run --example counter

# Todo 应用示例
cargo run --example todo_app
```

### 展开宏（调试）

```bash
# 安装 cargo-expand
cargo install cargo-expand

# 查看展开后的代码
cargo expand --lib
```

## 💡 最佳实践

### 1. 组件拆分

将复杂的 UI 拆分为小的、可复用的组件：

```rust
// ✅ 推荐：拆分为多个方法
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

### 2. 使用常量

将重复的样式提取为常量：

```rust
const PRIMARY_BG: Rgb = rgb(0x3b82f6);
const PRIMARY_TEXT: Rgb = rgb(0xffffff);

rsx! {
    <button bg={PRIMARY_BG} text_color={PRIMARY_TEXT}>
        {"Button"}
    </button>
}
```

### 3. 避免过度嵌套

```rust
// ❌ 不推荐：过度嵌套
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

// ✅ 推荐：扁平化结构
rsx! {
    <div class="container">
        {"Content"}
    </div>
}
```

## 🐛 常见问题

### Q1: 如何在 RSX 中使用变量？

```rust
let title = "Hello";
rsx! {
    <div>{title}</div>
}
```

### Q2: 如何处理 Option 类型？

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

### Q3: 宏展开后的代码是什么样的？

使用 `cargo expand` 查看：

```bash
cargo expand --lib
```

### Q4: 支持哪些元素？

所有 GPUI 支持的元素都可以使用，如 `div`, `button`, `input`, `span` 等。

## 🤝 贡献

欢迎贡献！请随时提交 Issue 或 Pull Request。

### 开发流程

1. Fork 项目
2. 创建功能分支: `git checkout -b feature/amazing-feature`
3. 提交更改: `git commit -m 'Add amazing feature'`
4. 推送分支: `git push origin feature/amazing-feature`
5. 提交 Pull Request

### 代码规范

- 使用 `rustfmt` 格式化代码
- 使用 `clippy` 检查代码质量
- 为新功能添加测试
- 更新文档

## 📝 许可

MIT License

## 🙏 致谢

灵感来源于：
- [Dioxus RSX](https://dioxuslabs.com/) - RSX 语法设计
- [Yew html! macro](https://yew.rs/) - html! 宏
- [React JSX](https://react.dev/) - JSX 语法
- [GPUI](https://www.gpui.rs/) - 底层 UI 框架

---

**让 GPUI 开发更加愉快！** 🎉
