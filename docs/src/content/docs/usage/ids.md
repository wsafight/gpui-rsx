---
title: IDs and Keys
description: How GPUI-RSX generates IDs and how to use keys in loops.
---

GPUI-RSX generates stable IDs when GPUI APIs require an element identity.

Most layout and styling calls do not need an ID. The macro only injects one when a stateful GPUI method is used.

## Auto IDs

For static RSX outside loops, the macro can generate IDs from the call site. This keeps common markup concise:

```rust
rsx! {
    <button onClick={handler}>
        {"Click"}
    </button>
}
```

Static stateful scroll classes also trigger auto IDs:

```rust
rsx! {
    <div class="overflow-y-scroll">
        {"Scrollable content"}
    </div>
}
```

The generated ID is deterministic for the macro expansion site.

For component tags such as `<ui::card onClick={handler} />`, automatic ID injection depends
on the constructor's return type. Values implementing both `gpui::InteractiveElement` and
`gpui::Element` receive an ID, so functions returning `Div` keep working. Other values,
including `#[component]` builders and gpui-component's `Button`, keep their own event
setters without an injected `.id()`. The same rule applies to `base={...}` expressions.

Dispatch uses the type and trait bounds visible at the macro call site. Generic or opaque
element types must expose both bounds for automatic IDs. If `.id()` is available but the
`Element` bound is hidden, set `id={...}` explicitly. With only `impl IntoElement`, apply
IDs and stateful methods inside the constructor, where the concrete type is available.

Common ID-triggering attributes include `onClick`, `onHover`, `onDrag`, `onAuxClick`, `onA11yAction`, `active`, `activeClass`, `groupActive`, `tooltip`, `tooltipShowDelay`, `focusable`, `role`, `accessibilityId`, `ariaLabel`, `ariaDescription`, `ariaKeyShortcuts`, `ariaActiveDescendant`, `a11ySyntheticChildren`, `ariaNumericValueStep`, `ariaValue`, `ariaPlaceholder`, `overflowScroll`, `overflowXScroll`, `overflowYScroll`, `restrictScrollToAxis`, `trackScroll`, `externalDragPayload`, and static `overflow-scroll` class variants.

Attributes such as `hover`, `hoverClass`, `focus`, `focusClass`, `group`, `groupHover`, `scrollbarWidth`, `onMouseDown`, `onMouseExit`, `onMousePressure`, `captureMousePressure`, `onPinch`, `capturePinch`, and `captureKeyDown` do not by themselves require a stateful ID in the current GPUI target.

`accessibilityId` sets the author-provided identifier exposed to assistive technology. It is not the
GPUI element identity created by `.id(...)`; the macro still injects a separate GPUI ID before calling
`.accessibility_id(...)`. `ariaActiveDescendant` and `a11ySyntheticChildren` also need an appropriate
`role` to produce a meaningful accessibility node.

## Keys in Loops

When rendering repeated stateful elements, provide `key={...}`:

```rust
rsx! {
    <div>
        {for task in &self.tasks {
            <button key={task.id} onClick={handler} class="flex items-center gap-2">
                {task.title.clone()}
            </button>
        }}
    </div>
}
```

Loops that need stateful IDs produce a compile error if the key is missing. This prevents repeated rows from accidentally sharing the same generated ID.

Component tags with stateful attribute names still require `key` or `id` inside loops,
even when their builders do not need an injected ID: the macro checks loop safety before
Rust resolves the type. Key expressions are still evaluated once even when a builder
does not use the generated ID.

`key` is a macro-only attribute. It never becomes a `.key(...)` method call.

## Literal and Dynamic Keys

Literal keys can be expanded with `concat!`:

```rust
rsx! {
    <button key="toolbar" onClick={handler} />
}
```

Literal keys use `concat!`. Concrete dynamic integer values up to 64 bits (including
`usize` / `isize`) use GPUI's `(&'static str, u64)` ID without formatting or allocating a
string. Other `Display` values retain the formatted-string fallback, including `u128` /
`i128` to avoid truncation. Borrowed integer references and generic `T: Display` values may
also use that fallback. The expression is borrowed and evaluated once, so a String key is
not moved. Prefer an owned integer value such as `key={task.id}` for large stateful lists.

Example:

```rust
rsx! {
    <button key={task.id} onClick={handler} />
}
```

If an element does not need an ID, `key` is ignored:

```rust
rsx! {
    <div key={task.id} class="flex items-center">
        {task.title.as_str()}
    </div>
}
```

The example above stays a plain element because no stateful method is present.

## Explicit IDs

An explicit `id` always wins:

```rust
rsx! {
    <button id="save-button" key={task.id} onClick={handler}>
        {"Save"}
    </button>
}
```

Use explicit IDs when identity must remain stable across source-line movement or refactors.

## When to Add a Key

Add a key when:

- the element appears inside `{for ...}`,
- the class or attribute uses a GPUI stateful behavior,
- the identity must remain stable across reorder or update operations.

Keep simple, non-repeated elements keyless and let the macro generate IDs where needed.
