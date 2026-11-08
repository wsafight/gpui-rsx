---
title: Troubleshooting
description: Common parser, type, runtime, styling, and dependency issues when using GPUI-RSX.
---

Start with the error reported by Rust or `cargo rsx lint`, then find the matching section below. For dependency failures, check the [verified dependency set](/gpui-rsx/compatibility/) before changing the lockfile.

## Parser Errors

### Closing Tag Mismatch

Example error:

```text
Closing tag `</span>` does not match opening tag `<div>`.
```

Fix the nesting:

```rust
// Wrong
rsx! {
    <div>
        <span>{"Text"}</div>
    </span>
}

// Correct
rsx! {
    <div>
        <span>{"Text"}</span>
    </div>
}
```

### Unclosed Tag or Fragment

Add the missing closing tag:

```rust
// Wrong
rsx! {
    <div>{"Content"}
}

// Correct
rsx! {
    <div>{"Content"}</div>
}
```

### Unexpected Child Token

Bare identifiers are not children. Wrap expressions in braces:

```rust
// Wrong
rsx! { <div>count</div> }

// Correct
rsx! { <div>{count}</div> }
```

String literals can be written directly:

```rust
rsx! { <div>"Count"</div> }
```

### For Loop Body Missing Braces

The loop body must be a braced RSX body:

```rust
// Wrong
rsx! {
    <ul>
        {for item in items
            <li>{item}</li>
        }
    </ul>
}

// Correct
rsx! {
    <ul>
        {for item in items {
            <li>{item}</li>
        }}
    </ul>
}
```

## Class Errors

### Unsupported Static Class in Strict Mode

`rsx_strict!` rejects unsupported static classes:

```rust
rsx_strict! {
    <div class="md:flex" />
}
```

Use a supported class, a direct GPUI method, `when`, or `rsx_permissive!` if ignoring unsupported classes is intentional.

### Invalid Arbitrary Length

Arbitrary length classes must use supported units and finite values:

```rust
// Correct
rsx! { <div class="w-[280px] max-w-[37.5%] gap-[14px]" /> }
```

Sizing supports `px`, `rem`, percentages, and fractions. Spacing supports definite lengths such as `px` and `rem`; percentage spacing is rejected.

### Invalid Arbitrary Color

Use supported hex, RGB, or RGBA forms:

```rust
rsx! {
    <div class="bg-[#ff0000] text-[rgb(15,23,42)] border-[rgba(15,23,42,0.35)]" />
}
```

RGBA alpha must be in the `0.0..=1.0` range.

### Dynamic Class Is Ignored

Dynamic classes use a runtime matcher. Unsupported tokens are ignored in permissive mode and warn once per call site in debug builds.

If a dynamic class must always be applied, prefer one of these:

```rust
rsx! {
    <div
        class="flex"
        w={px(width)}
        whenClass={(active, "bg-blue-500 text-white")}
    />
}
```

Use `rsx_strict!` to panic when an unsupported dynamic token is evaluated.

### State Class Attribute Uses a Dynamic Value

`hoverClass`, `focusClass`, and `activeClass` are compile-time helpers and only accept string literals:

```rust
// Wrong
let classes = "bg-blue-600";
rsx! { <button hoverClass={classes} /> }

// Correct
rsx! { <button hoverClass="bg-blue-600" /> }
```

Use `hover`, `focus`, or `active` directly when the refinement needs Rust logic:

```rust
rsx! {
    <button hover={|style| style.bg(rgb(0x2563eb))} />
}
```

### State Class Contains an Element-Level Class

State class attributes receive a GPUI `StyleRefinement`, so element-level classes such as
`overflow-scroll` or `debug-outline` cannot be applied there:

```rust
// Wrong
rsx! { <button activeClass="overflow-scroll" /> }
```

Put always-on element behavior on the main `class`:

```rust
rsx! {
    <button
        class="overflow-scroll"
        activeClass="opacity-75"
    />
}
```

For conditional element-level behavior, provide an explicit `id` and call GPUI through `when`:

```rust
rsx! {
    <button
        id="results"
        when={(is_scrollable, |el| el.overflow_scroll())}
        activeClass="opacity-75"
    />
}
```

## Conditional Attribute Errors

### `when`, `whenSome`, or `whenClass` Tuple Shape

These attributes expect exactly two tuple values:

```rust
rsx! {
    <div when={(active, |el| el.bg(rgb(0x3b82f6)))} />
    <div whenSome={(width, |el, w| el.w(px(w)))} />
    <div whenClass={(active, "bg-blue-500 text-white")} />
}
```

### `whenClass` Requires a String Literal

This is invalid:

```rust
let classes = "bg-blue-500";
rsx! {
    <div whenClass={(active, classes)} />
}
```

Use `when` for dynamic styling, or put the literal directly in `whenClass`.

### `whenClass` Rejects Stateful Classes

`overflow-scroll`, `overflow-x-scroll`, and `overflow-y-scroll` need ID semantics. Use `when` with explicit GPUI calls:

```rust
rsx! {
    <div when={(scrollable, |el| el.overflow_scroll())} />
}
```

## ID and Loop Errors

### Stateful Element in Loop Has No `id` or `key`

Repeated stateful elements need unique IDs:

```rust
// Wrong
rsx! {
    <ul>
        {for item in &self.items {
            <li onClick={handler}>{item.name.as_str()}</li>
        }}
    </ul>
}

// Correct
rsx! {
    <ul>
        {for item in &self.items {
            <li key={item.id} onClick={handler}>{item.name.as_str()}</li>
        }}
    </ul>
}
```

