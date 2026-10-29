---
title: 性能优化
description: GPUI-RSX 如何展开、哪些路径会产生运行时工作，以及如何保持 render 轻量。
---

GPUI-RSX 的设计目标是让常见路径在编译期生成普通 GPUI 代码。因此，性能优化的重点通常是：尽量保持标记静态，避免在 render 中产生不必要的分配和状态刷新。

## 心智模型

静态 RSX 会在宏执行期间展开：

```rust
rsx! {
    <div class="flex gap-4 bg-blue-500">
        {"Hello"}
    </div>
}
```

生成的表达式类似普通 GPUI builder 链：

```rust
div()
    .flex()
    .gap(px(4.0))
    .bg(rgb(0x3b82f6))
    .child("Hello")
```

静态标签、属性、class 字符串、颜色和很多布局工具类都会在应用运行前解析完成。

## 优先使用静态 Class 字符串

稳定 UI 结构使用字面量 `class`：

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4 bg-white border border-gray-200 rounded-md" />
}
```

字面量 class 没有运行时 parser 成本，也能让 strict 模式给出更有用的编译期诊断。

## 保持条件 Class 可编译期优化

如果 `class={...}` 内部的 `if` 或 `match` 每个分支都是字符串字面量，宏可以直接优化：

```rust
rsx! {
    <div class={if active { "bg-blue-500 text-white" } else { "bg-gray-100 text-gray-900" }} />
}

rsx! {
    <div class={match state {
        State::Ready => "border-green-500",
        State::Error => "border-red-500",
        _ => "border-gray-300",
    }} />
}
```

这个优化只作用于传给 `class` 的表达式。如果先把结果赋值给变量，宏看到的就是运行时表达式，会使用动态 matcher。

## format 模板的静态首尾

`class={format!("p-{} bg-red-500", n)}` 在编译期展开 `bg-red-500`，仅动态部分进入
运行时 matcher。原 `format!` 仍求值一次，保留命名参数与格式选项，因此仍会分配 String。
首个动态 token 之前、最后一个动态 token 之后的完整静态 class 可以优化。
动态 token 之间的静态 class 保留运行时处理，因为插值结果可能包含空格；应用顺序与副作用保持一致。

## 谨慎使用动态 Class

动态 class 适合 class 字符串确实必须运行时组装的情况：

```rust
let class_name = format!("w-[{}px] bg-{}", width, tone);

