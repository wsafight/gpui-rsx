
## Explicit state updates

`rsx_update!(view, cx, update)` runs a synchronous update closure and calls `cx.notify()`
after normal return. It accepts the mutable view and its matching GPUI context, runs a
`FnOnce(&mut View, &mut Context<View>) -> Output`, and returns `Output` unchanged.
Each invocation adds its own notification; it does not batch separate calls or suppress
notifications made by the closure.

```rust
use gpui::Context;
use gpui_rsx::rsx_update;

struct Counter { count: usize, label: String }

fn update_counter(view: &mut Counter, cx: &mut Context<'_, Counter>, label: String) -> usize {
    rsx_update!(view, cx, move |view, _| {
        view.count += 1;
        view.label = label;
        view.count
    })
}
```

Use it inside an entity update, a native listener, or the plain `rsx_listener!` form.
Do not add the listener's `notify` marker when this macro already owns notification.
The macro notifies after an early return too, even when no field changed. It does not
intercept ordinary assignments, own application state, detect equality, roll back on
panic, or notify when an asynchronous task finishes. Let GPUI manage the entity and
use another explicit update when an asynchronous result is ready.
