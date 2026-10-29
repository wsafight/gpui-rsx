---
title: 迁移指南
description: 从手写 GPUI builder 链迁移，升级工具 API，并对齐 GPUI Kit 兼容目标。
---


## 从 0.9 升级到 0.10

根 Fragment 中包含 `for` 循环或 `{...spread}` 时，现在
展开为 `Vec<AnyElement>`；此前会生成装着迭代器的 `vec!`，混合子节点时无法编译。用
`#[deprecated]` 标记的组件现在在使用处给出警告，而不是在其生成代码内部报警。

类型为 `Rc<dyn Fn...>` 的 `#[component]` 属性现在与 `Box<dyn Fn...>` 属性及 GPUI 事件 setter
一样接收 `impl Fn... + 'static`，`on_press={|_, _, _| ...}` 可自动推导参数。原先传入已有
`Rc` 回调的代码改为转发：

```rust
<Row on_select={{ let on_select = on_select.clone(); move |item, w, cx| on_select(item, w, cx) }} />
```

组件构造函数与 `base` 表达式仅在同时实现 `InteractiveElement` 和 `Element` 时自动注入 ID。
返回原生元素的函数仍获得自动 ID；gpui-component 的 `Button` 和名为 `on_click` 的回调属性
保留自身 setter。泛型与不透明返回类型须暴露所需约束，详见 [ID 与 Key](/gpui-rsx/zh-cn/usage/ids/)。

宽松模式下，静态 CSS 专属 class（`z-*`、`space-x-*`、`space-y-*`、`divide-*`、
`transition*`、`duration-*`、`ease-*`、`delay-*`、`animate-*`）现在会警告并忽略，不再透传
为扩展方法。如果扩展 trait 实现了这些方法，将 `class="z-10"` 改为显式 flag `z_10`，
或在 `base={...}` 中调用。严格模式下这些 class 会报错。

`gpui-rsx-tools` 集成方需要注意的破坏性变更：

- `Diagnostic::severity` 由 `String` 改为 `DiagnosticSeverity` 枚举，JSON 输出保持不变（`"error"`、
  `"warning"`、`"information"`）。需要字符串时使用 `severity.as_str()` 或 `to_string()`；该枚举为
  `#[non_exhaustive]`，匹配时需要兜底分支。
- 整文件分析（`analyze_rust`、`analyze_rust_document`、`update_rust_document`）现在包含嵌套在其他
  宏内部的 RSX 宏。`macro_calls` 仍只返回互不重叠的顶层调用；需要全部调用时使用
  `nested_macro_calls`，用 `call_at` 查找某个偏移处最内层的调用。
- `report_body` 一次解析即返回诊断、展开结果和预览。`BodyReport` 为 `#[non_exhaustive]`，请读取
  字段而不要自行构造。它的展开结果与新增的 `expand_summary` 一样省略动态 class helper，`expand`
  仍输出完整代码。

CLI 会以退出码 2 拒绝命令不使用的选项（例如 `lint --port`），`lint` 会检查显式指定的任意扩展名
文件。工具会自行识别 `gpui-rsx` 的 `tailwind-scale` feature，启用该 feature 的项目不再需要
`--tailwind`。

`RustAnalysis` 新增 `components` 与 `class_tokens` 字段。使用 `RustAnalysis::default()`
与字段更新，或直接调用 `analyze_rust_document`，避免逐项列出结构体字段。class token 偏移
仍是 UTF-8 字节，可复用分类与颜色结果。`update_rust_document` 的 mode 必须与上次一致；
配置变更应完整分析。

动态整数 key 改为 GPUI 命名整数表示。如其他代码读取或持久化旧的格式化 ID，请显式指定
`id`；key 仍依赖源码位置，不保证编辑前后身份一致。

`convert(source)` 保留原像素宏体输出。要保留 Tailwind 数值，将
`ConversionOptions::default().tailwind` 设为 `true` 并调用 `convert_with_options`，
此时 `rsx` 字段包含完整 `rsx_tw!` 表达式。WASM 新增 `analyze_with_theme` 与
`convert_with_options`，原有导出保留。

