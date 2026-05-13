# gpui-rsx 布局与样式能力改造方案

本文档把 `gpui-rsx` 的布局与样式优化收敛成可执行方案。重点是补齐桌面应用常用能力，同时避免破坏当前已经公开的 class 语义。

## 当前完成状态

截至当前版本，本文档内 P0、P1、P2 中确定要做的功能均已落地并通过验证：

- `font-*` 已统一展开到 `font_weight(FontWeight::...)`，覆盖 `font-thin`、`font-extralight`、`font-light`、`font-normal`、`font-medium`、`font-semibold`、`font-bold`、`font-extrabold`、`font-black`。
- `styled` 标题 preset 继续使用 `font-bold` 字符串，但会经统一解析路径生成 GPUI 0.2 可用的 `font_weight(FontWeight::BOLD)`。
- 现有数值 class 继续保持 `px(n)` 语义，例如 `gap-4 -> .gap(px(4.0))`、`w-64 -> .w(px(64.0))`。
- 静态与动态 class 均支持新增尺寸语法：`w-[280px]`、`w-[18rem]`、`w-[37.5%]`、`w-6/24`、`max-w-[32rem]`、`gap-x-[14px]`、`mx-[1.25rem]` 等。
- spacing 百分比继续拒绝，例如 `gap-[10%]`、`p-[10%]` 会产生可读编译错误。
- 颜色 arbitrary values 已支持 `#rgb`、`#rgba`、`#rrggbb`、`#rrggbbaa`、`rgb(r,g,b)`、`rgba(r,g,b,a)`，静态与动态路径一致。
- GPUI 0.2 缺失或不兼容的常用 class 已提供兼容展开：`items-stretch`、`justify-evenly`、`content-stretch`、`self-*`、`flex-grow-0`。
- 已新增 `debug-outline`，debug 构建中启用 GPUI debug 边框，release 构建中为空操作。
- 已新增 `rsx_strict!`、`rsx_permissive!`、`rsx_expand!`。
- README、README_CN、API reference 和 crate docs 已同步说明这些能力，并加入桌面三栏布局示例。

验证命令：

```bash
cargo fmt --check
cargo test
cargo bench --no-run
```

Tailwind spacing scale 已明确不引入。`gap-4`、`p-4`、`w-64` 继续按项目既有像素语义展开，避免破坏当前用户界面尺寸。

## 背景与现状

改造前代码已经支持一部分 Tailwind-like class，但存在几个不一致点：

- `class="font-bold"` 会展开为 `.font_bold()`。在 `gpui 0.2.x` 下这可能不是有效 API，应该改为 `font_weight(...)` 系列映射。
- `h1` / `h2` 等 `styled` preset 会注入 `font-bold`，因此会继承同一个兼容问题。
- 现有数值 class 语义是 `px(n)`，例如 `gap-4 -> .gap(px(4.0))`、`w-64 -> .w(px(64.0))`。P0 必须保留这个行为，不能改成 Tailwind spacing scale。
- 任意 hex 颜色已经支持 `bg-[#fff]`、`text-[#334155]`、`border-[#e2e8f0]` 这类格式，P0 不需要重复实现。
- 常用 Flex class、`gap-x-*`、`gap-y-*`、`min-w-0`、`min-h-0` 已经通过静态或动态解析路径部分支持，后续重点是补齐缺口和保持静态/动态行为一致。
- 静态未知 class 可能被直接展开成不存在的 GPUI 方法，导致用户看到底层方法错误；动态未知 class 当前是 debug warning / release ignore。P0 应先改善静态 class 的关键错误诊断，动态 strict mode 可以后置。

## 目标

P0 目标：

1. 修复 `gpui 0.2.x` 下 `font-bold`、标题 preset 等字体权重展开错误。
2. 在不改变现有 `px(n)` 数值语义的前提下，新增桌面布局必需的尺寸语法：`w-[280px]`、`w-[18rem]`、`w-[37.5%]`、`w-6/24`。
3. 补齐静态 class 和动态 class 的关键尺寸解析缺口，至少保证文档示例可用。
4. 对非法 arbitrary value、非法分数、明确不支持的静态 class 给出可读诊断。
5. 文档明确说明 `gpui-rsx` 是 Tailwind-like 子集，不是完整 Tailwind。

