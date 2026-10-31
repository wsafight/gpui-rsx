---
title: 语法参考
description: 元素、属性、子节点、Fragment、条件渲染和特殊 GPUI 标签。
---

本页说明 RSX 语法。Class 工具类范围见 [Class 处理](/gpui-rsx/zh-cn/usage/class/)，生成 ID 的规则见 [ID 与 Key](/gpui-rsx/zh-cn/usage/ids/)。

## 元素

RSX 元素使用 HTML-like 结构：

```rust
rsx! {
    <tag attr value={expr}>
        {child_expr}
    </tag>
}
```

常见 HTML-like 标签会映射为 `div()`，因为 GPUI 并没有浏览器 DOM 节点：

| 分组 | 标签 |
| --- | --- |
| 结构 | `div`, `span`, `section`, `article`, `header`, `footer`, `main`, `nav`, `aside` |
| 文本 | `h1`, `h2`, `h3`, `h4`, `h5`, `h6`, `p`, `label`, `a` |
| 表单 | `button`, `input`, `textarea`, `select`, `form` |
| 列表 | `ul`, `ol`, `li` |

无子节点时，自闭合和成对标签等价：

```rust
rsx! { <div flex /> }
rsx! { <div flex></div> }
```

## 特殊标签

部分标签会直接使用 GPUI 构造器：

| RSX | 生成的构造器 |
| --- | --- |
| `<svg src={path} />` | `svg().path(path)` |
| `<img src={source} />` | `img(source)` |
| `<img source={source} />` | `img(source)` |
| `<canvas prepaint={prepaint} paint={paint} />` | `canvas(prepaint, paint)` |

`img` 必须提供 `src` 或 `source`。`canvas` 必须同时提供 `prepaint` 和 `paint`。

## 组件

未知的单段标签会被视为构造函数：

```rust
rsx! {
    <MyComponent title={"Inbox"} />
}
```

路径标签会调用对应路径：

```rust
rsx! {
    <ui::TaskCard title={task.title.clone()} />
}
```

当 builder 需要参数，或来自组件库时，使用 `base={expr}`：

```rust
rsx! {
    <Button
        base={Button::new("save")}
        label={"Save"}
        small
    />
}
```

`base` 会被宏消费，不会生成 `.base(...)` 方法调用。

### 函数式组件

在同步函数上使用 `#[component]`，即可生成与结构体组件相同的属性 builder。
函数体在 GPUI 渲染该元素时执行：

```rust
use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx};

#[component]
fn Badge(
    label: SharedString,
    #[prop(default)] highlighted: bool,
    #[prop(default = 2)] depth: u8,
    #[children] children: Vec<AnyElement>,
    #[context] cx: &mut App,
) -> impl IntoElement {
    let _ = (cx, depth);
    rsx! {
        <div class="flex gap-2" whenClass={(highlighted, "bg-blue-500")}>
            {label}{...children}
        </div>
    }
}

let badge = rsx! {
    <Badge label="Inbox" highlighted>
        <><span>{"3 unread"}</span><span>{"!"}</span></>
    </Badge>
};
```

普通参数为必填属性。`#[prop(default)]` 使用 `Default::default()`，
`#[prop(default = expr)]` 提供显式默认值。带默认值的布尔属性使用 flag setter
（`highlighted`）或条件 setter（`highlightedIf={active}`）。`#[children]` 将子节点
收集到 `Vec<AnyElement>`。也支持 `<ui::Badge ...>` 这样的模块路径标签。

无后缀数值字面量与纯数值表达式按声明的基础数值类型推断，例如
`#[prop(default = 7)] value: u32`、`#[prop(default = 7.0)] ratio: f32`。
带后缀的数值保留 `Into` 转换，字符串默认值也保留转换能力，例如 `SharedString` 的 `"ready"`。

可选的 `#[context]` 参数必须是 `&mut Window` 或 `&mut App`，由 GPUI 渲染时传入，
调用方无需将它们作为属性传递；宏也不会隐式捕获调用方的 `window` 或 `cx`。
属性使用具体的拥有型类型或 `'static` 引用。暂不支持 async/unsafe 函数或解构参数。原函数名会成为无参 builder 构造函数：
`Badge().label("Inbox")`。

### 结构体组件与弃用提示

命名结构体使用相同的 prop 注解，实现 `RenderOnce` 并派生 `gpui::IntoElement`：

```rust
#[component]
#[derive(gpui::IntoElement)]
struct Panel {
    title: gpui::SharedString,
    #[prop(default)] compact: bool,
    #[children] children: Vec<gpui::AnyElement>,
}
impl gpui::RenderOnce for Panel {
    fn render(self, _: &mut gpui::Window, _: &mut gpui::App) -> impl gpui::IntoElement {
        div().child(self.title).children(self.children)
    }
}
// rsx! { <Panel title="Inbox" compact><span>{"Hello"}</span></Panel> }
```

在 `#[component]` 结构体或函数上添加 `#[deprecated(note = "use NewPanel")]`，
即可在 `<Panel ...>` 与 `Panel()` 调用处提示弃用；生成代码内部不会重复报警。

