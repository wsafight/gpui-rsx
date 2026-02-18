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
│                          │                   │                  │
│                          └─────────┬─────────┘                  │
│                                    ▼                            │
│                  ┌──────────────────────────┐                   │
│                  │  element.rs（生成）       │                   │
│                  └────────────┬─────────────┘                   │
│                               │                                 │
│              ┌────────────────┘                                 │
│              ▼                                                   │
│  ┌──────────────────┐                                           │
│  │   runtime.rs     │  （仅动态 class）                         │
│  └──────────────────┘                                           │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    生成的 GPUI 代码                             │
│  div().id("__rsx_div_0").flex().gap(px(4.0)).on_click(handler) │
└─────────────────────────────────────────────────────────────────┘
```

## 模块组织

```
src/
├── lib.rs                     (~123 行)  - 宏入口点
├── parser.rs                  (~311 行)  - RSX → AST
├── diagnostics.rs             (~110 行)  - 错误消息
└── codegen/
    ├── mod.rs                 (~24 行)   - 模块协调
    ├── tables.rs              (~417 行)  - O(1) match 查找表
    ├── class.rs               (~151 行)  - CSS class 解析
    ├── attribute.rs           (~79 行)   - 属性 → 方法
    ├── element.rs             (~250 行)  - 元素生成 + 自动 ID
    └── runtime.rs             (~170 行)  - 动态 class 代码生成
```

### 模块职责

| 模块 | 用途 | 依赖 | 关键函数 |
|------|------|------|----------|
| `lib.rs` | 宏入口点 | `parser`, `codegen` | `rsx!` 宏 |
| `parser.rs` | RSX 语法解析 | `syn`, `quote` | `parse()`, AST 类型 |
| `diagnostics.rs` | 错误消息 | `syn` | span 感知错误构造器 |
| `codegen/tables.rs` | O(1) match 查找 | 无 | `lookup_color()`, `lookup_attr_method()` |
| `codegen/class.rs` | class 解析 | `tables` | `parse_class_string()`, `parse_color_with_method()` |
| `codegen/attribute.rs` | 属性处理 | `tables`, `class`, `runtime` | `generate_attr_methods()` |
| `codegen/element.rs` | 元素生成 | 以上所有 | `generate_body()`, `generate_element()` |
| `codegen/runtime.rs` | 动态 class 生成 | `class` | `generate_dynamic_class_code()` |

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

**步骤 3a**：单次属性扫描提取 `user_id`、`has_styled`、`needs_id`

```rust
// onClick 是有状态属性，needs_id = true
// → 注入自动 ID
generate_base() → div().id("__rsx_div_0")
```

**步骤 3b**：class 解析——字符串字面量 → 编译期展开

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

**步骤 3d**：子节点处理

```rust
generate_children_methods([Expr("Hello")]) → .child("Hello")
```

### 4. 最终输出

```rust
div()
    .id("__rsx_div_0")
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
- `RsxElement`：带属性和子节点的标签
- `RsxNode`：Element | Expr | Spread | For
- `RsxAttribute`：Flag | Value | When | WhenSome

**关键特性**：
- Fragment 支持（`<>...</>`）
- For 循环语法（`{for item in items { ... }}`）
- 条件渲染（`when`、`whenSome`）
- 表达式子节点（`{expr}`）
- 展开语法（`{...items}`）

### 代码生成器 (codegen/)

#### tables.rs — O(1) 查找基础

**用途**：所有编译期映射的核心数据源。

**所有查找均使用 `match` 语句**——编译器为 match 生成高效跳转表或 trie 结构，
最坏情况下 O(1)，无运行时初始化成本。

**函数列表**：

| 函数 | 条目数 | 描述 |
|------|--------|------|
| `lookup_color(name)` | 242 | 完整 Tailwind 色板（所有色阶 + black/white） |
| `lookup_attr_method(name)` | 15 事件 + 30+ 属性 | camelCase/snake_case → GPUI 方法名 |
| `lookup_spacing_method(prefix)` | 17 | `"gap_"`、`"px_"`… → GPUI 方法名 |
| `is_valid_text_size(size)` | 9 | `"xs"` … `"5xl"` 白名单 |
| `lookup_tag_default(tag)` | 11 | 语义标签默认 class 字符串 |
| `is_stateful_attr(name)` | — | `starts_with("on_")` + 显式 match |