非目标：

- 不实现新的布局引擎。GPUI 负责真实布局与渲染。
- 不追求完整 Tailwind 兼容。
- 不在 P0 支持复杂响应式断点、媒体查询、container query。
- 不引入 Tailwind spacing scale；`gap-4`、`p-4`、`w-64` 等 numeric class 固定保持 `px(n)` 语义。

## GPUI 与 gpui-rsx 的职责边界

GPUI 负责真实布局与渲染，例如：

```rust
div()
    .flex()
    .flex_col()
    .gap(px(4.0))
    .w(px(280.0))
    .min_w(px(280.0))
```

`gpui-rsx` 负责把 class 字符串解析并展开成 GPUI API：

```rust
rsx! {
    <aside class="flex flex-col gap-4 w-[280px] min-w-[280px]">
        "项目栏"
    </aside>
}
```

P0 展开语义：

```rust
div()
    .flex()
    .flex_col()
    .gap(px(4.0))
    .w(px(280.0))
    .min_w(px(280.0))
```

## 尺寸系统设计

建议把新增尺寸解析统一走一个内部值类型，避免 `w-*`、`p-*`、`gap-*` 各自写一套 arbitrary value 解析。

```rust
enum LengthValue {
    Px(f32),
    Rem(f32),
    Relative(f32),
    Auto,
    Full,
}
```

展开规则：

```text
Px(v)       -> px(v)
Rem(v)      -> rems(v)
Relative(v) -> relative(v)
Auto        -> auto()
Full        -> relative(1.0)
```

注意：不是所有属性族都应该接受所有 `LengthValue`。

| 属性族 | P0 支持值 | 说明 |
|---|---|---|
| `w-*` / `h-*` / `size-*` | `px`、`rem`、`%`、分数、`full`、`auto` | 桌面布局主路径 |
| `min-w-*` / `max-w-*` / `min-h-*` / `max-h-*` | `px`、`rem`、`%`、分数 | `auto` 对 min/max 尺寸意义不清，P0 不支持 |
| `gap-*` / `gap-x-*` / `gap-y-*` | `px`、`rem` | 百分比 gap 容易和 GPUI 类型能力冲突，P0 不支持 |
| `p-*` / `px-*` / `py-*` / `m-*` / `mx-*` / `my-*` 等 | `px`、`rem` | P0 保守支持长度，不支持百分比 |

### 现有数值 token

保留现有 Tailwind-like token：

```text
w-0
w-1
w-2
w-4
w-16
w-64
h-12
p-4
gap-3
```

P0 继续按当前项目语义展开为 `px(n)`：

```text
w-64  -> .w(px(64.0))
h-12  -> .h(px(12.0))
p-4   -> .p(px(4.0))
gap-3 -> .gap(px(3.0))
```

不引入 Tailwind spacing scale，因此不会采用下面这种 Tailwind rem scale 语义：

```text
1  -> rems(0.25)
2  -> rems(0.5)
3  -> rems(0.75)
4  -> rems(1.0)
64 -> rems(16.0)
```

这会明显改变现有界面尺寸，已明确排除，不再作为 P2 待办项。

### Arbitrary px

用于固定桌面面板、工具栏、事件流：

```rust
class="w-[280px] min-w-[280px] max-w-[360px] h-[48px] p-[18px] gap-[14px]"
```

展开：

```rust
.w(px(280.0))
.min_w(px(280.0))
.max_w(px(360.0))
.h(px(48.0))
.p(px(18.0))
.gap(px(14.0))
```

### Arbitrary rem

用于随字体基准缩放的桌面 UI：

```rust
class="w-[18rem] p-[1.25rem] gap-[0.75rem]"
```

展开：

