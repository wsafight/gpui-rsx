---
title: 问题排查
description: 使用 GPUI-RSX 时常见的解析、类型、运行时、样式和依赖问题。
---

## 解析错误

### 闭合标签不匹配

错误示例：

```text
Closing tag `</span>` does not match opening tag `<div>`.
```

修正标签嵌套：

```rust
// 错误
rsx! {
    <div>
        <span>{"Text"}</div>
    </span>
}

// 正确
rsx! {
    <div>
        <span>{"Text"}</span>
    </div>
}
```

### 标签或 Fragment 未闭合

补上缺失的闭合标签：

```rust
// 错误
rsx! {
    <div>{"Content"}
}

// 正确
rsx! {
    <div>{"Content"}</div>
}
```

### 子节点 token 不符合预期

裸 ident 不是子节点。表达式需要放在花括号里：

```rust
// 错误
rsx! { <div>count</div> }

// 正确
rsx! { <div>{count}</div> }
```

字符串字面量可以直接写：

```rust
rsx! { <div>"Count"</div> }
```

### For 循环体缺少花括号

循环体必须是花括号包住的 RSX body：

```rust
// 错误
rsx! {
    <ul>
        {for item in items
            <li>{item}</li>
        }
    </ul>
}

// 正确
rsx! {
    <ul>
        {for item in items {
            <li>{item}</li>
        }}
    </ul>
}
```

## Class 错误

### Strict 模式下静态 class 不支持

`rsx_strict!` 会拒绝未知静态 class：

```rust
rsx_strict! {
    <div class="md:flex" />
}
```

改用受支持的 class、直接 GPUI 方法、`when`，或在确实希望忽略未知 class 时使用 `rsx_permissive!`。

### 任意长度无效

任意长度 class 必须使用受支持单位和有限值：

```rust
// 正确
rsx! { <div class="w-[280px] max-w-[37.5%] gap-[14px]" /> }
```

尺寸支持 `px`、`rem`、百分比和分数。间距支持 `px`、`rem` 等确定长度；百分比间距会被拒绝。

### 任意颜色无效

使用受支持的 hex、RGB 或 RGBA 形式：

```rust
rsx! {
    <div class="bg-[#ff0000] text-[rgb(15,23,42)] border-[rgba(15,23,42,0.35)]" />
}
```

RGBA alpha 必须在 `0.0..=1.0` 范围内。

### 动态 Class 被忽略

动态 class 使用运行时 matcher。permissive 模式下，不支持的 token 会被忽略；debug 构建里同一调用点只警告一次。

如果某个动态样式必须生效，优先改成：

```rust
rsx! {
    <div
        class="flex"
        w={px(width)}
        whenClass={(active, "bg-blue-500 text-white")}
    />
}
```

需要在运行时遇到未知 token 直接失败时，使用 `rsx_strict!`。

### 状态 Class 属性使用了动态值

`hoverClass`、`focusClass` 和 `activeClass` 是编译期 helper，只接受字符串字面量：

```rust
// 错误
let classes = "bg-blue-600";
rsx! { <button hoverClass={classes} /> }

// 正确
rsx! { <button hoverClass="bg-blue-600" /> }
```

如果 refinement 需要 Rust 逻辑，直接使用 `hover`、`focus` 或 `active`：

```rust
rsx! {
    <button hover={|style| style.bg(rgb(0x2563eb))} />
}
```

### 状态 Class 包含元素级 Class

状态 class 属性接收的是 GPUI `StyleRefinement`，因此 `overflow-scroll` 或
`debug-outline` 这类元素级 class 不能放进去：

```rust
// 错误
rsx! { <button activeClass="overflow-scroll" /> }
```

始终启用的元素行为放在主 `class`：

```rust
rsx! {
    <button
        class="overflow-scroll"
        activeClass="opacity-75"
    />
}
```

需要条件启用元素级行为时，提供显式 `id`，再通过 `when` 调用 GPUI：

```rust
rsx! {
    <button
        id="results"
        when={(is_scrollable, |el| el.overflow_scroll())}
        activeClass="opacity-75"
    />
}
```

## 条件属性错误

### `when`、`whenSome` 或 `whenClass` 的 Tuple 形状错误

这些属性都需要恰好两个 tuple 值：

```rust
rsx! {
    <div when={(active, |el| el.bg(rgb(0x3b82f6)))} />
    <div whenSome={(width, |el, w| el.w(px(w)))} />
    <div whenClass={(active, "bg-blue-500 text-white")} />
}
```

### `whenClass` 必须使用字符串字面量

下面写法无效：

```rust
let classes = "bg-blue-500";
rsx! {
    <div whenClass={(active, classes)} />
}
```

动态样式请使用 `when`，或把字面量直接写在 `whenClass` 中。

### `whenClass` 拒绝有状态 Class

`overflow-scroll`、`overflow-x-scroll`、`overflow-y-scroll` 需要 ID 语义。请使用 `when` 明确调用 GPUI 方法：

```rust
rsx! {
    <div when={(scrollable, |el| el.overflow_scroll())} />
}
```

## ID 和循环错误

### 循环中的有状态元素缺少 `id` 或 `key`

重复的有状态元素需要唯一 ID：

```rust
// 错误
rsx! {
    <ul>
        {for item in &self.items {
            <li onClick={handler}>{item.name.as_str()}</li>
        }}
    </ul>
}

// 正确
rsx! {
    <ul>
        {for item in &self.items {
            <li key={item.id} onClick={handler}>{item.name.as_str()}</li>
        }}
    </ul>
}
```

`key` 只在元素需要 ID 时生效。普通布局行上的 `key` 会被忽略。

### 重构后自动 ID 变化

