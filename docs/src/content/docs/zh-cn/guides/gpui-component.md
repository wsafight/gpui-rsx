---
title: gpui-component
description: 配合 gpui-component builder 使用 GPUI-RSX，并避免重复 GPUI crate 实例。
---

只要组件 builder 和应用的 GPUI 依赖解析到同一个 GPUI source，`gpui-component` 就可以与 GPUI-RSX 一起使用。

## 依赖形态

demo 使用：

```toml
[dependencies]
gpui = { package = "gpui-kit", version = "=0.8.1" }
gpui-rsx = { path = ".." }
```

Kit 统一提供配套的 GPUI 快照、组件和资源。应用应提交 lockfile；调用 `gpui::init(cx)` 初始化，再用 `gpui::open_window(...)` 打开窗口。

## 用 `base` 指定构造器

GPUI-RSX 可以从标签名推导简单构造器，但组件 builder 往往需要显式构造器。此时使用宏专用的 `base` 属性：

```rust
use gpui::component::button::{Button, ButtonVariants as _};
use gpui::component::Sizable as _;

rsx! {
    <Button
        base={Button::new("save")}
        label={"Save"}
        small
        primary
    />
}
```

`base` 会替换推导出的构造器，不会生成 `.base(...)` 方法调用。

组件方法名与 GPUI stateful 属性重名时也应使用 `base`。例如
`gpui::component::tab::Tab::aria_label` 是组件 builder；直接写成 RSX 属性会按 GPUI 的
stateful `aria_label` 分类。应显式保留组件方法链：

```rust
<Tab base={Tab::new().label("Overview").aria_label("Overview tab")} underline />
```

Kit 通过 `gpui::component` 导出带样式的组件，通过 `gpui::base` 导出基础行为。
可序列化的 `Edges` helper 从 `gpui::base` 或 `gpui::component` 导入；各层共享同一套渲染引擎类型。

## 显式导入扩展 Trait

很多 `gpui-component` 方法来自扩展 trait。建议在组件使用处显式导入：

```rust
use gpui::component::button::ButtonVariants as _;
use gpui::component::Sizable as _;
```

这样宏生成的方法链可以正常通过类型检查，缺少 trait 导入时也更容易定位。

## 兼容性检查

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
cargo xtask check-gpui
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui-pre
```

如果组件类型不匹配，先查看依赖树。大多数问题来自重复 GPUI crate 实例。