**设计**：零依赖，纯函数，无堆分配。

#### class.rs — class 字符串解析

**用途**：将 Tailwind 风格的 class 字符串解析为 GPUI 方法调用 `TokenStream`。

**关键创新**：

1. **统一的 `parse_color_with_method(color, method)`** — 被 `text_color`、`bg`、
   `border_color` 三条路径共享，消除了三个近乎相同的实现。

2. **`rfind('_') + match` 前缀查找** — O(1) 间距前缀检测，无需扫描完整字符串。

3. **零堆分配 3 位 hex 展开** — `[#abc]` → `0xaabbcc` 通过位运算半字节复制实现，
   不分配任何 `String`。

4. **`Cow<str>` 实现 `-` → `_` 转换** — 不含连字符时零拷贝借用；仅在需要替换时分配。

**支持的模式**：
- 命名颜色：`text-red-500` → `.text_color(rgb(0xef4444))`
- 任意 hex 6 位：`bg-[#ff0000]` → `.bg(rgb(0xff0000))`
- 任意 hex 3 位：`text-[#f00]` → `.text_color(rgb(0xff0000))`
- 间距：`gap-4` → `.gap(px(4.0))`
- 文本大小：`text-xl` → `.text_xl()`
- 边框：`border` → `.border_1()`，`border-2` → `.border_2()`

#### attribute.rs — 属性到方法的映射

**用途**：RSX 属性 → GPUI 方法调用 `TokenStream`

**属性类型**：
1. **Flag**：`<div flex />` → `.flex()`
2. **Value**：`<div width={100} />` → `.w(100)`
3. **Class（静态）**：`<div class="flex" />` → `.flex()`（编译期）
4. **Class（动态）**：`<div class={expr} />` → 通过 `runtime.rs` 运行时 match
5. **事件**：`<div onClick={h} />` → `.on_click(h)`
6. **条件**：`<div when={(cond, |el| el.flex())} />` → `.when(cond, …)`

**特殊情况**：
- `invisible` → `.visible(false)`
- `styled` → 注入标签默认样式（在 `element.rs` 中在用户属性前处理）
- `id` → 此处跳过；在 `element.rs` 基础生成中处理

#### element.rs — 生成编排

**用途**：将所有代码生成编排为完整的方法链。

**关键概念**：

1. **方法链** — GPUI 使用流式 API，每个方法返回 `Self`（`.id()` 后返回新类型）：
   ```rust
   div().flex().gap(px(4.0)).child(...)
   ```

2. **类型转换** — `.id()` 改变返回类型：
   ```rust
   Div → Stateful<Div>
   ```
   生成代码必须在任何有状态方法前链式调用 `.id()`。

3. **单次属性扫描** — `user_id`、`has_styled`、`needs_id` 在一次循环中提取：
   ```rust
   for attr in &element.attributes {
       match attr {
           RsxAttribute::Value { name, value } if name == "id" => user_id = Some(value),
           RsxAttribute::Flag(name) if name == "styled" => has_styled = true,
           RsxAttribute::Value { name, .. } | RsxAttribute::Flag(name) => {
               if !needs_id { needs_id = is_stateful_attr(&name.to_string()); }
           }
           _ => {}
       }
   }
   ```

4. **自动 ID 注入** — 含有状态属性的元素自动获得确定性 ID：
   ```rust
   <div onClick={h} />
   ↓
   div().id("__rsx_div_0").on_click(h)
   ```

5. **子节点聚合** — 3+ 个连续 `Expr` 子节点批量处理：
   ```rust
   // 3+ 个连续表达式 → 单次 .children([...])
   .children([expr1, expr2, expr3])

   // < 3 → 独立 .child() 调用
   .child(expr1).child(expr2)
   ```

