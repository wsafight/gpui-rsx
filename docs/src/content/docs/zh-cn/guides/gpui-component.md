---
title: gpui-component
description: 配合 gpui-component builder 使用 GPUI-RSX，并避免重复 GPUI crate 实例。
---

只要组件 builder 和应用的 GPUI 依赖解析到同一个 GPUI source，`gpui-component` 就可以与 GPUI-RSX 一起使用。

## 依赖形态

demo 使用：

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit", "runtime_shaders", "wayland", "x11"] }
gpui-rsx = { path = ".." }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }
```

应用项目应提交 `Cargo.lock`，避免解析到的 Zed revision 意外漂移。

## 用 `base` 指定构造器

GPUI-RSX 可以从标签名推导简单构造器，但组件 builder 往往需要显式构造器。此时使用宏专用的 `base` 属性：

```rust
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::Sizable as _;

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
`gpui_component::tab::Tab::aria_label` 是组件 builder；直接写成 RSX 属性会按 GPUI 的
stateful `aria_label` 分类。应显式保留组件方法链：

```rust
<Tab base={Tab::new().label("Overview").aria_label("Overview tab")} underline />
```

当前主线内部已使用新的 `gpui-base` crate。优先使用 `gpui-component` facade re-export 的
`StyledExt`、`Edges` 和 input state 类型，让两者保持在同一个 lockfile revision。

## 显式导入扩展 Trait

很多 `gpui-component` 方法来自扩展 trait。建议在组件使用处显式导入：

```rust
use gpui_component::button::ButtonVariants as _;
use gpui_component::Sizable as _;
```

这样宏生成的方法链可以正常通过类型检查，缺少 trait 导入时也更容易定位。

## 兼容性检查

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui
```

如果组件类型不匹配，先查看依赖树。大多数问题来自重复 GPUI crate 实例。
