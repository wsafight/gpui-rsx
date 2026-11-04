
## Event listeners

`rsx_listener!` is an optional shorthand for the repeated `Context::listener` shape. It keeps
the existing callback signature and does not change event propagation or GPUI's event behavior:

```rust
use gpui::{Context, div, prelude::*};
use gpui_rsx::{rsx, rsx_listener};
# struct Counter { count: usize }

fn render(cx: &mut Context<Counter>) -> impl IntoElement {
    rsx! {
        <button onClick={rsx_listener!(cx, notify, |view, _, _, _| {
            view.count += 1;
        })}>
            {"Increment"}
        </button>
    }
}
```

The plain form is equivalent to `cx.listener(handler)` and leaves notification responsibility in
the handler. The `notify` form calls `cx.notify()` after the handler returns normally, including when the
handler returns early. Use it only when that notification policy is intended. The context and
handler expressions are each evaluated once, and handlers remain synchronous `Fn` callbacks.
Notification is not guaranteed after a panic or when an asynchronous task finishes. Use the
plain form when `rsx_update!` already owns notification inside the handler.
