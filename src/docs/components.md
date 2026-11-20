Turns a struct or synchronous function into an RSX component with a typestate props builder.

Function parameters become props, using the same defaults, children, slot, and style annotations as
struct fields. The function runs when GPUI renders the element. Optional `#[context]`
parameters receive `&mut Window` or `&mut App` at that point; they are not props.

```
use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx};

#[component]
fn Badge(
    label: SharedString,
    #[prop(default)] highlighted: bool,
    #[children] children: Vec<AnyElement>,
    #[context] cx: &mut App,
) -> impl IntoElement {
    let _ = cx;
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
# let _ = badge.into_any_element();
```

### Style forwarding (`#[style]`) and Named Slots (`#[slot]`)

- `#[style] style: StyleRefinement` implements `gpui::Styled` on the generated builder so callers can pass static or dynamic `class` attributes directly on `<Component class="..." />`.
- `#[slot]` declares optional named slots (`Option<AnyElement>` or `Vec<AnyElement>`). Callers can pass them either as attributes (`header={rsx! { ... }}`) or by marking direct child elements with `slot="header"`.
- `#[children]` accepts `Vec<AnyElement>`, `SmallVec<[AnyElement; N]>`, or `Option<AnyElement>` (for zero-allocation single-child components).

```
use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx};

#[component]
fn Modal(
    title: SharedString,
    #[style] style: StyleRefinement,
    #[slot] header: Option<AnyElement>,
    #[slot] footer: Vec<AnyElement>,
    #[children] body: Option<AnyElement>,
) -> impl IntoElement {
    let mut root = rsx! {
        <div class="flex flex-col gap-3 p-4">
            <h2>{title}</h2>
        </div>
    };
    root.style().refine(&style);
    root
        .when_some(header, |el, h| el.child(h))
        .when_some(body, |el, b| el.child(b))
        .children(footer)
}

let modal = rsx! {
    <Modal title="Confirm" class="w-64 rounded-lg">
        <span slot="header">{"Notice"}</span>
        <p>{"Proceed with changes?"}</p>
        <button slot="footer">{"Cancel"}</button>
        <button slot="footer">{"Confirm"}</button>
    </Modal>
};
# let _ = modal.into_any_element();
```

Functions must use owned (or `'static`) prop types and an explicit element return type.
Async/unsafe functions, destructured parameters and `impl Trait` props are not supported.
The generated constructor is `Badge()` (with `#[track_caller]` for GPUI Inspector source tracking); use its builder or `<Badge ...>` to supply props.

Callback props `Box<dyn Fn...>` and `Rc<dyn Fn...>` take `impl Fn... + 'static`, so
closure arguments are inferred; forward an existing `Rc` with `move |..| shared(..)`.
For a trait alias (`trait OnPress: Fn(..) {}` with a blanket impl), write
`#[prop(callback)] on_press: Rc<dyn OnPress>`; `type` aliases are not recognized.
`Option<T>` accepts an inner `T`, `Some(T)` or `None`. Type/const generics and `where`
clauses are supported; function component types must satisfy GPUI's `'static` bounds.
Infer types from direct generic props or use `base={Card::<T>()}` for explicit arguments.

Struct components keep their existing `RenderOnce` implementation:

- plain fields are **required** props; forgetting one is a compile error naming the prop
- `#[prop(default)]` / `#[prop(default = expr)]` fields are optional; `bool` props become
  flags (`<Card elevated />`) plus a `fooIf={cond}` form
- one `#[children]` field (`Vec<AnyElement>`, `SmallVec`, or `Option<AnyElement>`) collects RSX children
- `#[slot]` fields collect named slot elements and `#[style]` forwards `gpui::Styled`

The macro adds a function with the component's name (`Card()`), which is what
`<Card ...>` expands to, and implements `IntoElement` for the finished builder by
delegating to the component's own `IntoElement` (usually `#[derive(IntoElement)]` +
`RenderOnce`). `IntoElement` must be in scope (`use gpui::prelude::*`).

```
use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx};

#[component]
#[derive(IntoElement)]
struct Card {
    title: SharedString,
    #[prop(default)]
    elevated: bool,
    #[children]
    children: Vec<AnyElement>,
}

impl RenderOnce for Card {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        rsx! {
            <div class="flex flex-col gap-2 p-4 rounded-lg" whenClass={(self.elevated, "shadow-md")}>
                <h2 class="text-lg font-semibold">{self.title}</h2>
                {...self.children}
            </div>
        }
    }
}

let card = rsx! {
    <Card title="Inbox" elevated>
        <span>{"3 unread"}</span>
    </Card>
};
# let _ = card.into_any_element();
```

Leaving out `title` fails to compile with
``missing required prop `title` on `<Card>` ``.