自动 ID 包含文件、行号和列号。身份必须在移动源码后仍保持稳定时，使用显式 `id`：

```rust
rsx! {
    <button id="settings-save" onClick={handler}>
        {"Save"}
    </button>
}
```

## 类型错误

### `IntoElement` 未实现

子节点必须是有效 GPUI child。显式处理 `Option`：

```rust
rsx! {
    <div>{self.optional_text.as_deref().unwrap_or("")}</div>
}
```

或做条件渲染：

```rust
rsx! {
    <div>
        {if let Some(text) = &self.optional_text {
            rsx! { <span>{text.as_str()}</span> }
        } else {
            rsx! { <span /> }
        }}
    </div>
}
```

### 无法从 `self` 中 move

使用引用迭代：

```rust
rsx! {
    <ul>
        {for item in &self.items {
            <li>{item.name.as_str()}</li>
        }}
    </ul>
}
```

### Fragment 根元素类型混合

Fragment 会展开为 `vec![...]`。如果根元素具体类型不同，包一层共同父元素或做类型擦除：

```rust
rsx! {
    <div>
        {self.render_header()}
        {self.render_custom_button()}
    </div>
}
```

## 运行时问题

### UI 没有更新

修改 view 状态后调用一次 `cx.notify()`：

```rust
onClick={cx.listener(|view, _, _window, cx| {
    view.count += 1;
    cx.notify();
})}
```

同时确认 `render` 返回的是 `rsx!` 表达式，末尾不要加分号：

```rust
fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    rsx! { <div>{"Content"}</div> }
}
```

### `dx serve --hot-patch` 没有响应 RSX 修改

Dioxus CLI 0.7.10 根据宏路径的最后一段识别 `rsx!` 和 `render!`，因此
`gpui_rsx::rsx!` 也会被识别为 Dioxus 模板。它的模板解析器不接受 GPUI-RSX
的 HTML 风格语法，而 CLI 在解析失败后直接跳过，没有回退到 Rust 编译。
如果只修改这个宏的内容，GPUI 的 Subsecond 集成就收不到 Rust 补丁。

可以通过本地别名，让修改进入 Rust 编译路径：

```rust
use gpui::prelude::*;
use gpui_rsx::rsx as gpui_view;

fn content() -> impl gpui::IntoElement {
    gpui_view! { <div>{"修改这段文字"}</div> }
}
```

修改 import 和调用位置后，重新启动 `dx serve`。保持热更新开启：CLI 0.7.10
的 `--hot-reload false` 会关闭文件变更处理，也就不会自动生成 Rust 补丁。
别名保留原宏的 class 检查模式。默认 `rsx!` 使用宽松模式；需要严格检查时，
可以改为 `use gpui_rsx::rsx_strict as gpui_view;`。

别名解决的是变更检测。应用补丁还需要使用支持 Subsecond 的 GPUI 版本，
并在应用中定义转发到 `gpui/hot-reload` 的 feature，通过
`dx serve --hot-patch --hot-reload true --features hotreload` 启用。先检查 `dx --version`，
再区分“没有触发编译”和“补丁已生成但窗口没有更新”，这属于两个不同阶段。

独立夹具 `tests/fixtures/hot-reload/` 固定使用 GPUI Pre 0.3.11 和
Dioxus 0.7.10，其 README 提供解析器测试以及原生 builder、原始宏和别名
的对照步骤。它与 demo 的已验证依赖组合分开维护。
也可以使用现有的[构建并重启流程](../performance/#测量开发重载)，
它通过重启进程应用修改，每次重启都会重置应用状态。

### 事件没有触发

检查 handler 签名是否符合对应 GPUI 方法要求。重复的有状态事件目标还需要 `key` 或显式 `id`。

### `groupDragOver` 被拒绝

`groupDragOver` 不作为 RSX 属性暴露，因为 GPUI 需要在调用点明确拖拽数据类型：

```rust
// 错误
rsx! {
    <div groupDragOver={("items", |style| style.opacity(0.75))} />
}
```

通过 `when` 或 `base` 直接调用 GPUI，这样可以显式写出类型参数：

```rust
rsx! {
    <div
        when={(drag_enabled, |el| {
            el.group_drag_over::<MyDragData>("items", |style| style.opacity(0.75))
        })}
    />
}
```

### 样式没有生效

使用 `rsx_expand!` 查看生成的方法：

```rust
let preview = gpui_rsx::rsx_expand! {
    <div class="flex gap-4 bg-blue-500" />
};
```

注意 GPUI-RSX 不是 Tailwind CSS。`hover:bg-blue-500` 这类状态 variant 会映射为 GPUI 的 `.hover(...)`，但 `md:`、`dark:` 等响应式与媒体 variant 在 GPUI 中没有对应能力，请自行计算条件并使用 `when`。

## 依赖问题

### 出现重复 GPUI Crate

如果 `gpui-component` 类型和应用类型不匹配，先检查依赖树：

```bash
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui-pre
```

采用[兼容性说明](/gpui-rsx/zh-cn/compatibility/)中的单一 Kit 依赖，通过 `gpui::component` 导入组件。仓库内可用 `cargo xtask check-gpui` 检查配套快照和共享的 GPUI 类型。

### 更新依赖后 Demo 检查失败

运行：

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
```

如果 `--locked` 因 lockfile 需要变化而失败，需要明确判断是否真的要移动 GPUI 目标快照版本。不要在应用兼容性检查中随意更新 lockfile。

## 获取帮助

提交 issue 时请包含：

- GPUI-RSX 版本。
- `cargo tree -i gpui-pre` 中解析到的 GPUI 快照版本和 source。
- 最小 RSX 片段。
- 完整编译错误或运行时现象。
- 问题是否能在 `demo/` crate 中复现。