6. **for 循环代码生成** — 单子节点用 `.map()`；多子节点用 `.flat_map()` + `vec![]`
   以支持混合元素类型：
   ```rust
   // 单个子节点
   (iter).into_iter().map(|binding| child_expr)

   // 多个子节点（vec! 允许不同元素类型）
   (iter).into_iter().flat_map(|binding| vec![child1, child2])
   ```

**自动 ID 计数器**：
```rust
// thread_local 计数器；在单次编译进程中单调递增。
// 已知限制：增量编译可能改变展开顺序，导致 ID 变化。
// 依赖 ID 稳定性的元素应显式指定 `id` 属性。
thread_local! {
    static AUTO_ID_COUNTER: Cell<usize> = const { Cell::new(0) };
}

fn next_auto_id(tag: &str) -> String {
    AUTO_ID_COUNTER.with(|c| {
        let n = c.get();
        c.set(n + 1);
        format!("__rsx_{tag}_{n}")
    })
}
```

#### runtime.rs — 动态 class 代码生成

**用途**：为 `class={expression}` 属性生成运行时代码。

**重要限制**：运行时仅识别约 58 个预编译的常用 class。不在列表中的 class 会被**静默忽略**。
建议优先使用静态字符串字面量以获得完整 class 支持。

**预编译的常用 class**（部分示例）：
```
flex, flex-col, flex-row, flex-1, items-center, justify-center,
gap-1..gap-8, p-1..p-8, px-2, px-4, py-1..py-4, m-2, m-4,
w-full, h-full, text-xs..text-3xl, font-bold, border, rounded-*,
cursor-pointer, overflow-hidden, bg-white, bg-black, …
```

**生成的代码模式**：
```rust
{
    #[inline(never)]  // 阻止 match 表内联；支持 LLVM ICF 合并
    fn __rsx_apply_class<E: Styled>(el: E, class: &str) -> E {
        match class {
            "flex" => el.flex(),
            "gap-4" => el.gap(px(4.0)),
            // … 约 58 个预编译 class …
            _ => el,  // 未知 class → 静默忽略
        }
    }
    let __class_expr = <expression>;
    let __class_str: &str = __class_expr.as_ref();  // &str 零拷贝
    __class_str.split_whitespace().fold(__el, __rsx_apply_class)
}
```

## 设计模式

### 1. 基于 match 的 O(1) 查找表

**模式**：纯函数内的 `match` 语句，替代运行时 hashmap 或线性扫描常量数组。

```rust
pub(crate) fn lookup_color(name: &str) -> Option<u32> {
    match name {
        "red_500" => Some(0xef4444),
        "blue_500" => Some(0x3b82f6),
        // … 242 条 …
        _ => None,
    }
}
```

Rust 编译器为这些 match 语句生成高效的跳转表或 trie，实现 O(1) 查找，无运行时初始化，零堆分配。

### 2. 递归下降解析

**模式**：每个语法结构实现 `syn::parse::Parse`

```rust
impl Parse for RsxBody {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![<]) && input.peek2(Token![>]) {
            // Fragment <>...</>
        } else {
            // 单个元素
        }
    }
}
```

### 3. Token 流——直接推送到调用方

**模式**：增量生成 `TokenStream`；属性方法直接推送到调用方的 `Vec`，避免中间分配。

```rust
pub(crate) fn generate_attr_methods(attr: &RsxAttribute, out: &mut Vec<TokenStream>) {
    // 直接推送到 out，无中间 Vec
    out.push(quote! { .flex() });
}
```

### 4. 方法链构建

**模式**：生成流式 API 调用，而非赋值风格。

```rust
// 错误：赋值模式（.id() 改变类型后会失败）
let mut el = div();
el = el.flex();

// 正确：方法链
div().flex().gap(px(4.0))
```

**原因**：GPUI 的 `.id()` 返回 `Stateful<T>`，是不同类型。链式调用是唯一正确的模式。

### 5. proc-macro 上下文的 thread_local 缓存

**模式**：`thread_local! + Cell/RefCell` 用于同一编译单元内跨宏调用共享的状态。