```rust
.w(rems(18.0))
.p(rems(1.25))
.gap(rems(0.75))
```

### 百分比

P0 百分比只用于尺寸属性族：

```rust
class="w-[37.5%] h-[50%] max-w-[80%]"
```

展开：

```rust
.w(relative(0.375))
.h(relative(0.5))
.max_w(relative(0.8))
```

`p-[10%]`、`gap-[10%]` 这类写法 P0 应报错，而不是生成可能无法通过 GPUI 类型检查的代码。

### 分数尺寸

支持 Tailwind 风格分数，优先用于 `w-*`，同时可扩展到 `h-*`、`size-*`：

```text
w-1/2
w-1/3
w-2/3
w-1/4
w-3/4
w-1/24
w-6/24
```

展开：

```rust
w-1/24 -> .w(relative(1.0 / 24.0))
w-6/24 -> .w(relative(6.0 / 24.0))
```

分数 class 必须在把 `-` / `/` 替换成 `_` 之前解析，否则 `w-6/24` 会退化成 Rust 方法名 `w_6_24()`，无法表达任意分母。

## 推荐尺寸语法矩阵

| class | P0 语义 | 展开 |
|---|---|---|
| `w-64` | 当前数值像素语义 | `.w(px(64.0))` |
| `w-full` | 100% | `.w(relative(1.0))` 或当前等价 helper |
| `w-auto` | auto | `.w(auto())` 或当前等价 helper |
| `w-[280px]` | 固定像素 | `.w(px(280.0))` |
| `w-[18rem]` | rem | `.w(rems(18.0))` |
| `w-[37.5%]` | 百分比 | `.w(relative(0.375))` |
| `w-6/24` | 分数 | `.w(relative(6.0 / 24.0))` |
| `min-w-[280px]` | 最小宽度 | `.min_w(px(280.0))` |
| `max-w-[32rem]` | 最大宽度 | `.max_w(rems(32.0))` |
| `h-[48px]` | 固定高度 | `.h(px(48.0))` |
| `gap-[14px]` | 固定 gap | `.gap(px(14.0))` |
| `gap-[0.75rem]` | rem gap | `.gap(rems(0.75))` |
| `p-[18px]` | padding | `.p(px(18.0))` |

## Flex 映射

GPUI 已有 Flex 风格 API，`gpui-rsx` 只需要补齐映射并确认静态/动态路径一致。

P0 需要保证以下 class 可用：

```text
flex
flex-row
flex-col
flex-wrap
flex-nowrap
flex-1
flex-auto
flex-none
min-w-0
min-h-0

items-start
items-center
items-end
items-stretch

justify-start
justify-center
justify-end
justify-between
justify-around

gap-*
gap-x-*
gap-y-*
```

桌面应用中 `min-w-0` 和 `min-h-0` 很重要。三栏布局里如果中间区域没有 `min-w-0`，长文本或事件卡片容易把侧栏挤出窗口。

## 字体权重映射

改造前问题：

```rust
class="font-bold"
```

错误展开：

```rust
.font_bold()
```

`gpui 0.2.x` 下应改为 `font_weight(...)` 系列调用。具体常量名以 GPUI 0.2 实际导出的 `FontWeight` API 为准；如果确认以下名称存在，建议映射为：

```text
font-thin       -> font_weight(FontWeight::THIN)
font-extralight -> font_weight(FontWeight::EXTRA_LIGHT)
font-light      -> font_weight(FontWeight::LIGHT)
font-normal     -> font_weight(FontWeight::NORMAL)
font-medium     -> font_weight(FontWeight::MEDIUM)
font-semibold   -> font_weight(FontWeight::SEMIBOLD)
font-bold       -> font_weight(FontWeight::BOLD)
font-extrabold  -> font_weight(FontWeight::EXTRA_BOLD)
font-black      -> font_weight(FontWeight::BLACK)
```

同时，`h1` / `h2` 等 `styled` preset 不能直接注入会展开成不兼容方法的 `font-bold`。有两种可选实现：