转换器现在原样保留 `{items}`、`{item.title}` 等简单表达式，不再给出需要手动转换的诊断。
这里只检查语法，仍须核对 Rust 类型和所有权。集合子节点可能需要 `{...items}` 或 `{for ...}`；
保留 `{items}` 不代表已把 JavaScript 的渲染语义转换为 GPUI 子节点。调用、运算符、索引、
可选链和事件处理仍会给出人工检查诊断。

## 从 0.8 升级到 0.9

0.9 版本以 GPUI Kit 0.8.1 为兼容目标，默认间距仍按像素解释；现有 RSX
语法和自动 ID 行为保持一致。工具集成应使用 `gpui-rsx-core` 根导出或 `syntax`
模块，将结构体字面量改为 `ClassMode::builder()`、`Theme::builder()` 和
`MacroOptions::new().with_*()`。多次传递 `ClassMode` 时显式 `clone()`，Theme
通过 `Arc<Theme>` 共享；语法、class 结果和 token 枚举的匹配需要兜底分支。

`gpui-rsx-tools` 默认不再启用 SWC 转换器。调用转换器的库需要显式开启
`features = ["convert"]`；CLI 和 WASM Playground 已开启。格式化、lint 和预览
无需 SWC。

`rsx.toml` 新增 `text-size`、`line-height`、`opacity` 和 `shadow` 分区。
透明度使用 `"0.6"` 或 `"60%"`，阴影使用 `{ expr = "..." }`。重复键以及冲突的
`text-size-*` class 会报告配置错误。`cargo rsx theme check/list/explain` 支持
`--json`，校验失败退出码为 1，命令用法错误为 2。

LSP 声明增量同步（`change: 2`）与语义 token；仍支持全文替换，范围编辑按
UTF-16 位置依次应用，未修改的宏复用诊断结果。同一次 RSX 调用中的动态 class
共享一个 helper，各属性仍独立求值并调用自己的未知 class 处理器。

## Zed Git 迁移到 GPUI Kit 0.8

用 Kit 替换单独的 Zed Git `gpui`、`gpui_platform` 和 Git `gpui-component`
依赖；保留 `gpui` crate 名称，供生成代码使用：

```toml
[dependencies]
gpui = { package = "gpui-kit", version = "=0.8.1" }
gpui-rsx = "0.10.0"
```

Kit 0.8.1 固定 `gpui-pre = 0.3.10`。从 `gpui::component` 导入带样式控件，
从 `gpui::base` 导入基础行为。将 `gpui_platform::application()` 改为
`gpui::application()`；打开窗口前初始化 Kit，并挂载其根视图：

```rust
gpui::application().with_assets(gpui::assets::Assets).run(|cx| {
    gpui::init(cx);
    gpui::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
        cx.new(|_| MyView)
    }).expect("failed to open window");
});
```

依赖变更后重新生成并提交应用的 lockfile。仓库 demo 使用 Rust 1.98.1；宏与 core
的 MSRV 仍为 Rust 1.88。合入前检查 `cargo xtask check-gpui` 和真实 demo bins。
GPUI Fast 是单独的可选后端，不在当前兼容性目标中。

`gpui-component` 主题预设现在生成 `cx.theme().colors.<field>`，避免 `list`
颜色与 `Theme.list` 配置字段重名。使用预设时导入 `gpui::component::ActiveTheme`。

下文 0.5 到 0.6 的章节记录历史 Zed Git 目标；新项目请采用当前的
[兼容矩阵](/gpui-rsx/zh-cn/compatibility/)。

## 间距刻度、状态前缀与组件迁移

数值间距和尺寸默认仍按像素解释：`gap-4` 是 4px。使用 `rsx_tw!`、开启 Cargo
feature `tailwind-scale`，或在 crate 的 `Cargo.toml` 旁创建 `rsx.toml`，即可切换为
Tailwind 的每单位 4px：

