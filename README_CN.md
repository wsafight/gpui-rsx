# GPUI-RSX

[English](./README.md) | 简体中文

[![CI](https://github.com/wsafight/gpui-rsx/actions/workflows/ci.yml/badge.svg)](https://github.com/wsafight/gpui-rsx/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/wsafight/gpui-rsx/branch/main/graph/badge.svg)](https://codecov.io/gh/wsafight/gpui-rsx)
[![Crates.io](https://img.shields.io/crates/v/gpui-rsx.svg)](https://crates.io/crates/gpui-rsx)
[![Documentation](https://docs.rs/gpui-rsx/badge.svg)](https://docs.rs/gpui-rsx)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

一个为 GPUI 提供 JSX-like 语法的 Rust 过程宏，让 UI 开发更加简洁和直观。

## ✨ 特性

- 🎨 **HTML-like 语法** - 类似 React JSX 的开发体验
- 🚀 **零运行时开销** - 编译时展开为原生 GPUI 代码
- 📦 **轻量级** - 仅依赖 `syn`, `quote`, `proc-macro2`
- 🔧 **灵活** - 支持表达式、条件渲染、组件组合
- 💡 **类型安全** - 完全的编译时检查
- 🧩 **Fragment 支持** - 使用 `<>...</>` 返回多个根元素
- 🔁 **For 循环语法糖** - 使用 `{for item in iter { ... }}` 迭代
- 🎨 **完整 Tailwind 色板** - 内置 242 种颜色 + 任意 hex 值

## 📚 文档资源

- **[架构指南](./ARCHITECTURE_CN.md)** - 详细的架构文档
  - 模块组织和数据流
  - 代码生成策略
  - 设计模式和测试方法
  - 扩展点和调试指南
- **[快速入门](./docs/getting-started.md)** - 分步教程
- **[API 参考](./docs/api-reference.md)** - 完整 API 文档
- **[最佳实践](./docs/best-practices.md)** - 推荐模式
- **[迁移指南](./docs/migration-guide.md)** - 升级说明
- **[问题排查](./docs/troubleshooting.md)** - 常见问题和解决方案

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

### 2. Fragment（多根节点）

当你需要返回多个元素而不需要外层包装时：

```rust
rsx! {
    <>
        <div>{"第一个"}</div>
        <div>{"第二个"}</div>
        <div>{"第三个"}</div>
    </>
}
```

展开为：
```rust
vec![
    div().child("第一个"),
    div().child("第二个"),
    div().child("第三个"),
]
```

### 3. 属性

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

### 4. Class 属性

`class` 属性接受类似 Tailwind 的字符串，展开为多个 GPUI 方法调用：

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4" />
}
```

展开为：
```rust
div().flex().flex_col().gap(px(4.0)).p(px(4.0))
```

> **注意：** `class` 仅支持字符串字面量。动态样式请使用独立属性（如 `bg={color_var}`）。

#### 支持的 class 模式

**布局：**
- `flex`, `flex-col`, `flex-row`, `flex-wrap`, `flex-1`, `flex-none`, `flex-auto`
- `items-center`, `items-start`, `items-end`
- `justify-center`, `justify-between`

**间距**（数值自动转为 `px(n)`）：
- `gap-4` → `.gap(px(4.0))`
- `p-4`, `px-4`, `py-4`, `pt-4`, `pb-4`, `pl-4`, `pr-4`
- `m-4`, `mx-4`, `my-4`, `mt-4`, `mb-4`, `ml-4`, `mr-4`
- `w-64`, `h-32`
- 小数值：`p-0.5` → `.p(px(0.5))`

**尺寸：**
- `w-full`, `h-full`, `size-full`

**文本：**
- `text-xs`, `text-sm`, `text-base`, `text-lg`, `text-xl`
- `text-2xl`, `text-3xl`, `text-4xl`, `text-5xl`
- `font-bold`

**边框：**
- `border` → `.border_1()`
- `border-2` → `.border_2()`, `border-4` → `.border_4()`
- `rounded-sm`, `rounded-md`, `rounded-lg`, `rounded-xl`, `rounded-full`, `rounded-none`

**颜色**（完整 Tailwind 色板）：
- `text-red-500` → `.text_color(rgb(0xef4444))`
- `bg-blue-600` → `.bg(rgb(0x2563eb))`
- `border-green-500` → `.border_color(rgb(0x22c55e))`
- 任意 hex：`bg-[#ff0000]`, `text-[#333]`, `border-[#abc]`

**效果：**
- `shadow-sm`, `shadow-md`, `shadow-lg`
- `overflow-hidden`, `overflow-scroll`
- `cursor-pointer`, `cursor-default`, `cursor-text`

**支持的颜色：** slate, gray, zinc, neutral, stone, red, orange, amber, yellow, lime, green, emerald, teal, cyan, sky, blue, indigo, violet, purple, fuchsia, pink, rose（色阶 50-950）+ white, black

### 5. 事件处理

```rust
rsx! {
    <button onClick={cx.listener(|view, _, cx| {
        println!("clicked");
    })}>
        {"Click me"}
    </button>
}
```

支持的事件（camelCase / snake_case）：

| 事件 | 方法 |
|------|------|
| `onClick` / `on_click` | `.on_click(handler)` |
| `onMouseDown` / `on_mouse_down` | `.on_mouse_down(handler)` |
| `onMouseUp` / `on_mouse_up` | `.on_mouse_up(handler)` |
| `onMouseMove` / `on_mouse_move` | `.on_mouse_move(handler)` |
| `onMouseDownOut` / `on_mouse_down_out` | `.on_mouse_down_out(handler)` |
| `onMouseUpOut` / `on_mouse_up_out` | `.on_mouse_up_out(handler)` |
| `onKeyDown` / `on_key_down` | `.on_key_down(handler)` |
| `onKeyUp` / `on_key_up` | `.on_key_up(handler)` |
| `onFocus` / `on_focus` | `.on_focus(handler)` |
| `onBlur` / `on_blur` | `.on_blur(handler)` |
| `onHover` / `on_hover` | `.on_hover(handler)` |
| `onScrollWheel` / `on_scroll_wheel` | `.on_scroll_wheel(handler)` |
| `onDrag` / `on_drag` | `.on_drag(handler)` |
| `onDrop` / `on_drop` | `.on_drop(handler)` |
| `onAction` / `on_action` | `.on_action(handler)` |

### 6. 嵌套元素

```rust
rsx! {
    <div>
        <h1>{"标题"}</h1>
        <p>{"描述"}</p>
        <div>
            <button>{"操作 1"}</button>
            <button>{"操作 2"}</button>
        </div>
    </div>
}
```

### 7. 表达式

```rust
rsx! {
    <div>
        {format!("Count: {}", self.count)}
        {self.render_child_component()}
        {if self.show {
            rsx! { <span>{"可见"}</span> }
        } else {
            rsx! { <span>{"隐藏"}</span> }
        }}
    </div>
}
```

### 8. 列表渲染

#### 使用迭代器（传统方式）

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

#### 使用 for 循环语法糖

```rust
rsx! {
    <ul>
        {for item in &self.items {
            <li>{item.name.clone()}</li>
        }}
    </ul>
}
```

展开为：
```rust
div().children((&self.items).into_iter().map(|item| {
    div().child(item.name.clone())
}))
```

for 循环也支持 range 和方法调用：

```rust
rsx! {
    <div>
        {for i in 0..5 {
            <span>{i}</span>
        }}
    </div>
}
```

### 9. 展开语法

```rust
rsx! {
    <div>
        {...items.iter().map(|item| rsx! { <span>{item}</span> })}
    </div>
}
```

### 10. 属性映射参考

camelCase 属性会自动映射为 GPUI 的 snake_case 方法：

| RSX 属性 | GPUI 方法 |
|----------|-----------|
| `zIndex` | `.z_index()` |
| `opacity` | `.opacity()` |
| `visible` | `.visible()` |
| `invisible`（标志） | `.visible(false)` |
| `width` / `height` | `.w()` / `.h()` |
| `minWidth` / `maxWidth` | `.min_w()` / `.max_w()` |
| `minHeight` / `maxHeight` | `.min_h()` / `.max_h()` |
| `gapX` / `gapY` | `.gap_x()` / `.gap_y()` |
| `flexBasis` | `.basis()` |
| `flexGrow` / `flexShrink` | `.flex_grow()` / `.flex_shrink()` |
| `flexOrder` | `.order()` |
| `fontSize` | `.font_size()` |
| `lineHeight` | `.line_height()` |
| `fontWeight` | `.font_weight()` |
| `textAlign` | `.text_align()` |
| `textDecoration` | `.text_decoration()` |
| `borderRadius` | `.border_radius()` |
| `borderTop` / `borderBottom` | `.border_t()` / `.border_b()` |
| `borderLeft` / `borderRight` | `.border_l()` / `.border_r()` |
| `roundedTop` / `roundedBottom` | `.rounded_t()` / `.rounded_b()` |
| `roundedTopLeft` / `roundedTopRight` | `.rounded_tl()` / `.rounded_tr()` |
| `roundedBottomLeft` / `roundedBottomRight` | `.rounded_bl()` / `.rounded_br()` |
| `boxShadow` | `.shadow()` |
| `overflowX` / `overflowY` | `.overflow_x_hidden()` / `.overflow_y_hidden()` |
| `inset` | `.inset()` |

不在此表中的属性将原样透传（如 `bg={color}` → `.bg(color)`）。

### 11. 使用 `when` 和 `whenSome` 进行条件样式

#### when - 根据条件应用样式

```rust
rsx! {
    <div
        flex
        when={(is_active, |this| {
            this.bg(rgb(0x3b82f6))
                .text_color(rgb(0xffffff))
        })}
    >
        {"按钮"}
    </div>
}
```

#### whenSome - 当 Option 有值时应用样式

```rust
let custom_width: Option<f32> = Some(200.0);

rsx! {
    <div
        flex
        whenSome={(custom_width, |this, w| this.w(px(w)))}
    >
        {"内容"}
    </div>
}
```

#### 多个条件

```rust
rsx! {
    <button
        class="px-4 py-2 rounded-md"
        when={(is_selected, |this| this.bg(rgb(0x3b82f6)))}
        when={(is_disabled, |this| this.bg(rgb(0xe5e7eb)))}
        whenSome={(custom_color, |this, color| this.bg(rgb(color)))}
    >
        {"按钮"}
    </button>
}
```

### 12. styled 标志（默认标签样式）

`styled` 标志会根据标签名注入合理的默认样式：

```rust
rsx! {
    <h1 styled>{"标题"}</h1>
    // 展开为: div().text_3xl().font_bold().child("标题")

    <button styled>{"点击"}</button>
    // 展开为: div().cursor_pointer().child("点击")
}
```

各标签默认样式：

| 标签 | 默认样式 |
|------|---------|
| `h1` | `text-3xl font-bold` |
| `h2` | `text-2xl font-bold` |
| `h3` | `text-xl font-bold` |
| `h4` | `text-lg font-bold` |
| `h5` | `text-base font-bold` |
| `h6` | `text-sm font-bold` |
| `button`, `a` | `cursor-pointer` |
| `input`, `textarea` | `px-2 py-1` |
| `ul`, `ol` | `flex flex-col` |

用户属性在默认样式之后应用，可以覆盖。

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
                        class="bg-blue-500 text-white px-4 py-2 rounded-md"
                        onClick={cx.listener(|view, _, cx| {
                            view.add_todo();
                            cx.notify();
                        })}
                    >
                        {"Add"}
                    </button>
                </div>

                <div class="flex flex-col gap-2">
                    {for todo in self.todos.iter() {
                        <div
                            class="flex gap-2 items-center p-2 rounded-md"
                            bg={if todo.completed {
                                rgb(0xf3f4f6)
                            } else {
                                rgb(0xffffff)
                            }}
                        >
                            <span>{todo.text.clone()}</span>
                        </div>
                    }}
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
        <div class="rounded-lg shadow-md p-6">
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
cargo test --test macro_tests
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

### Q5: 可以使用动态 class 值吗？

不可以。`class` 属性仅支持字符串字面量。动态样式请使用独立属性：

```rust
// ❌ 不可用
rsx! { <div class={my_class} /> }

// ✅ 使用独立属性
rsx! {
    <div bg={dynamic_color} flex />
}

// ✅ 或使用 `when` 进行条件样式
rsx! {
    <div when={(is_active, |this| this.bg(rgb(0x3b82f6)))} />
}
```

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