1. 保留 preset 字符串为 `text-3xl font-bold`，但让 `parse_single_class("font-bold")` 走统一字体权重映射。
2. 把 preset 改成结构化默认样式，避免 preset 再经过字符串 class 解析。

P0 推荐选 1，改动小且能保证用户手写 `font-bold` 与 preset 行为一致。

## 颜色 arbitrary values

改造前项目已经支持：

```text
bg-[#0f172a]
text-[#334155]
border-[#e2e8f0]
```

展开：

```rust
.bg(rgb(0x0f172a))
.text_color(rgb(0x334155))
.border_color(rgb(0xe2e8f0))
```

改造后已支持格式：

```text
#rgb
#rgba
#rrggbb
#rrggbbaa
rgb(15,23,42)
rgba(15,23,42,0.8)
```

## 解析顺序

静态 class 建议按以下顺序解析：

1. 特殊兼容映射，例如 `font-bold -> font_weight(...)`、`border -> border_1()`。
2. arbitrary value，例如 `w-[280px]`、`gap-[0.75rem]`、`bg-[#fff]`。
3. 分数 token，例如 `w-6/24`。
4. 当前数值 token，例如 `w-64`、`p-4`、`gap-3`，继续展开为 `px(n)`。
5. 直接 class 映射，例如 `flex`、`flex-col`、`items-center`。
6. 失败诊断。

`styled` 标签 preset 不属于单个 class 的解析顺序。当前行为是：

```text
base element -> styled preset methods -> user attributes/class methods -> children
```

这个顺序应该保留。用户写在元素上的 class 或属性排在 preset 后面，才符合“用户覆盖默认样式”的直觉。

优先级建议：

- 保持 class 字符串顺序生成方法链；同一属性后出现的 class 通常由 GPUI 后续 builder 调用覆盖前面的值。
- 重复 class 不报错。
- 静态 class 中明确无法支持的 arbitrary value 和非法分数必须报错。
- 静态未知 class 默认仍使用 permissive 策略；`rsx_strict!` 可切换为 strict 策略并对不支持 class 报错。
- 动态 class 在 permissive 模式下继续保持 debug warning / release ignore；在 `rsx_strict!` 下运行时遇到不支持 class 会 panic。

## 错误诊断

当前宏错误可能表现为：

```text
no method named `font_bold` found for struct `gpui::Div`
```

P0 应避免生成这类已知不兼容调用，直接展开到兼容 API。对非法输入则在宏解析阶段报更清晰的错误。

非法 arbitrary value：

```text
Invalid length class `w-[abc]`.
Expected a numeric value with px, rem, or %, for example `w-[280px]`.
```

属性族不支持该单位：

```text
Invalid spacing class `gap-[10%]`.
Percentage values are only supported for sizing classes such as `w-*` and `h-*`.
```

非法分数：

```text
Invalid fraction `w-6/0`: denominator must be greater than 0.
```

未知字体权重：

```text
Unsupported font weight class `font-heavy`.
Supported font weight classes: font-thin, font-extralight, font-light, font-normal, font-medium, font-semibold, font-bold, font-extrabold, font-black.
```

## 版本适配策略

P0 不建议马上引入多版本 feature。项目文档当前以 `gpui = "0.2"` 为主，先把默认映射修到 GPUI 0.2 可用即可。

如果后续需要同时支持多个 GPUI 版本，再引入显式版本映射：

```rust
enum GpuiVersion {
    V0_2,
    Future,
}

enum Expansion {
    MethodCall(&'static str),
    MethodCallWithExpr {
        method: &'static str,
        expr: TokenStream,
    },
}
```

示例：

```text
font-bold + gpui 0.2 -> font_weight(FontWeight::BOLD)
font-bold + future   -> font_bold()
```

可选 Cargo feature：

```toml
[features]
gpui-0-2 = []
```

只有当 crate 真的需要同时支持多个 GPUI 主版本时，才应该让用户通过 feature 选择映射。

## 桌面三栏布局示例