```rust
// proc macro 单线程执行；thread_local 语义更准确且比 AtomicUsize 无原子操作开销
thread_local! {
    static AUTO_ID_COUNTER: Cell<usize> = const { Cell::new(0) };
    static COMMON_CLASS_MATCHES: RefCell<Option<Rc<Vec<TokenStream>>>> = ...;
}
```

## 测试策略

### 测试金字塔

```
               ┌──────────────────┐
               │ diagnostic_tests │  2 个编译错误格式测试
               └──────────────────┘
              ┌────────────────────┐
              │  coverage_tests    │  31 个边界情况/行为测试
              └────────────────────┘
            ┌──────────────────────┐
            │    macro_tests       │  203 个展开正确性测试
            └──────────────────────┘
```

### 宏测试 (tests/macro_tests.rs)

**覆盖率**：203 个测试用例

**分类**：
- 元素（29）：标签、嵌套、自闭合、特殊标签
- 属性（45）：Flag、值、camelCase/snake_case
- 事件（18）：所有 15 个事件处理器 + 自动 ID
- 样式（32）：class、颜色、间距、边框
- 子节点（24）：表达式、展开、for 循环、聚合
- 条件（12）：when、whenSome
- 边界情况（43）：自动 ID、styled 标签、fragments、invisible

**模式**：
```rust
#[test]
fn test_feature() {
    let result = quote! { rsx! { <div class="flex" /> } };
    let expected = quote! { div().flex() };
    assert_eq!(result.to_string(), expected.to_string());
}
```

## 扩展点

### 添加新颜色

**文件**：`src/codegen/tables.rs` → `lookup_color()`

添加新 match 分支：
```rust
pub(crate) fn lookup_color(name: &str) -> Option<u32> {
    match name {
        // …现有颜色…
        "my_brand_500" => Some(0xabcdef),  // 在此添加
        _ => None,
    }
}
```

**用法**：`class="text-my-brand-500"` → `.text_color(rgb(0xabcdef))`

### 添加新属性映射

**文件**：`src/codegen/tables.rs` → `lookup_attr_method()`

```rust
pub(crate) fn lookup_attr_method(name: &str) -> Option<&'static str> {
    match name {
        // …现有映射…
        "customAttr" | "custom_attr" => Some("custom_attr"),  // 在此添加
        _ => None,
    }
}
```

**用法**：`<div customAttr={value} />` → `.custom_attr(value)`

### 添加新事件处理器

**文件**：`src/codegen/tables.rs` — 需要两处修改：

1. 在 `lookup_attr_method()` 中添加：
   ```rust
   "onCustom" | "on_custom" => Some("on_custom"),
   ```

2. 若该事件需要有状态元素（`.id()`），同时更新 `is_stateful_attr()`：
   ```rust
   pub(crate) fn is_stateful_attr(name: &str) -> bool {
       // on_ 前缀已自动处理；为 camelCase 形式添加显式匹配：
       matches!(name, "hover" | "active" | … | "onCustom")
   }
   ```

### 添加新间距前缀

**文件**：`src/codegen/tables.rs` → `lookup_spacing_method()`

```rust
pub(crate) fn lookup_spacing_method(prefix: &str) -> Option<&'static str> {
    match prefix {
        // …现有前缀…
        "inset_" => Some("inset"),  // 在此添加
        _ => None,
    }
}
```

**用法**：`class="inset-4"` → `.inset(px(4.0))`

### 添加新标签默认样式

**文件**：`src/codegen/tables.rs` → `lookup_tag_default()`

```rust
pub(crate) fn lookup_tag_default(tag: &str) -> Option<&'static str> {
    match tag {
        // …现有默认值…
        "nav" => Some("flex items-center"),  // 在此添加
        _ => None,
    }
}
```

**用法**：`<nav styled />` → `div().flex().items_center()`

### 扩展动态 class 识别范围

**文件**：`src/codegen/runtime.rs` → `generate_common_class_matches()`

```rust
let common_classes = [
    // …现有 class…
    "my-custom-class",  // 在此添加；将被预编译到 match 表中
];
```

## 性能考虑

### 编译时