### 回调、可选属性与泛型

`Box<dyn Fn...>` / `Rc<dyn Fn...>` 的 setter 与 GPUI 自带的事件 setter 一样接收
`impl Fn... + 'static`，闭包参数可自动推导（`on_press={|_, _, _| ...}`），并保留声明的
`Send`、`Sync` 和生命周期约束。传入已有的共享回调时需转发：
`on_run={{ let run = run.clone(); move || run() }}`。

回调默认值可直接构造容器，例如在 Box GPUI 处理函数上写
`#[prop(default = Box::new(|_, _, _| {}))]`。默认值会转换为声明的回调类型，闭包参数与 setter
一样可自动推导。

宏按书写出的类型识别回调，因此 `type OnPress = Rc<dyn Fn(...)>` 这样的类型别名会被当作普通值属性。
需要给较长的处理函数签名起名时，请声明 trait 别名并给属性加上 `#[prop(callback)]`（可与 `default`
组合）；闭包仍可通过 supertrait 推导参数：

```rust
trait OnPress: Fn(&ClickEvent, &mut Window, &mut App) {}
impl<F: Fn(&ClickEvent, &mut Window, &mut App)> OnPress for F {}

#[component]
fn Pressable(#[prop(callback)] on_press: Rc<dyn OnPress>) -> impl IntoElement {
    div().id("pressable").on_click(move |e, w, cx| on_press(e, w, cx))
}
let pressable = rsx! { <Pressable on_press={|_, window, _| window.refresh()} /> };
```

对 `type` 别名使用 `#[prop(callback)]` 会产生编译错误，并提示改用上述写法。

`Option<T>` 可接收 `T`、`Some(T)` 或 `None`；内部值必须已经是 `T` 类型，
例如 `Option<SharedString>` 需先将字符串转为 `SharedString`。

```rust
#[component]
fn Action<T: Into<gpui::SharedString>>(
    label: T,
    on_run: std::rc::Rc<dyn Fn()>,
    #[prop(default)] hint: Option<gpui::SharedString>,
) -> impl gpui::IntoElement {
    let _ = (on_run, hint);
    div().child(label.into())
}
let action = rsx! {
    <Action label={String::from("Run")} on_run={|| println!("run")}
        hint={gpui::SharedString::from("Ready")} />
};
```

类型泛型、const 泛型、默认类型和 `where` 条件都会保留。函数组件自动添加 GPUI 所需的
`'static` 约束。类型为 `T` 的 prop 可帮助推导泛型；无法推导时，在构造函数上显式指定：
`<ArrayCard base={ArrayCard::<3>()} values={[1, 2, 3]} />`。
标签支持模块路径，不支持 turbofish 写法。泛型结构体需满足自身的 GPUI trait 约束。

## 属性

Flag 属性会生成无参方法：

```rust
rsx! { <div flex flex_col items_center /> }
```

Value 属性会生成单参方法：

```rust
rsx! {
    <div gap={px(16.0)} bg={rgb(0x3b82f6)} />
}
```

多参数 GPUI 方法使用 tuple 语法：

```rust
rsx! {
    <div onMouseDown={(MouseButton::Left, handler)} />
}
```

`InteractiveText` 的 `onClick` 和 `on_click` 也支持通过 tuple 传入两个参数：

```rust
rsx! {
    <div base={InteractiveText::new("text", StyledText::new("reserve"))}
        onClick={(vec![0..7], |_, _, _| {})} />
}
```

该写法展开为 `.on_click(ranges, listener)`。普通 `Div` 的监听器继续使用
单参数写法 `onClick={listener}`。

常见 CamelCase alias 会映射到 GPUI 的 snake_case 方法：

| RSX 属性 | GPUI 方法 |
| --- | --- |
| `onClick` | `on_click` |
| `onKeyDown` | `on_key_down` |
| `onMouseDown` | `on_mouse_down` |
| `textColor` | `text_color` |
| `backgroundColor` | `bg` |
| `fontSize` | `text_size` |
| `minWidth` / `maxHeight` | `min_w` / `max_h` |
| `overflowScroll` | `overflow_scroll` |
| `trackScroll` | `track_scroll` |
| `accessibilityId` / `ariaDescription` | `accessibility_id` / `aria_description` |
| `ariaKeyShortcuts` / `ariaValue` | `aria_keyshortcuts` / `aria_value` |
| `ariaActiveDescendant` | `aria_active_descendant` |
| `a11ySyntheticChildren` | `a11y_synthetic_children` |
| `restrictScrollToAxis` | `restrict_scroll_to_axis` |
| `externalDragPayload` | `external_drag_payload` |
| `onMouseExit` / `onMousePressure` / `onPinch` | `on_mouse_exit` / `on_mouse_pressure` / `on_pinch` |
| `gridColsMinContent` / `gridRowsMaxContent` | `grid_cols_min_content` / `grid_rows_max_content` |

`externalDragPayload` 必须位于同一元素的 `onDrag` 之后，并使用相同的拖拽值类型，以符合
GPUI 的方法契约。宏会保留属性顺序，不会自动重排这两个调用。