`key` only matters when the element needs an ID. On plain layout rows, it is ignored.

### Auto ID Changed After Refactor

Auto IDs include source file, line, and column. If identity must survive moving code, use an explicit `id`:

```rust
rsx! {
    <button id="settings-save" onClick={handler}>
        {"Save"}
    </button>
}
```

## Type Errors

### `IntoElement` Is Not Implemented

Children must be valid GPUI child values. Handle `Option` values explicitly:

```rust
rsx! {
    <div>{self.optional_text.as_deref().unwrap_or("")}</div>
}
```

Or render conditionally:

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

### Cannot Move Out of `self`

Iterate by reference:

```rust
rsx! {
    <ul>
        {for item in &self.items {
            <li>{item.name.as_str()}</li>
        }}
    </ul>
}
```

### Mixed Fragment Root Types

Fragments expand to `vec![...]`. If root element concrete types differ, wrap them or erase the type:

```rust
rsx! {
    <div>
        {self.render_header()}
        {self.render_custom_button()}
    </div>
}
```

## Runtime Issues

### UI Does Not Update

After mutating view state, call `cx.notify()` once:

```rust
onClick={cx.listener(|view, _, _window, cx| {
    view.count += 1;
    cx.notify();
})}
```

Also check that the `rsx!` expression is returned from `render` without a trailing semicolon:

```rust
fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    rsx! { <div>{"Content"}</div> }
}
```

### `dx serve --hot-patch` Ignores RSX Edits

Dioxus CLI 0.7.10 recognizes macros named `rsx!` or `render!` by the last segment
of their path. This also matches `gpui_rsx::rsx!`. Its template parser rejects
GPUI-RSX's HTML-like syntax, and the CLI skips that macro instead of falling back
to Rust compilation. GPUI's Subsecond integration therefore receives no Rust patch
for an edit confined to that macro.

Use a local alias to make the edit follow the Rust compilation path:

```rust
use gpui::prelude::*;
use gpui_rsx::rsx as gpui_view;

fn content() -> impl gpui::IntoElement {
    gpui_view! { <div>{"Edit this text"}</div> }
}
```

Restart `dx serve` after changing the import and call sites. Keep hot reloading
enabled: in CLI 0.7.10, `--hot-reload false` disables processing file changes,
including automatic Rust patches. The alias preserves the selected macro's class
handling mode. The default `rsx!` is permissive; for strict checking, import
`gpui_rsx::rsx_strict as gpui_view` instead.

The alias addresses change detection. Applying the patch also requires a GPUI
version with Subsecond support and an application feature forwarding to
`gpui/hot-reload`, enabled with
`dx serve --hot-patch --hot-reload true --features hotreload`.
Check `dx --version` and distinguish a missing rebuild from a generated patch
that fails to update the window. These are separate stages.

The standalone fixture in `tests/fixtures/hot-reload/` pins GPUI Pre 0.3.11 and
Dioxus 0.7.10. Its README provides parser tests and native builder, original macro
and alias comparisons. It is separate from the demo's verified dependency set.
Its `kit` feature checks Kit 0.9.0 initialization and root-window mounting. From
the repository root, `node scripts/test-hot-reload.mjs` uses Dioxus CLI 0.7.10 to
edit a temporary copy and check revision, PID and timer state in native render
callbacks. It does not click controls or compare pixels. The development CLI's
`doctor --hot-reload --features hotreload,kit` checks readiness separately;
passing that check does not mean a patch was applied.
The existing [build/restart workflow](../performance/#measure-development-reloads)
can also reload edits by restarting the process, which resets application state.

### Events Do Not Fire

Check the handler signature expected by the GPUI method you are calling. For repeated stateful event targets, ensure the element has a `key` or explicit `id`.

### `groupDragOver` Is Rejected

`groupDragOver` is not exposed as an RSX attribute because GPUI needs the drag data type at the
call site:

```rust
// Wrong
rsx! {
    <div groupDragOver={("items", |style| style.opacity(0.75))} />
}
```

Call GPUI directly through `when` or `base` so the type parameter is explicit:

```rust
rsx! {
    <div
        when={(drag_enabled, |el| {
            el.group_drag_over::<MyDragData>("items", |style| style.opacity(0.75))
        })}
    />
}
```

### Styling Does Not Apply

Use `rsx_expand!` to inspect generated methods:

```rust
let preview = gpui_rsx::rsx_expand! {
    <div class="flex gap-4 bg-blue-500" />
};
```

Remember that GPUI-RSX is not Tailwind CSS. State variants such as `hover:bg-blue-500` map to GPUI's `.hover(...)`, but responsive and media variants such as `md:` and `dark:` have no GPUI equivalent; compute the condition and use `when` instead.

## Dependency Issues

### Duplicate GPUI Crates

If `gpui-component` types do not match your application types, inspect the dependency tree:

```bash
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui-pre
```

Use the single Kit dependency shown in [Compatibility](/gpui-rsx/compatibility/) and import components through `gpui::component`. In this repository, `cargo xtask check-gpui` validates the matching snapshot and shared engine types.

### Demo Check Fails After Updating Dependencies

Run:

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
```

If `--locked` fails because the lockfile needs changes, decide whether the GPUI target revision should intentionally move. Do not update the lockfile casually for application compatibility checks.

## Getting Help

When filing an issue, include:

- GPUI-RSX version.
- The resolved GPUI snapshot version and source from `cargo tree -i gpui-pre`.
- Minimal RSX snippet.
- Full compiler error or runtime symptom.
- Whether the issue reproduces in the `demo/` crate.