```toml
[options]
spacing-scale = "tailwind"
```

| Class | 默认 | Tailwind 刻度 |
| --- | --- | --- |
| `gap-4` | 4px | 16px |
| `p-2.5` | 2.5px | 10px |
| `w-64` | 64px | 256px |
| `gap-[14px]` | 14px | 14px |

切换时要保持布局不变，可改成显式像素（`gap-4` → `gap-[4px]`），或将数值除以四
（`gap-4` → `gap-1`）。检查静态字面量、条件分支和动态字符串。类型化属性
（`gap={px(4.0)}`）、分数、任意值、颜色、透明度及 GPUI 预设（`rounded-md`、
`border-2`）不受刻度影响。库项目建议用 crate 局部的 `rsx.toml`，因为 Cargo feature
会在整个依赖图中合并。

静态 class 支持 `hover:`、`focus:`、`focus-visible:`、`active:`、`group-hover:` 和
`group-active:`，也支持具名 group。有状态的前缀会自动注入 ID，循环内需要稳定的 key。
响应式和 `dark:` 前缀在宽松模式下警告，严格模式下报错。动态字符串不支持状态前缀和
主题令牌。使用 `onUnknownClass={|class: &str| ...}` 可替换严格模式默认的 panic；
`dynamic-class-report` feature 会将 debug 提示汇总到进程退出时输出。

`#[component]` 为具名字段结构体生成 props builder。普通字段必填，
`#[prop(default)]` / `#[prop(default = expr)]` 可选，`#[children]` 字段收集子节点。
第三方组件仍可使用 `base={...}` 传入构造器。

`rsx.toml` 中的 `[colors]`、`[radius]`、`[spacing]` 定义主题令牌。
`[options] preset = "gpui-component"` 可复用语义颜色；使用时导入主题扩展 trait，
并确保 `cx` 在作用域内。`{ expr = "cx.theme().colors.background" }` 在渲染时读取当前主题，
十六进制颜色字面量则保持固定。

## 从手写 GPUI 迁移到 RSX

GPUI-RSX 可以渐进采用。手写 GPUI builder 和 RSX 输出都是普通 GPUI 元素，可以共存。

### 元素和属性

迁移前：

```rust
div()
    .flex()
    .flex_col()
    .gap(px(16.0))
    .bg(rgb(0x3b82f6))
```

迁移后：

```rust
rsx! {
    <div class="flex flex-col gap-[16px] bg-blue-500" />
}
```

如果源码里已经有类型化的 Rust 值，使用直接属性：

```rust
rsx! {
    <div w={px(width)} bg={self.background_color()} />
}
```

### 子节点

迁移前：

```rust
div()
    .child("Hello")
    .child(div().child("World"))
    .child(button().child("Click"))
```

迁移后：

```rust
rsx! {
    <div>
        "Hello"
        <span>{"World"}</span>
        <button>{"Click"}</button>
    </div>
}
```

### 事件

迁移前：

```rust
button()
    .on_click(cx.listener(|view, _event, _window, cx| {
        view.count += 1;
        cx.notify();
    }))
    .child("Click")
```

迁移后：

```rust
rsx! {
    <button onClick={cx.listener(|view, _event, _window, cx| {
        view.count += 1;
        cx.notify();
    })}>
        {"Click"}
    </button>
}
```

### 列表

迁移前：

```rust
let mut list = div().flex().flex_col();

for item in &self.items {
    list = list.child(div().child(item.name.clone()));
}

list
```

迁移后：

```rust
rsx! {
    <div class="flex flex-col">
        {for item in &self.items {
            <div>{item.name.as_str()}</div>
        }}
    </div>
}
```

如果行是有状态元素，添加 `key={...}`：

```rust
rsx! {
    <div class="flex flex-col">
        {for item in &self.items {
            <button key={item.id} onClick={handler}>
                {item.name.as_str()}
            </button>
        }}
    </div>
}
```

### 条件渲染

子节点不同，用 Rust 条件：