推荐官方文档增加这种真实桌面应用示例：

```rust
rsx! {
    <div class="flex h-full w-full bg-zinc-100">
        <nav class="w-[72px] min-w-[72px] bg-zinc-950" />

        <aside class="w-[280px] min-w-[280px] border-r border-zinc-200 bg-white">
            "项目与任务"
        </aside>

        <main class="flex-1 min-w-0 p-[18px]">
            "对话、计划、diff 和完成页"
        </main>

        <aside class="w-6/24 min-w-[320px] max-w-[460px] border-l border-zinc-200 bg-white">
            "执行轨迹"
        </aside>
    </div>
}
```

这个例子同时覆盖：

- 固定导航栏
- 固定最小宽度的项目栏
- 自适应主区域
- 分数宽度的右侧事件流
- `min-w-0` 防止文本撑破布局

## 测试建议

已增加 unit test、trybuild compile test 或宏展开预览测试覆盖：

1. `font-bold` 展开为 `font_weight(FontWeight::BOLD)` 或 GPUI 0.2 的等价调用。
2. `h1 styled` preset 不生成不存在的 GPUI 方法。
3. `w-64` 继续展开为 `w(px(64.0))`，验证 P0 不破坏现有语义。
4. `w-[280px]` 展开为 `w(px(280.0))`。
5. `w-[18rem]` 展开为 `w(rems(18.0))`。
6. `w-[37.5%]` 展开为 `w(relative(0.375))`。
7. `w-6/24` 展开为 `w(relative(6.0 / 24.0))`。
8. `min-w-0` 展开为 `min_w(px(0.0))` 或当前等价 API。
9. `gap-[14px]` 展开为 `gap(px(14.0))`。
10. `gap-[0.75rem]` 展开为 `gap(rems(0.75))`。
11. `gap-[10%]` 输出可读诊断。
12. `w-[abc]` 输出可读诊断。
13. `w-6/0` 输出可读诊断。
14. `bg-[#f8fafc]` 继续展开为 `bg(rgb(0xf8fafc))`，验证已有能力未回退。
15. 静态 class 和动态 class 对新增尺寸语法的支持范围一致。
16. `#rrggbbaa`、`rgb(...)`、`rgba(...)` 静态与动态路径一致。
17. `debug-outline` 可编译，且 release 构建为空操作。
18. `rsx_strict!`、`rsx_permissive!`、`rsx_expand!` 可用。

## 推荐优先级

P0：

- [x] 修复 `font-bold` / `h1` / `h2` 与 GPUI 0.2 的兼容。
- [x] 保留现有 numeric class 的 `px(n)` 语义。
- [x] 支持尺寸属性族的 `px` / `rem` / `%` arbitrary value。
- [x] 支持 spacing 属性族的 `px` / `rem` arbitrary value。
- [x] 支持 `w-6/24` 这类分数尺寸。
- [x] 确认 `flex-1`、`min-w-0`、`min-h-0`、`gap-x-*`、`gap-y-*` 在静态和动态路径可用。
- [x] 增加非法 arbitrary value、非法分数和已知不兼容 class 的错误诊断。

P1：

- [x] 支持完整字体权重映射。
- [x] 让新增尺寸语法在静态 class 和动态 class 中完全一致。
- [x] 支持颜色 alpha 和 `rgb(...)` / `rgba(...)` arbitrary values。
- [x] 增加桌面三栏布局示例到 README / docs。
- [x] 支持更多 arbitrary value 组合，例如 `max-w-[32rem]`、`gap-x-[14px]`、`gap-y-[0.75rem]`。

P2：

- [x] 支持更多 Tailwind-like 子集。
- [x] 增加 debug layout class，例如 `debug-outline`。
- [x] 输出宏展开预览，用于排查布局问题。
- [x] 提供 strict / permissive mode，允许用户选择未知 class 是报错还是忽略。
- [x] 明确不引入 Tailwind spacing scale，numeric class 固定保持 `px(n)` 语义。