**宏展开优化**：
1. **O(1) match 查找** — 所有表使用 `match`，无线性扫描
2. **单次属性扫描** — `generate_element` 在一次循环中提取所有信息
3. **缓存 `Ident::to_string()`** — 每次调用时字符串转换一次
4. **返回迭代器** — `parse_class_string` 返回迭代器，不分配 `Vec`
5. **直接 Vec push** — `generate_attr_methods` 推送到调用方缓冲区
6. **`Vec::with_capacity` 预分配** — 全面使用合理容量提示
7. **thread_local 缓存** — 常用 class match 分支每进程只生成一次

### 运行时

**零成本** — 生成的 GPUI 代码与手写代码完全相同：
```rust
// RSX
rsx! { <div class="flex gap-4" onClick={handler} /> }

// 生成代码（单态化后与手写相同）
div().id("__rsx_div_0").flex().gap(px(4.0)).on_click(handler)
```

无反射、无字符串解析、无动态分发。

**动态 class 例外** — `class={expression}` 生成运行时 `fold` + `match`。
如需零开销样式，使用静态字符串字面量。

### 二进制体积

- 无运行时库
- 字符串字面量由链接器内联
- 动态 class 辅助函数用 `#[inline(never)]` 防止 match 表重复
- LLVM ICF 跨组件合并相同的单态化实例

## 调试指南

### 查看生成的代码

```bash
# 安装 cargo-expand
cargo install cargo-expand

# 查看所有展开的宏
cargo expand --lib

# 特定测试
cargo test test_name -- --nocapture
```

### 理解错误

**常见模式**：
```
error[E0599]: no method named `flex_col` found for struct `Div`
```

**诊断**：class 名拼写错误——`flex-col` 不在预编译列表中，被当作方法名字面量传入。

**修复**：检查 class 名拼写，确认在支持列表中。

### 常见问题

| 错误 | 原因 | 修复 |
|------|------|------|
| `no method named X` | 无效的 GPUI 方法名 | 查看 GPUI 文档 |
| `mismatched types` | `.id()` 类型变化未处理 | 验证自动 ID 是否注入 |
| 动态 class 未生效 | class 不在约 58 个常用列表中 | 改用静态字符串字面量 |
| 重新构建后自动 ID 变化 | 增量编译改变了展开顺序 | 添加显式 `id` 属性 |
| `expected &str, found String` | 传入 `class={}` 的类型错误 | 使用 `.as_str()` 或字面量 |

### 测试更改

**工作流**：
1. 修改 `src/codegen/` 中的代码
2. 运行 `cargo test`（全部 236 个测试）
3. 检查特定测试：`cargo test test_name`
4. 查看生成代码：`cargo expand --test macro_tests`

## 未来改进

### 短期

1. **扩展动态 class 覆盖范围** — 根据实际使用数据扩展约 58 个预编译 class 列表
2. **未知 class 编译时警告** — 对静态 class 名称中的未知 class 发出 `proc_macro_warning`
3. **更多 Tailwind 工具类** — 阴影、变换、动画
4. **自定义调色板** — 用户定义颜色 token

### 中期

1. **LSP 集成** — class 名称和属性的自动补全
2. **快照测试** — 通过 `insta` 进行生成代码的回归检测
3. **源映射** — 指向 RSX 语法的更好错误位置
4. **`trybuild` 编译失败测试** — 恢复错误消息验证

### 长期

1. **主题系统** — 暗黑模式、CSS 自定义属性
2. **响应式设计** — `class="md:flex lg:grid"`
3. **可访问性** — ARIA 属性、语义化 HTML
4. **性能分析** — 使用 `criterion` 测量宏展开指标

## 迁移指南

### 从 0.1.x 到 0.2.x

**破坏性更改**：无（仅内部重构）

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

**优势**：代码减少约 50%，类 HTML 结构，Tailwind 熟悉度，性能完全相同。

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

查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解代码风格指南、PR 流程、测试要求和发布流程。

---

**最后更新**：2026-02-18
**版本**：0.2.1（+ 未发布修复）
**维护者**：@wangshian