```rust
rsx! {
    <div>
        {if show_message {
            rsx! { <span>{"Message visible"}</span> }
        } else {
            rsx! { <span>{"Message hidden"}</span> }
        }}
    </div>
}
```

同一个元素只需要可选 builder 调用时，用 `when`：

```rust
rsx! {
    <div when={(show_message, |el| el.child("Message visible"))} />
}
```

## 0.5.x / crates.io GPUI 0.2.2 到 0.6.0 / Zed Git GPUI

GPUI-RSX `0.6.x` 目标是 Zed 仓库中的 GPUI，而不是 crates.io 的 `gpui = "0.2.2"` 包。

Zed git 依赖仍会把 package version 显示为 `gpui v0.2.2`，但 API 面与 crates.io release 不同。这会影响 `flex_grow_1`、`flex_shrink_1`、对齐 helper、桌面尺寸工具类和新的启动路径等生成调用。

### 替换依赖

迁移前：

```toml
[dependencies]
gpui = "0.2.2"
gpui-rsx = "0.5"
```

迁移后：

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit", "runtime_shaders", "wayland", "x11"] }
gpui-rsx = "0.6"
```

应用项目应提交 `Cargo.lock`，固定实际解析到的 Zed revision。

如果应用也依赖 `gpui-component`，直接依赖的 `gpui` 和 `gpui_platform` 应与 `gpui-component` 解析到的 source 和 revision 保持一致。混用 bare Zed git 和另一份 pinned `rev` 容易生成重复 GPUI crate，导致组件类型不兼容。

### 更新应用启动方式

迁移前：

```rust
Application::new().run(|cx: &mut App| {
    cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| MyView));
});
```

迁移后：

```rust
use gpui_platform::application;

application().run(|cx: &mut App| {
    cx.open_window(WindowOptions::default(), |_, cx| {
        cx.new(|_| MyView)
    }).unwrap();
    cx.activate(true);
});
```

### 验证依赖图

仓库 demo crate 是一个已知可用的集成目标：

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui-pre
cargo xtask check-gpui
```

如果 `cargo tree` 显示 `git+https://github.com/zed-industries/zed#...`，即使 package version 是 `0.2.2`，项目也在使用 Zed git GPUI。

## 渐进迁移计划

1. 在一个 view module 中添加 `gpui_rsx::rsx` import。
2. 先转换小的静态子树。
3. 静态结构通过编译后，再转换列表和重复行。
4. 只给需要有状态 ID 的重复元素添加 `key={...}`。
5. 每个有意义的步骤后运行 `cargo check`。

混合写法是可以接受的：

```rust
rsx! {
    <div>
        {self.legacy_gpui_builder()}
        <section class="flex flex-col gap-4">
            {self.render_new_rsx_panel()}
        </section>
    </div>
}
```

## 常见迁移模式

| 手写 GPUI | GPUI-RSX |
| --- | --- |
| `div().child("Hello")` | `rsx! { <div>{"Hello"}</div> }` |
| `div().child(format!("Count: {count}"))` | `rsx! { <div>{format!("Count: {count}")}</div> }` |
| `.w(px(200.0)).h(px(100.0))` | `<div w={px(200.0)} h={px(100.0)} />` |
| `.flex().gap(px(4.0))` | `<div class="flex gap-4" />` |
| 带参数的 builder 构造器 | `<Button base={Button::new("id")} />` |

## 检查清单

- [ ] 当前 Kit 正式版本命名为 `gpui`，并使用配套快照。
- [ ] 启动使用 `gpui::application()`、`gpui::init(cx)` 和 `gpui::open_window(...)`。
- [ ] 应用项目提交了 `Cargo.lock`。
- [ ] 通过 `gpui::component` 导入组件，共享同一份 GPUI 实例。
- [ ] 新 RSX 模块引入了 `gpui_rsx::rsx`。
- [ ] 重复的有状态元素有 `key={...}` 或显式 `id={...}`。
- [ ] 迁移后的视图通过 `cargo check`。
- [ ] 状态变化后仍然调用 `cx.notify()`。