rsx! {
    <div class={class_name} />
}
```

这条路径会生成运行时 matcher 来处理 class token。它支持常见布局、间距、尺寸、颜色、透明度、排版、边框、任意长度和任意颜色，但这些仍然是在 render 期间执行的工作。

如果值本来就是 Rust 表达式，优先使用直接 GPUI 属性：

```rust
rsx! {
    <aside class="flex flex-col min-w-0" w={px(self.sidebar_width)} />
}
```

少量状态变体优先使用 `when` 或 `whenClass`：

```rust
rsx! {
    <button
        class="px-3 py-2 rounded-md"
        whenClass={(selected, "bg-blue-500 text-white")}
        whenClass={(!selected, "bg-gray-100 text-gray-900")}
    >
        {label.as_str()}
    </button>
}
```

## 共享动态 Class Runtime

默认情况下，每次使用动态 class 的宏调用都会生成自己的匹配 helper。调用点较多时，
可以在 crate 根生成一次：

```rust title="src/lib.rs"
gpui_rsx::rsx_runtime!();
```

在当前包的配置中启用：

```toml title="rsx.toml"
[options]
dynamic-runtime = "rsx_runtime"
```

生成的公开模块包含 `px` 与 `tailwind` 两个 helper，均作用于 GPUI 的
`StyleRefinement`，不同元素类型也复用同一份编译结果。`rsx!` 和 `rsx_tw!`
按间距刻度选择 helper。已有样式、每个元素的 `onUnknownClass` 回调和严格模式
行为保持一致；状态变体与主题 token 仍需使用静态字符串。
此设置减少编译工作，动态字符串在渲染时仍需解析。

库内部使用 `crate::rsx_runtime`；同一包的 integration test、example 和 bin
自动引用库中的模块，也支持自定义 `[lib].name`。仅含 bin 的包需要在每个使用
动态 class 的目标根部调用 `rsx_runtime!()`，包括独立的 integration test 和 example。
显式配置 `dynamic-runtime = "crate::rsx_runtime"` 总是使用当前目标；
`"::my_library::rsx_runtime"` 可直接指定已有模块。GPUI 使用其他依赖名时，
将路径传给宏：`gpui_rsx::rsx_runtime!(my_gpui);`。

0.10 中此优化为可选功能，未配置时保持原行为。lint 与 LSP 会在未配置共用 runtime
时为动态 class 给出 information 级建议。

本机 macOS ARM64 / Rust 1.99.1 测量：50 个公开函数各含一个 `class={classes}`，
使用仓库的 GPUI stub。预先编译依赖，关闭增量编译，每次仅触碰应用源码后测量
一次对应命令：

| 项目 | 内联 helper | 共用 runtime |
| --- | --- | --- |
| `cargo check` | 2.180 s | 0.259 s |
| Debug 构建 | 4.112 s | 0.386 s |
| Release 构建 | 12.841 s | 0.885 s |
| Debug `.rlib` | 11.86 MB | 2.06 MB |

此比较仅测生成代码的编译成本。真实应用的冷构建还包含 GPUI 等依赖，整体提升比例会不同。

## 避免 Render 循环里的分配

Render 可能频繁执行。每次 render 的分配都应当是有意的：

```rust
rsx! {
    <ul>
        {for item in &self.items {
            <li>{item.name.as_str()}</li>
        }}
    </ul>
}
```

除非确实需要，避免在循环里 clone 大字符串或重复构造格式化 class 名。

为可见文本做少量 `format!` 是正常的：

```rust
rsx! {
    <span>{format!("Count: {}", self.count)}</span>
}
```

重复标签优先使用借用字符串或预先计算好的显示文本。

## 合并状态通知

状态更新通常应只触发一次 render 通知：

```rust
onClick={cx.listener(|view, _, _window, cx| {
    view.count += 1;
    view.last_update = Instant::now();
    view.recompute_summary();
    cx.notify();
})}
```

避免每改一个字段就调用一次 `cx.notify()`。

## 理解 ID 和 Key 的成本

只有需要 GPUI 有状态身份的元素才会生成自动 ID。常见触发项包括 `onClick`、`onHover`、
`onDrag`、`onAuxClick`、`onA11yAction`、`active`、`activeClass`、`groupActive`、
`tooltip`、`hoverableTooltip`、`tooltipShowDelay`、`focusable`、`role`、`aria*`、
`trackScroll`、`overflowScroll`、`overflowXScroll`、`overflowYScroll` 和静态
`overflow-scroll` class 变体。

`hover`、`hoverClass`、`focus`、`focusClass`、`group`、`groupHover` 和 `scrollbarWidth`
走样式 refinement 路径，本身不会触发自动 ID 注入。

普通布局元素不会因为带了 `key={...}` 就变成有状态元素：

```rust
rsx! {
    <div key={task.id} class="flex items-center">
        {task.title.as_str()}
    </div>
}
```

上面的 `key` 会被忽略，因为元素不需要 ID。循环里的重复有状态元素才需要添加 `key`：

```rust
rsx! {
    <button key={task.id} onClick={handler}>
        {task.title.as_str()}
    </button>
}
```

字面量 key 使用 `concat!`。具体类型的动态整数（不超过 64 位，含 `usize` / `isize`）
使用 GPUI 的 `(&'static str, u64)` ID，无需格式化或分配字符串。其他 `Display` 值保留
字符串格式化路径，`u128` / `i128` 也走此路径以避免截断。整数引用和泛型 `T: Display`
也可能走格式化路径。表达式借用求值一次，不会移动 String key；大型有状态列表优先
使用 `key={task.id}` 这样的整数值。

## 保持 RSX Block 易读

很大的宏输入会增加编译期工作，也会让编译错误更难定位。按真实 UI 概念拆分视图：

```rust
fn render_sidebar(&self) -> impl IntoElement {
    rsx! {
        <aside class="flex flex-col w-[280px] min-w-[280px] border-r border-gray-200">
            {self.render_sidebar_header()}
            {self.render_project_list()}
        </aside>
    }
}
```

这主要是可维护性优化，也能让增量修改更小。

## 查看生成代码

使用 `rsx_expand!` 检查 class 是否保持静态，还是进入了动态 matcher：

```rust
let preview = gpui_rsx::rsx_expand! {
    <div class={if active { "flex gap-4" } else { "block" }} />
};
```

完整类型验证仍以 `cargo check` 为准。

## 验证性能敏感改动

### 0.9 开发测量

macOS ARM64、Rust 1.99.1 下的短时 Criterion 测量（10 个样本、预热和测量各 1 秒）
得到以下斜率估计值：

| 场景 | 优化前 | 优化后 |
| --- | --- | --- |
| 静态 dashboard 宏展开 | 66.8 µs | 66.7 µs |
| 单个动态 class 属性 | 489 µs | 477 µs |
| 同次宏调用内四个动态 class 属性 | 1.99 ms | 0.495 ms |
| 分析 80 个宏 / 复用 79 个未修改宏体 | 1.00 ms（完整分析） | 223 µs（复用） |

这些是本地微基准，不代表整个项目的冷编译时间。复用分析的 95% 置信区间约为 199–268 µs。
早期增量分析仍扫描 Rust token，只跳过未修改 RSX 宏体的解析。动态 helper 在单次宏调用内共享，
各属性仍保留独立的表达式求值和未知 class 回调。

```bash
cargo bench -p gpui-rsx-tools --bench analysis --no-default-features -- \
  --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo bench -p gpui-rsx-core --bench expansion -- \
  --save-baseline before09 --sample-size 10 --warm-up-time 1 --measurement-time 1
# 应用 helper 共享改动后，使用相同的 benchmark 输入：
cargo bench -p gpui-rsx-core --bench expansion -- \
  --baseline before09 --sample-size 10 --warm-up-time 1 --measurement-time 1
```

比较两个版本时使用相同的 benchmark 代码，并保留 Criterion 报告目录。发布性能结论前，
应在空闲机器上进行更长时间的测量。

仓库包含 class 解析/codegen 的 benchmark 目标：

```bash
cargo bench --bench class_performance --no-run
```

真实 GPUI 集成使用 demo crate 检查：

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
```

文档示例和导航用文档站构建验证：

```bash
cd docs
bun run build
```

## 编辑器与 matcher 实测（2026-10-29）

本机 macOS ARM64 上，包含 50 个宏的 5000 行文件完整分析约 **1.12ms**，修改单个平衡
宏后增量分析约 **0.15ms**。样本由普通注释行与小型 RSX 宏体组成，不代表任意 Rust 文件的
延迟上限。复现：`cargo bench -p gpui-rsx-tools --bench analysis -- 5000_lines`。
宏外或结构性修改会完整扫描；不含 `rsx` 的文件跳过分析，LSP 会合并队列中连续的编辑。
语义高亮分类和颜色缓存在分析结果中，后续请求直接复用。

常用运行时 matcher 改为按首字节分派。10 个短时 stub 样本中，简单动态 class 约
16→14ns，复杂组合约 129→89ns；单独的 `gap-4` 从约 9ns 回退至 11ns，数值回退路径
保持约 13ns。这是小型合成基准，不代表所有输入均提速。复现：
`cargo bench --bench class_performance -- dynamic_`。
