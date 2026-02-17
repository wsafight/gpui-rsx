# GPUI-RSX 架构

## 概述

GPUI-RSX 是一个为 GPUI UI 框架提供类 JSX 语法的过程宏。它在编译时将类 HTML 的标记转换为惯用的 GPUI 方法链，通过编译期代码生成实现**零运行时开销**。

### 核心理念

- **零成本抽象**：所有转换都在编译时完成
- **类型安全**：生成的代码充分利用 Rust 类型系统
- **GPUI 原生**：输出与手写的 GPUI 代码模式一致
- **Tailwind 风格**：熟悉的实用类样式系统

## 高层架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         用户代码 (RSX)                          │
│  rsx! { <div class="flex gap-4" onClick={handler}> ... </div> }│
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    解析器 (parser.rs)                           │
│  • 词法分析                                                     │
│  • 递归下降解析                                                 │
│  • AST 构建                                                     │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
                    ┌────────┐
                    │  AST   │
                    └────┬───┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                 代码生成器 (codegen/)                           │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   tables.rs  │  │   class.rs   │  │ attribute.rs │         │
│  │   (查找表)   │◄─┤   (解析)     │◄─┤   (方法)     │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                              │                  │
│                         ┌────────────────────┘                  │
│                         │                                       │
│                         ▼                                       │
│                  ┌──────────────┐                               │
│                  │  element.rs  │                               │
│                  │   (生成)     │                               │
│                  └──────────────┘                               │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    生成的 GPUI 代码                             │
│  div().id("auto_0").flex().gap(px(4.0)).on_click(handler).child│
└─────────────────────────────────────────────────────────────────┘
```

## 模块组织

```
src/
├── lib.rs                     (124 行)   - 宏入口点
├── parser.rs                  (371 行)   - RSX → AST
└── codegen/
    ├── mod.rs                 (~20 行)   - 模块协调
    ├── tables.rs              (~450 行)  - 静态查找表
    ├── class.rs               (~150 行)  - CSS 类解析
    ├── attribute.rs           (~80 行)   - 属性 → 方法
    └── element.rs             (~230 行)  - 元素生成
```

### 模块职责

| 模块 | 用途 | 依赖 | 关键函数 |
|------|------|------|----------|
| `lib.rs` | 宏入口点 | `parser`, `codegen` | `rsx!` 宏 |
| `parser.rs` | RSX 语法解析 | `syn`, `quote` | `parse()`, AST 类型 |
| `codegen/tables.rs` | 常量表 | 无 | `lookup_color()` |
| `codegen/class.rs` | 类解析 | `tables` | `parse_class_string()`, `parse_color_with_method()` |
| `codegen/attribute.rs` | 属性处理 | `tables`, `class` | `generate_attr_methods()` |
| `codegen/element.rs` | 元素生成 | 以上所有 | `generate_body()`, `generate_element()` |

## 数据流

### 1. 宏调用

```rust
rsx! {
    <div class="flex gap-4 bg-blue-500" onClick={handler}>
        {"Hello"}
    </div>
}
```

### 2. 解析阶段

**输入**：来自 `rsx!` 宏的 `TokenStream`
**输出**：`RsxBody` AST

```rust
RsxBody::Single(
    RsxElement {
        name: Ident("div"),
        attributes: [
            RsxAttribute::Value {
                name: "class",
                value: Lit("flex gap-4 bg-blue-500")
            },
            RsxAttribute::Value {
                name: "onClick",
                value: Expr(handler)
            }
        ],
        children: [
            RsxNode::Expr(Lit("Hello"))
        ]
    }
)
```

### 3. 代码生成阶段

**步骤 3a**：元素基础构建

```rust
generate_base() → div().id("__rsx_div_a1b2c3d4")
```

**步骤 3b**：类解析（去重）

```rust
parse_class_string("flex gap-4 bg-blue-500") → [
    .flex(),
    .gap(px(4.0)),
    parse_color_with_method("blue_500", "bg") → .bg(rgb(0x3b82f6))
]
```

**步骤 3c**：属性转换

```rust
generate_attr_methods(onClick={handler}) → .on_click(handler)
```

**步骤 3d**：子元素处理

```rust
generate_children_methods([Expr("Hello")]) → .child("Hello")
```

### 4. 最终输出

```rust
div()
    .id("__rsx_div_a1b2c3d4")
    .flex()
    .gap(px(4.0))
    .bg(rgb(0x3b82f6))
    .on_click(handler)
    .child("Hello")