`whiteSpace` 会被显式拒绝。可使用 `whitespace-nowrap` 等 class，或在 GPUI 可用时直接调用对应方法。

## 子节点

字符串字面量可以直接作为子节点：

```rust
rsx! { <div>"Hello"</div> }
```

Rust 表达式需要放在花括号中：

```rust
rsx! {
    <div>
        {format!("Count: {}", self.count)}
        {self.render_footer()}
    </div>
}
```

嵌套元素会生成 `.child(...)`：

```rust
rsx! {
    <section>
        <h1 styled>{"Title"}</h1>
        <p>{"Body"}</p>
    </section>
}
```

已有 iterable 可以用 spread 子节点：

```rust
rsx! {
    <div>
        {...rows}
    </div>
}
```

## For 循环

使用 `{for pattern in iter { ... }}` 渲染迭代器：

```rust
rsx! {
    <ul>
        {for (index, item) in items.iter().enumerate() {
            <li>{format!("{}: {}", index, item.name)}</li>
        }}
    </ul>
}
```

如果循环体里包含有状态元素，需要添加 `key={...}` 或 `id={...}`。详见 [ID 与 Key](/gpui-rsx/zh-cn/usage/ids/)。

## Fragment

Fragment 可以返回多个根元素：

```rust
rsx! {
    <>
        <div>{"First"}</div>
        <div>{"Second"}</div>
    </>
}
```

Fragment 会展开为 `vec![...]`。如果根元素具体类型不同，通常需要 `into_any_element()` 或包一层共同父元素。根 Fragment 中包含 `for` 循环或 `{...spread}` 时，会展开为 `Vec<AnyElement>`，元素顺序保持不变。

需要混合不同根元素类型时，使用始终返回 `Vec<gpui::AnyElement>` 的 `rsx_fragment!`：

```rust
let parts = gpui_rsx::rsx_fragment! {
    <>
        <div />
        <div id="stateful" />
    </>
};
rsx! { <div>{...parts}</div> }
```

它沿用 `rsx!` 的间距刻度与项目配置。单根元素返回只有一项的向量，空 Fragment 返回空向量。
普通 `rsx!` Fragment 继续保留原来的具体元素类型。

Fragment 也可以嵌套：

```rust
rsx! {
    <div>
        <><span>{"First"}</span><span>{"Second"}</span></>
        {for item in items {
            <><span>{item.name.clone()}</span><span>{"Details"}</span></>
        }}
    </div>
}
```

嵌套分组按顺序直接添加子节点，不增加包装元素或额外的运行时向量，支持空分组。
循环内的 stateful 后代仍需提供 `key` 或 `id`；格式化保留分组，诊断和预览会遍历其中内容。

## 条件渲染

子节点中可以使用 Rust 表达式：

```rust
rsx! {
    <div>
        {if let Some(user) = &self.user {
            rsx! { <span>{user.name.as_str()}</span> }
        } else {
            rsx! { <span>{"Guest"}</span> }
        }}
    </div>
}
```

使用 `when` 条件转换当前正在构建的元素：

```rust
rsx! {
    <div
        class="px-4 py-2"
        when={(active, |el| el.bg(rgb(0x3b82f6)).text_color(rgb(0xffffff)))}
    >
        {"Status"}
    </div>
}
```

当条件值是 `Option` 时使用 `whenSome`：

```rust
rsx! {
    <div whenSome={(custom_width, |el, width| el.w(px(width)))} />
}
```

使用 `whenClass` 应用条件静态 class：

```rust
rsx! {
    <div
        class="px-2"
        whenClass={(active, "bg-neutral-900 text-white")}
        whenClass={(!active, "text-neutral-600")}
    />
}
```

`whenClass` 的 class 必须是字符串字面量，并且会拒绝 `overflow-scroll` 这类有状态 class。涉及 ID 语义时请使用 `when`。

## 特殊属性

`visible={bool}` 会映射到 `.visible()` 或 `.invisible()`，且表达式只求值一次：

```rust
rsx! {
    <div visible={self.show_sidebar} />
}
```

`grayscale` 会映射为 `.grayscale(true)`：

```rust
rsx! {
    <img src={source} grayscale />
}
```

`styled` 会在用户属性前注入标签默认 class：

| 标签 | 默认 class |
| --- | --- |
| `h1` | `text-3xl font-bold` |
| `h2` | `text-2xl font-bold` |
| `h3` | `text-xl font-bold` |
| `h4` | `text-lg font-bold` |
| `h5` | `text-base font-bold` |
| `h6` | `text-sm font-bold` |
| `button`, `a` | `cursor-pointer` |
| `input`, `textarea` | `px-2 py-1` |
| `ul`, `ol` | `flex flex-col` |
| `li` | `flex items-center` |
| `p` | `text-base` |
| `label` | `text-sm` |
| `form` | `flex flex-col gap-4` |

用户属性会在 `styled` 之后应用，因此后续值可以覆盖默认值：

```rust
rsx! {
    <h1 styled class="text-xl">{title.as_str()}</h1>
}
```
