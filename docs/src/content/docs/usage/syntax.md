---
title: Syntax Reference
description: Elements, attributes, children, fragments, conditionals, and special GPUI tags.
---

Look up elements, attributes, Rust expressions, loops, and components here. Open `syntax_gallery` from the [runnable demos](/gpui-rsx/examples/demos/) to try them. Utility support is covered in [Class Handling](/gpui-rsx/usage/class/); element identity is covered in [IDs and Keys](/gpui-rsx/usage/ids/).

## Elements

RSX elements use an HTML-like shape:

```rust
rsx! {
    <tag attr value={expr}>
        {child_expr}
    </tag>
}
```

Common HTML-like tags map to `div()` because GPUI does not expose browser DOM nodes:

| Group | Tags |
| --- | --- |
| Structure | `div`, `span`, `section`, `article`, `header`, `footer`, `main`, `nav`, `aside` |
| Text | `h1`, `h2`, `h3`, `h4`, `h5`, `h6`, `p`, `label`, `a` |
| Forms | `button`, `input`, `textarea`, `select`, `form` |
| Lists | `ul`, `ol`, `li` |

These names do not supply HTML semantics: `<input />` is not an editable input,
`<button />` does not add a component library's keyboard or focus behavior, and
`<form>` does not submit data. `styled` adds style defaults only. Use explicit
component constructors for controls, as shown under [Components](#components).

Self-closing and paired tags are equivalent when there are no children:

```rust
rsx! { <div flex /> }
rsx! { <div flex></div> }
```

## Special Tags

Some tags use GPUI constructors directly:

| RSX | Generated constructor |
| --- | --- |
| `<svg src={path} />` | `svg().path(path)` |
| `<img src={source} />` | `img(source)` |
| `<img source={source} />` | `img(source)` |
| `<canvas prepaint={prepaint} paint={paint} />` | `canvas(prepaint, paint)` |

`img` requires `src` or `source`. `canvas` requires both `prepaint` and `paint`.

## Components

Unknown single-segment tags are treated as constructor functions:

```rust
rsx! {
    <MyComponent title={"Inbox"} />
}
```

Path-qualified tags call the path:

```rust
rsx! {
    <ui::TaskCard title={task.title.clone()} />
}
```

Use `base={expr}` when the builder needs arguments or comes from a component library:

```rust
use gpui::component::button::Button;
use gpui::component::Sizable as _;

rsx! {
    <Button
        base={Button::new("save")}
        label={"Save"}
        small
    />
}
```

`base` is consumed by the macro. It does not generate a `.base(...)` method call.

| Construction need | Recommended form |
| --- | --- |
| Builder with a zero-argument constructor | `<Card title="Inbox" />` or `<ui::Card ... />` |
| Constructor arguments or a preconfigured builder | `<Button base={Button::new("save")} ... />` |
| Optional props | Declare `#[prop(default)]` or `#[prop(default = expr)]`; omit the prop to use its default |
| Heterogeneous roots or stored child collections | Use [`rsx_fragment!`](#fragments), then `{...parts}` |
| Several roots that need concrete builder methods | Keep an ordinary `rsx!` fragment with compatible root types |

`base` evaluates once and keeps the builder's own setters. Pass event callbacks
according to that builder's API; an `onClick` prop on a component is not automatically
a view listener. Prefer camelCase for GPUI event attributes and snake_case for direct
native methods; both supported spellings generate the same calls.

### Function components

Use `#[component]` on a synchronous function to generate the same props builder as a
struct component. The function runs during GPUI rendering:

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

Plain parameters are required props. `#[prop(default)]` uses `Default::default()`;
`#[prop(default = expr)]` supplies an explicit default. Defaulted booleans use flag setters
(`highlighted`) or conditional setters (`highlightedIf={active}`). `#[children]` collects
children into `Vec<AnyElement>`. Module-qualified tags such as `<ui::Badge ...>` work too.

Unsuffixed numeric literals and pure numeric expressions infer the declared primitive
field type, such as `#[prop(default = 7)] value: u32` or `#[prop(default = 7.0)] ratio: f32`.
Suffixed values retain `Into` conversions, as do string defaults such as `"ready"` for `SharedString`.

Optional `#[context]` parameters must be `&mut Window` or `&mut App`; they are supplied by
GPUI, so callers never pass them as props. The macro does not capture the caller's `window`
or `cx`. Prop types must be concrete and owned (or `'static` references). Async/unsafe
functions and destructured parameters are unsupported.
The original function name becomes a zero-argument builder constructor: `Badge().label("Inbox")`.

### Struct components and deprecation

Named structs use the same prop annotations. Implement `RenderOnce` and derive
`gpui::IntoElement` to render the built value:

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

Put `#[deprecated(note = "use NewPanel")]` on a `#[component]` struct or function to
warn at `<Panel ...>` and `Panel()` call sites. Generated internals suppress their own
warnings; callers still receive the deprecation notice.

### Callbacks, optional props and generics

`Box<dyn Fn...>` and `Rc<dyn Fn...>` setters take `impl Fn... + 'static` like GPUI's own
event setters, so closure arguments are inferred (`on_press={|_, _, _| ...}`) and declared
`Send`, `Sync` and lifetime bounds are kept. To pass an existing shared callback, forward it:
`on_run={{ let run = run.clone(); move || run() }}`.

Callback defaults can construct their container directly, for example
`#[prop(default = Box::new(|_, _, _| {}))]` on a boxed GPUI handler. The default is
coerced to the declared callback type, so closure arguments infer as they do in setters.

The macro recognizes callbacks by their written type, so a `type OnPress = Rc<dyn Fn(...)>`
alias is treated as an ordinary value prop. To name a long handler signature, declare a
trait alias and mark the prop with `#[prop(callback)]` (combinable with `default`); closures
still infer their arguments through the supertrait:

```rust
trait OnPress: Fn(&ClickEvent, &mut Window, &mut App) {}
impl<F: Fn(&ClickEvent, &mut Window, &mut App)> OnPress for F {}

#[component]
fn Pressable(#[prop(callback)] on_press: Rc<dyn OnPress>) -> impl IntoElement {
    div().id("pressable").on_click(move |e, w, cx| on_press(e, w, cx))
}
let pressable = rsx! { <Pressable on_press={|_, window, _| window.refresh()} /> };
```

`#[prop(callback)]` on a `type` alias is a compile error that points to this pattern.

`Option<T>` setters accept `T`, `Some(T)` or `None`; the inner value must already have type
`T` (for `Option<SharedString>`, convert a string to `SharedString` first).

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

Type and const generics, defaults and `where` clauses are preserved. Function components
add `'static` bounds required by GPUI. Types can infer from a prop typed directly as `T`;
use explicit constructor arguments when inference is insufficient, for example
`<ArrayCard base={ArrayCard::<3>()} values={[1, 2, 3]} />`. Tags use module paths rather
than turbofish syntax. Generic struct components must satisfy their own GPUI trait bounds.

## Attributes

Flag attributes become zero-argument methods:

```rust
rsx! { <div flex flex_col items_center /> }
```

Value attributes become one-argument methods:

```rust
rsx! {
    <div gap={px(16.0)} bg={rgb(0x3b82f6)} />
}
```

Multi-argument GPUI methods use tuple syntax:

```rust
rsx! {
    <div onMouseDown={(MouseButton::Left, handler)} />
}
```

### Event listeners

`InteractiveText` also accepts two arguments through `onClick` or `on_click`:

```rust
rsx! {
    <div base={InteractiveText::new("text", StyledText::new("reserve"))}
        onClick={(vec![0..7], |_, _, _| {})} />
}
```

This expands to `.on_click(ranges, listener)`. Ordinary `Div` listeners keep
the single-argument form `onClick={listener}`.

For view callbacks, the optional `rsx_listener!` macro keeps GPUI's four-argument
`(view, event, window, cx)` handler signature:

```rust
use gpui_rsx::rsx_listener;

rsx! {
    <button onClick={rsx_listener!(cx, notify, |view, _, _, _| {
        view.count += 1;
    })}>{"Increment"}</button>
}
```

`rsx_listener!(cx, handler)` delegates to `cx.listener(handler)` and leaves notification
to the handler. The explicit `notify` form calls `cx.notify()` after normal return,
including early return with no state change; avoid another unconditional notification
inside that handler. Context and handler expressions each evaluate once. `move` closures
and function expressions work; callbacks remain synchronous `Fn`, with no panic or async
notification guarantee. For event field access, generic inference may need a type such
as `event: &MouseDownEvent`. Mouse button tuples keep their ordinary syntax:
`onMouseDown={(MouseButton::Right, rsx_listener!(cx, notify, handler))}`.

### Explicit state updates

`rsx_update!(view, cx, update)` runs a synchronous `FnOnce` with the mutable view and
its matching `Context<View>`, calls `cx.notify()` after normal return and returns the
updater's result. Several field changes can share one notification:

```rust
use gpui_rsx::{rsx_listener, rsx_update};

rsx! {
    <div onClick={rsx_listener!(cx, |view, _, _, cx| {
        rsx_update!(view, cx, |view, _| {
            view.count += 1;
            view.label = String::from("updated");
        });
    })}>{"Update"}</div>
}
```

View, context and updater expressions each evaluate once. Moved values and borrowed
captures are supported; the updater does not need to be `'static`. Early return and
unchanged fields still notify. Use the plain listener form above, since `rsx_update!`
owns notification. The macro does not intercept other assignments, roll back on panic
or notify on asynchronous completion. Use another entity update when an async result
is ready. The entity and its lifecycle remain managed by GPUI.

Each invocation calls `cx.notify()` independently; the macro does not batch separate
calls or suppress notifications made inside the update closure.

### Attribute aliases

CamelCase aliases map to GPUI snake_case methods where needed:

| RSX attribute | GPUI method |
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

`externalDragPayload` must follow `onDrag` on the same element and use the same dragged value type,
matching GPUI's method contract. The macro preserves attribute order; it does not reorder these calls.

`whiteSpace` is intentionally rejected. Use GPUI's supported whitespace helpers through class utilities such as `whitespace-nowrap`, or direct GPUI methods when available.

## Children

String literals are valid children:

```rust
rsx! { <div>"Hello"</div> }
```

Rust expressions are wrapped in braces:

```rust
rsx! {
    <div>
        {format!("Count: {}", self.count)}
        {self.render_footer()}
    </div>
}
```

Nested elements become `.child(...)` calls:

```rust
rsx! {
    <section>
        <h1 styled>{"Title"}</h1>
        <p>{"Body"}</p>
    </section>
}
```

Use spread children for existing iterables:

```rust
rsx! {
    <div>
        {...rows}
    </div>
}
```

## For Loops

Use `{for pattern in iter { ... }}` to render an iterator as children:

```rust
rsx! {
    <ul>
        {for (index, item) in items.iter().enumerate() {
            <li>{format!("{}: {}", index, item.name)}</li>
        }}
    </ul>
}
```

If a loop body contains stateful elements, add `key={...}` or `id={...}`. See [IDs and Keys](/gpui-rsx/usage/ids/).

## Fragments

Fragments return multiple root elements:

```rust
rsx! {
    <>
        <div>{"First"}</div>
        <div>{"Second"}</div>
    </>
}
```

A fragment expands to `vec![...]`. Mixed concrete root types may need `into_any_element()` or a common wrapper element. A root fragment that contains a `for` loop or a `{...spread}` expands to a `Vec<AnyElement>` instead, so every element keeps its order.

Use `rsx_fragment!` to always return `Vec<gpui::AnyElement>`, including mixed root types:

```rust
let parts = gpui_rsx::rsx_fragment! {
    <>
        <div />
        <div id="stateful" />
    </>
};
rsx! { <div>{...parts}</div> }
```

It follows the same spacing scale and project configuration as `rsx!`. A single root
returns a one-element vector, and an empty fragment returns an empty vector. Ordinary
`rsx!` fragments keep their existing concrete element types.

Fragments can also be nested:

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

A nested group adds its children directly, preserving order without a wrapper or an extra
runtime vector. Empty groups are allowed. Inside loops, stateful descendants still need
`key` or `id`. Formatting, diagnostics and preview retain or traverse the group as appropriate.

## Conditional Rendering

Rust expressions work inside children:

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

Use `when` to conditionally transform the element being built:

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

Use `whenSome` when the conditional value is an `Option`:

```rust
rsx! {
    <div whenSome={(custom_width, |el, width| el.w(px(width)))} />
}
```

Use `whenClass` for conditional static class strings:

```rust
rsx! {
    <div
        class="px-2"
        whenClass={(active, "bg-neutral-900 text-white")}
        whenClass={(!active, "text-neutral-600")}
    />
}
```

`whenClass` requires a string literal and rejects stateful classes such as `overflow-scroll`. Use `when` for ID-sensitive methods.

## Special Attributes

`visible={bool}` maps to `.visible()` or `.invisible()` while evaluating the expression once:

```rust
rsx! {
    <div visible={self.show_sidebar} />
}
```

`grayscale` maps to `.grayscale(true)`:

```rust
rsx! {
    <img src={source} grayscale />
}
```

`styled` injects default classes for semantic tags before user attributes:

| Tag | Default classes |
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

User attributes are applied after `styled`, so later values can override defaults:

```rust
rsx! {
    <h1 styled class="text-xl">{title.as_str()}</h1>
}
```