```

## 关键组件

### 解析器 (parser.rs)

**架构**：使用 `syn::parse::Parse` 的递归下降解析器

**AST 类型**：
- `RsxBody`：顶层（单个元素或 Fragment）
- `RsxElement`：带属性和子元素的标签
- `RsxNode`：Element | Expr | Spread | For
- `RsxAttribute`：Flag | Value | When | WhenSome

**关键特性**：
- Fragment 支持 (`<>...</>`)
- For 循环语法 (`{for item in items { ... }}`)
- 条件渲染 (`when`, `whenSome`)
- 表达式子元素 (`{expr}`)
- 展开语法 (`{...items}`)

### 代码生成器 (codegen/)

#### tables.rs - 基础

**用途**：所有映射的核心数据源

**内容**：
- `COLOR_MAP`：242 种 Tailwind 颜色 (slate, gray, ... rose)
- `EVENT_HANDLERS`：14 个事件映射 (onClick → on_click)
- `ATTRIBUTE_NAME_MAP`：30+ camelCase → snake_case
- `TAG_DEFAULT_STYLES`：11 个语义标签默认样式
- `SPACING_PATTERNS`：17 个间距/尺寸前缀
- `VALID_TEXT_SIZES`：9 个文本大小变量
- `lookup_color()`：快速颜色表查找

**设计**：零依赖，纯 const 数据

#### class.rs - 去重

**用途**：将类字符串解析为方法调用

**关键创新**：`parse_color_with_method()`

**重构前**（重复 3 次）：
```rust
// text_color
if let Some(color) = class.strip_prefix("text_") {
    for &(color_name, color_value) in COLOR_MAP {
        if color == color_name {
            return Some(quote! { .text_color(rgb(#color_value)) });
        }
    }
}

// bg（相同逻辑重复）
// border_color（相同逻辑再次重复）
```

**重构后**（统一）：
```rust
fn parse_color_with_method(color: &str, method: &str) -> Option<TokenStream> {
    // 1. 尝试颜色表
    if let Some(hex) = lookup_color(color) {
        let ident = syn::Ident::new(method, Span::call_site());
        return Some(quote! { .#ident(rgb(#hex)) });
    }
    // 2. 尝试任意十六进制
    if let Some(hex) = parse_arbitrary_hex(color) {
        let ident = syn::Ident::new(method, Span::call_site());
        return Some(quote! { .#ident(rgb(#hex)) });
    }
    None
}

// 使用
parse_color_with_method(color, "text_color")
parse_color_with_method(color, "bg")
parse_color_with_method(color, "border_color")
```

**优势**：
- DRY：3 个实现 → 1 个
- 可维护性：单一修改点
- 一致性：所有颜色处理方式相同

**支持的模式**：
- 命名颜色：`text-red-500` → `.text_color(rgb(0xef4444))`
- 任意十六进制：`bg-[#ff0000]` → `.bg(rgb(0xff0000))`
- 短十六进制：`text-[#f00]` → `.text_color(rgb(0xff0000))`
- 间距：`gap-4` → `.gap(px(4.0))`
- 文本大小：`text-xl` → `.text_xl()`

#### attribute.rs - 映射

**用途**：RSX 属性 → GPUI 方法

**属性类型**：
1. **Flag**：`<div flex />` → `.flex()`
2. **Value**：`<div width={100} />` → `.w(100)`
3. **Class**：`<div class="flex" />` → `.flex()`
4. **Events**：`<div onClick={h} />` → `.on_click(h)`
5. **Conditional**：`<div when={cond, |el| el.flex()} />` → `.when(cond, |el| el.flex())`

**特殊情况**：
- `invisible` → `.visible(false)`
- `styled` → 注入标签默认值（在 element.rs 中处理）
- `id` → 跳过（在 element.rs 基础生成中处理）
- `class` → 必须是字符串字面量（不支持动态值）

#### element.rs - 生成

**用途**：协调所有代码生成

**关键概念**：

1. **方法链**：GPUI 使用流式 API
   ```rust
   div().flex().gap(px(4.0)).child(...)
   ```

2. **类型转换**：`.id()` 改变类型
   ```rust
   Div → Stateful<Div>
   ```

3. **自动 ID 注入**：事件需要有状态的元素
   ```rust
   <div onClick={h} />
   ↓
   div().id("__rsx_div_a1b2c3d4").on_click(h)
   ```

4. **子元素聚合**：优化连续的子元素
   ```rust
   // 3+ 个表达式
   .children(vec![expr1, expr2, expr3])
   // vs
   .child(expr1).child(expr2)
   ```

5. **默认样式**：`styled` 标志注入语义
   ```rust
   <h1 styled>{"Title"}</h1>
   ↓
   div().text_3xl().font_bold().child("Title")
   ```

**自动 ID 算法**：
```rust
fn next_auto_id(tag: &str, attrs: &[Attr]) -> String {
    let mut hasher = DefaultHasher::new();
    tag.hash(&mut hasher);
    for attr in attrs {
        attr.name.hash(&mut hasher);
    }
    let counter = AUTO_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    counter.hash(&mut hasher);
    format!("__rsx_{tag}_{:x}", hasher.finish())
}
```

## 设计模式

### 1. 编译时表

**模式**：使用 `const` 查找表而不是运行时 hashmap

```rust
const COLOR_MAP: &[(&str, u32)] = &[
    ("red_500", 0xef4444),
    // ...
];

fn lookup_color(name: &str) -> Option<u32> {
    COLOR_MAP.iter().find(|(n, _)| *n == name).map(|(_, v)| *v)
}
```

**优势**：
- 零运行时成本
- 无内存分配
- 二进制大小高效（字符串内联）

### 2. 递归下降解析

**模式**：每个语法结构都有专用解析器

```rust
impl Parse for RsxBody {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![<]) {
            if input.peek2(Token![>]) {
                // Fragment
            } else {
                // 单个元素
            }
        }
    }
}
```

### 3. Token 流

**模式**：增量生成 `TokenStream`

```rust
let mut methods = Vec::new();
methods.push(quote! { .flex() });
methods.push(quote! { .gap(px(4.0)) });
quote! { div() #(#methods)* }
```

**优势**：
- 可组合
- 类型安全
- 保留错误的 span 信息

### 4. 方法链构建

**模式**：生成流式 API 调用

```rust
// 错误：变异模式
let mut el = div();
el = el.flex();
el = el.gap(px(4.0));

// 正确：方法链
div().flex().gap(px(4.0))
```

**原因**：GPUI 方法通常改变类型（`.id()` 返回 `Stateful<T>`）

## 测试策略

### 测试金字塔

```
                   ┌──────────────┐
                   │   集成测试   │  示例（手动）
                   │              │
                   └──────────────┘
                  ┌────────────────┐
                  │   宏测试      │  203 个展开测试
                  │               │
                  └────────────────┘
```

### 宏测试 (tests/macro_tests.rs)

**覆盖率**：203 个测试用例

**分类**：
- 元素 (29)：标签、嵌套、自闭合
- 属性 (45)：Flag、值、camelCase/snake_case
- 事件 (18)：所有 14 个事件处理器
- 样式 (32)：类、颜色、间距
- 子元素 (24)：表达式、展开、for 循环
- 条件 (12)：when、whenSome
- 边界情况 (43)：自动 ID（含 onHover/onDrag/onDrop）、样式标签、fragments

**模式**：
```rust
#[test]
fn test_feature() {
    let expanded = quote! {
        rsx! { <div class="flex" /> }
    };
    let expected = quote! {
        div().flex()
    };
    assert_eq!(expanded.to_string(), expected.to_string());
}
```

## 扩展点

### 添加新颜色

**文件**：`src/codegen/tables.rs`

```rust
const COLOR_MAP: &[(&str, u32)] = &[
    // ...现有颜色...
    ("my_custom_500", 0xabcdef),  // 在此添加
];
```

**用法**：`class="text-my-custom-500"` → `.text_color(rgb(0xabcdef))`

### 添加新属性

**文件**：`src/codegen/tables.rs`

```rust
const ATTRIBUTE_NAME_MAP: &[(&str, &str)] = &[
    // ...现有映射...
    ("customAttr", "custom_attr"),  // 在此添加
];
```

**用法**：`<div customAttr={value} />` → `.custom_attr(value)`

### 添加新事件处理器

**文件**：`src/codegen/tables.rs`

```rust
const EVENT_HANDLERS: &[(&str, &str, &str)] = &[
    // ...现有处理器...
    ("onCustom", "on_custom", "on_custom"),  // 在此添加
];
```

**用法**：`<div onCustom={h} />` → `.on_custom(h)` (带自动 ID)

### 添加新间距模式

**文件**：`src/codegen/tables.rs`

```rust
const SPACING_PATTERNS: &[(&str, &str)] = &[
    // ...现有模式...
    ("custom_", "custom"),  // 在此添加
];
```

**用法**：`class="custom-4"` → `.custom(px(4.0))`

### 添加新默认样式

**文件**：`src/codegen/tables.rs`

```rust
const TAG_DEFAULT_STYLES: &[(&str, &str)] = &[
    // ...现有默认值...
    ("myTag", "flex gap-2"),  // 在此添加
];
```

**用法**：`<myTag styled />` → `myTag().flex().gap(px(2.0))`

## 性能考虑

### 编译时

**优化**：
1. **Const 表**：无运行时初始化
2. **线性查找**：小表（< 500 项）
3. **无内存分配**：基于栈的解析
4. **最小克隆**：TokenStream 复用

**基准**：1000 个元素宏展开约 ~0.1 秒

### 运行时

**零成本**：
- 无反射
- 无字符串解析
- 无动态分派
- 与手写 GPUI 代码相同

**生成的代码**：
```rust
// RSX
rsx! { <div class="flex" /> }

// 手写（单态化后相同）
div().flex()
```

### 二进制大小

**影响**：最小

**原因**：
- 无运行时库
- 字符串字面量内联
- 方法调用内联
- 死代码消除

## 调试指南

### 查看生成的代码

```bash
# 安装 cargo-expand
cargo install cargo-expand

# 查看展开的宏
cargo expand --lib

# 特定测试
cargo expand --test macro_tests --tests test_name
```

### 理解错误

**编译错误**：
```
error[E0599]: no method named `flex_col` found for struct `Div`
```

**诊断**：类名拼写错误（`flex-col` vs `flex_col`）

**修复**：使用正确的 Tailwind 类 `flex-col`

### 测试更改

**工作流**：
1. 修改 `src/codegen/` 中的代码
2. 运行 `cargo test --test macro_tests`
3. 检查特定测试：`cargo test test_name`
4. 查看展开：`cargo expand --test macro_tests`
5. 对比：`diff <(cargo expand) expected.rs`

### 常见问题

| 错误 | 原因 | 修复 |
|------|------|------|
| "no method named X" | 无效的 GPUI 方法 | 查看 GPUI 文档 |
| "mismatched types" | `.id()` 类型变化 | 验证自动 ID 注入 |
| "expected struct `Div`" | 缺少自动 ID | 检查 `NEEDS_ID_ATTRS` |
| "cannot find value" | 作用域问题 | 检查表达式转义 |

## 未来改进

### 短期

1. **组件支持**：`<MyComponent prop={value} />`
2. **Ref 处理**：`ref={my_ref}`
3. **更多 Tailwind 工具**：阴影、变换、动画
4. **自定义调色板**：用户定义颜色

### 中期

1. **LSP 集成**：类的自动完成
2. **编译时验证**：对未知类发出警告
3. **热重载**：开发期间快速迭代
4. **源映射**：更好的错误位置

### 长期

1. **主题系统**：暗黑模式、配色方案
2. **响应式设计**：`class="md:flex lg:grid"`
3. **可访问性**：ARIA 属性、语义 HTML
4. **性能分析**：宏展开指标

## 迁移指南

### 从 0.1.x 到 0.2.x

**破坏性更改**：无（仅内部重构）

**验证**：
```bash
# 保证相同输出
cargo expand --lib > before.rs
# 升级
cargo update -p gpui-rsx
cargo expand --lib > after.rs
diff before.rs after.rs  # 应该为空
```

### 从手写 GPUI

**之前**：
```rust
div()
    .flex()
    .flex_col()
    .gap(px(16.0))
    .bg(rgb(0x3b82f6))
    .child("Hello")
```

**之后**：
```rust
rsx! {
    <div class="flex flex-col gap-4 bg-blue-500">
        {"Hello"}
    </div>
}
```

**优势**：
- 减少 40% 代码
- 类 HTML 结构
- Tailwind 熟悉度
- 相同性能

## 参考

### 文档

- [GPUI 文档](https://www.gpui.rs/)
- [Tailwind CSS](https://tailwindcss.com/)
- [syn crate](https://docs.rs/syn/)
- [quote crate](https://docs.rs/quote/)

### 相关项目

- [dioxus](https://dioxuslabs.com/)：用于 web/desktop 的 RSX
- [yew](https://yew.rs/)：用于 WebAssembly 的 RSX
- [leptos](https://leptos.dev/)：带 signals 的 RSX

### 贡献

查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解：
- 代码风格指南
- PR 流程
- 测试要求
- 发布流程

---

**最后更新**：2026-02-17
**版本**：0.2.0
**维护者**：@wangshian
