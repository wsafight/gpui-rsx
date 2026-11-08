---
title: Migration Guide
description: Upgrade GPUI-RSX, adopt RSX in existing views, and align GPUI dependencies.
---


Start with the section for your current version. Dependency migration from Zed Git and builder-to-RSX examples follow the version notes.

## From 0.11 to 0.12

Existing RSX markup and pixel/Tailwind spacing modes are unchanged. Macro/core still require Rust 1.88; CLI/tooling require Rust 1.98.

`gpui-rsx-tools::Diagnostic` adds `code: Option<String>`. Add `code: None` to existing struct literals, and use `..` when destructuring if the code is not needed. Old JSON without `code` still deserializes; uncategorized diagnostics omit it when serialized. JSON compatibility does not make this Rust source change patch-compatible, so this release is 0.12.0 rather than 0.11.1.

Direct macro imports such as `use gpui_rsx::rsx as view` retain their original spacing mode in tooling. Balanced macros can still be analyzed while surrounding Rust is incomplete, but formatting and migration refuse to write lexically damaged files. Local `macro_rules!` definitions shadow unqualified calls; crate aliases and re-exports are not full Rust name resolution.

The demo targets Kit 0.9.0 / GPUI Pre 0.3.11. Its large incident queue is virtualized, and native profiling is opt-in. New hot-reload diagnostics and probes do not establish that every Dioxus/Kit hot-patch configuration works; see the [development reload workflow](/gpui-rsx/guides/performance/#measure-development-reloads) for the tested configuration and limits.

## From 0.10 to 0.11

Existing markup can keep its current syntax. The main changes are negative spacing, configurable GPUI paths, and optional state-update helpers.

- Negative classes such as `-mt-4` and `-ml-[8px]` now resolve in static and dynamic classes. Numeric spacing still defaults to pixels. GPUI clamps negative widths, heights, and maximum sizes to zero and ignores negative minimum sizes; static uses of these size classes produce a warning.
- Set `[options] gpui-path = "your_alias"` in `rsx.toml` if your GPUI dependency has a different crate name. The setting also applies to components, IDs, listener/update helpers, and the default shared runtime. The demo keeps the `gpui` alias for Kit.
- `rsx_listener!(cx, handler)` preserves the native callback signature and leaves notification to the handler. `rsx_listener!(cx, notify, handler)` notifies after the handler returns normally. Remove the matching manual `cx.notify()` when using `notify`.
- `rsx_update!(view, cx, update)` runs a synchronous update and notifies after normal return. Use it with the listener form that leaves notification to the handler to avoid notifying twice.

The [counter demo](/gpui-rsx/examples/demos/) shows both notification styles. The [development reload workflow](/gpui-rsx/guides/performance/#measure-development-reloads) rebuilds and restarts the application; each restart resets state.

Tool integrations should also check that sibling diagnostics are handled independently. Static negative-size warnings remain warnings in strict mode.

## From 0.9 to 0.10

A root fragment that contains a `for` loop or `{...spread}` now expands to `Vec<AnyElement>`; previously it built a
`vec!` of iterators that did not compile with mixed children. Deprecating a component with
`#[deprecated]` now warns where the component is used, not inside its generated code.

`#[component]` props typed `Rc<dyn Fn...>` now take `impl Fn... + 'static`, matching
`Box<dyn Fn...>` props and GPUI's event setters, so `on_press={|_, _, _| ...}` infers its
arguments. Code that passed an existing `Rc` callback forwards it instead:

```rust
<Row on_select={{ let on_select = on_select.clone(); move |item, w, cx| on_select(item, w, cx) }} />
```

Auto IDs on component constructors and `base` expressions now require both
`InteractiveElement` and `Element`. Native element factories retain auto IDs; builders
such as gpui-component's `Button` and callback props named `on_click` keep their own setters.
Generic and opaque element types must expose the required bounds; see [IDs and Keys](/gpui-rsx/usage/ids/).

Permissive static CSS-only classes (`z-*`, `space-x-*`, `space-y-*`, `divide-*`,
`transition*`, `duration-*`, `ease-*`, `delay-*`, `animate-*`) now warn and are ignored
instead of being passed through to extension methods. If an extension implements these
methods, replace `class="z-10"` with the explicit flag `z_10`, or call it in `base={...}`.
Strict mode rejects these classes.

Breaking changes for `gpui-rsx-tools` integrations:

- `Diagnostic::severity` is a `DiagnosticSeverity` enum instead of a `String`. JSON output is
  unchanged (`"error"`, `"warning"`, `"information"`). Use `severity.as_str()` or
  `to_string()` where a string is needed, and add a fallback arm when matching, because the
  enum is `#[non_exhaustive]`.
- Whole-file analysis (`analyze_rust`, `analyze_rust_document`, `update_rust_document`) now
  includes RSX macros nested inside another macro's body. `macro_calls` still returns only
  top-level calls, whose ranges never overlap; use `nested_macro_calls` for every call and
  `call_at` to find the innermost call at an offset.
- `report_body` returns diagnostics, expansion and preview from one parse. `BodyReport` is
  `#[non_exhaustive]`, so read its fields instead of constructing it. Its expansion, like
  the new `expand_summary`, omits the dynamic class helper; `expand` still prints it in full.

The CLI rejects options a command does not use (for example `lint --port`) with exit
status 2, and `lint` checks explicitly named files whatever their extension. Tools detect
the `tailwind-scale` feature of `gpui-rsx` themselves, so `--tailwind` is no longer needed
for projects that enable it.

`RustAnalysis` also stores `components` and `class_tokens`. Construct it with
`RustAnalysis::default()` and field updates, or use `analyze_rust_document`, instead of
listing every field. Class token offsets remain UTF-8 bytes and classifications/colors
can be reused by hosts. `update_rust_document` requires the same mode as the previous
analysis; configuration changes must trigger a full analysis.

Dynamic integer keys now have GPUI's named-integer representation. Use an explicit `id`
if other code inspects or persists the old formatted representation. Keys still depend on
source location; this change does not promise identity across edits.

`convert(source)` keeps its pixel-body output. To preserve Tailwind values, set
`ConversionOptions::default().tailwind = true` and call `convert_with_options`; its `rsx`
field then contains a full `rsx_tw!` expression. WASM adds `analyze_with_theme` and
`convert_with_options` while retaining the original exports.

Conversion now preserves simple expressions such as `{items}` and `{item.title}` without
a manual-conversion diagnostic. It checks syntax only: review Rust types and ownership.
A collection used as children may need `{...items}` or `{for ...}`; preserving `{items}`
does not convert JavaScript rendering semantics into GPUI children. Calls, operators,
indexing, optional chaining and handlers still produce review diagnostics.

## From 0.8 to 0.9

Version 0.9 targets GPUI Kit 0.8.1 and keeps pixel spacing as the default.
Existing RSX syntax and generated IDs keep their behavior. Tool integrations should use
the `gpui-rsx-core` crate-root facade or `syntax` module, replace struct literals with
`ClassMode::builder()`, `Theme::builder()` and `MacroOptions::new().with_*()`, and clone
`ClassMode` when passing it more than once. Theme handles now use `Arc<Theme>`; syntax,
class outcome and token enums require a fallback match arm.

`gpui-rsx-tools` no longer enables SWC conversion by default. Libraries that call the
converter must opt in with `features = ["convert"]`; the CLI and WASM playground already
do so. Formatting, linting and preview work without SWC.

New `rsx.toml` sections are `text-size`, `line-height`, `opacity` and `shadow`. Opacity
uses a quoted fraction or percentage, such as `"0.6"` or `"60%"`; shadow values use
`{ expr = "..." }`. Duplicate keys and conflicting `text-size-*` classes now produce
configuration errors. `cargo rsx theme check/list/explain` supports `--json`, with
validation failures returning exit status 1 and usage errors returning 2.

The LSP advertises incremental sync (`change: 2`) and semantic tokens. Full replacements
are still accepted; range changes use UTF-16 positions and apply in order. Unchanged
macro bodies reuse their diagnostics. Dynamic classes within one RSX invocation share
one helper; expressions and unknown-class handlers remain local to each attribute.

## Zed Git to GPUI Kit 0.8

Replace the separate Zed Git `gpui`, `gpui_platform` and Git `gpui-component`
dependencies with Kit, preserving the `gpui` crate name for generated code:

```toml
[dependencies]
gpui = { package = "gpui-kit", version = "=0.9.0" }
gpui-rsx = "0.12.0"
```

Kit 0.9.0 pins `gpui-pre = 0.3.11`. Use `gpui::component` for styled controls and
`gpui::base` for unstyled behavior. Replace `gpui_platform::application()` with
`gpui::application()`, initialize Kit before opening a window, and mount its root:

```rust
gpui::application().with_assets(gpui::assets::Assets).run(|cx| {
    gpui::init(cx);
    gpui::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
        cx.new(|_| MyView)
    }).expect("failed to open window");
});
```

Regenerate and commit the application lockfile after the dependency change. The
repository demo uses Rust 1.98.1; the macro/core MSRV remains Rust 1.88. Check
`cargo xtask check-gpui` and the real demo bins before merging the update. GPUI
Fast is a separate optional backend, outside the current compatibility target.

The `gpui-component` theme preset now emits `cx.theme().colors.<field>`. This
keeps colors such as `list` distinct from Kit settings such as `Theme.list`.
Import `gpui::component::ActiveTheme` when using the preset.

The 0.5-to-0.6 section below documents the historical Zed Git target; use the
current [compatibility matrix](/gpui-rsx/compatibility/) for new dependencies.

## Spacing scale, state variants and components

Numeric spacing/sizing remains in pixels by default: `gap-4` is 4px. To use
Tailwind units (4px per unit), choose `rsx_tw!`, enable the `tailwind-scale` Cargo
feature, or put the following in an `rsx.toml` next to your crate's `Cargo.toml`:

```toml
[options]
spacing-scale = "tailwind"
```

| Class | Default | Tailwind scale |
| --- | --- | --- |
| `gap-4` | 4px | 16px |
| `p-2.5` | 2.5px | 10px |
| `w-64` | 64px | 256px |
| `gap-[14px]` | 14px | 14px |

To preserve an existing layout when switching, replace numeric values with explicit
pixels (`gap-4` → `gap-[4px]`) or divide by four (`gap-4` → `gap-1`). Check static
literals, conditional branches and dynamic strings. Typed attributes (`gap={px(4.0)}`),
fractions, arbitrary values, colors, opacity and GPUI presets (`rounded-md`, `border-2`)
are unchanged. Prefer crate-local `rsx.toml` in libraries because Cargo features unify
across the dependency graph.

Static class literals now accept `hover:`, `focus:`, `focus-visible:`, `active:`,
`group-hover:` and `group-active:` (including named groups). Stateful variants inject
IDs; supply stable keys or explicit unique IDs in loops. Responsive and `dark:` variants emit warnings in
permissive mode and errors in strict mode. Dynamic class strings cannot use variants
or theme tokens. `onUnknownClass={|class: &str| ...}` handles unsupported dynamic classes
without the strict mode's default panic; `dynamic-class-report` collects debug reports
until process exit.

`#[component]` generates a props builder for named structs. Plain fields are required;
`#[prop(default)]` and `#[prop(default = expr)]` are optional. A `#[children]` field
collects children. Third-party constructors continue to work with `base={...}`.

Theme tokens in `rsx.toml` support `[colors]`, `[radius]` and `[spacing]`. Use
`[options] preset = "gpui-component"` for semantic colors; import the theme extension
trait and keep `cx` in scope. Tokens with `{ expr = "cx.theme().colors.background" }` read the
current theme during rendering. Literal hex tokens remain fixed colors.

## From Manual GPUI to RSX

GPUI-RSX can be adopted incrementally. Manual GPUI builders and RSX output are both normal GPUI elements, so they can live side by side.

### Elements and Attributes

Before:

```rust
div()
    .flex()
    .flex_col()
    .gap(px(16.0))
    .bg(rgb(0x3b82f6))
```

After:

```rust
rsx! {
    <div class="flex flex-col gap-[16px] bg-blue-500" />
}
```

Use direct attributes when the source code already has a typed Rust value:

```rust
rsx! {
    <div w={px(width)} bg={self.background_color()} />
}
```

### Children

Before:

```rust
div()
    .child("Hello")
    .child(div().child("World"))
    .child(button().child("Click"))
```

After:

```rust
rsx! {
    <div>
        "Hello"
        <span>{"World"}</span>
        <button>{"Click"}</button>
    </div>
}
```

### Events

Before:

```rust
button()
    .on_click(cx.listener(|view, _event, _window, cx| {
        view.count += 1;
        cx.notify();
    }))
    .child("Click")
```

After:

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

### Lists

Before:

```rust
let mut list = div().flex().flex_col();

for item in &self.items {
    list = list.child(div().child(item.name.clone()));
}

list
```

After:

```rust
rsx! {
    <div class="flex flex-col">
        {for item in &self.items {
            <div>{item.name.as_str()}</div>
        }}
    </div>
}
```

If rows are stateful, add `key={...}`:

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

### Conditional Rendering

Use Rust conditionals for different children:

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

Use `when` when the same element receives optional builder calls:

```rust
rsx! {
    <div when={(show_message, |el| el.child("Message visible"))} />
}
```

## 0.5.x / crates.io GPUI 0.2.2 to 0.6.0 / Zed Git GPUI

GPUI-RSX `0.6.x` targets GPUI from the Zed repository, not the crates.io `gpui = "0.2.2"` package.

The Zed git dependency still reports the package version as `gpui v0.2.2`, but its API surface is different from the crates.io release. This matters for generated calls such as `flex_grow_1`, `flex_shrink_1`, alignment helpers, desktop sizing utilities, and the newer startup path.

### Replace Dependencies

Before:

```toml
[dependencies]
gpui = "0.2.2"
gpui-rsx = "0.5"
```

After:

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit", "runtime_shaders", "wayland", "x11"] }
gpui-rsx = "0.6"
```

Applications should commit `Cargo.lock` to pin the exact resolved Zed revision.

If your app also depends on `gpui-component`, keep direct `gpui` and `gpui_platform` dependencies on the same source and revision resolved by `gpui-component`. Mixing a bare Zed git dependency with another pinned `rev` can create duplicate GPUI crate instances and incompatible component types.

### Update Application Startup

Before:

```rust
Application::new().run(|cx: &mut App| {
    cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| MyView));
});
```

After:

```rust
use gpui_platform::application;

application().run(|cx: &mut App| {
    cx.open_window(WindowOptions::default(), |_, cx| {
        cx.new(|_| MyView)
    }).unwrap();
    cx.activate(true);
});
```

### Verify the Dependency Graph

For the historical Zed Git target, inspect the application dependency tree. The current demo uses Kit; see the compatibility matrix above:

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui-pre
cargo xtask check-gpui
```

If `cargo tree` shows `git+https://github.com/zed-industries/zed#...`, the project is using Zed git GPUI even if the displayed package version is `0.2.2`.

## Incremental Migration Plan

1. Add `gpui_rsx::rsx` imports in one view module.
2. Convert a small static subtree first.
3. Convert child lists and repeated rows after the static structure compiles.
4. Add `key={...}` only to repeated elements that need stateful IDs.
5. Run `cargo check` after each meaningful step.

Hybrid code is fine:

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

## Common Migration Patterns

| Manual GPUI | GPUI-RSX |
| --- | --- |
| `div().child("Hello")` | `rsx! { <div>{"Hello"}</div> }` |
| `div().child(format!("Count: {count}"))` | `rsx! { <div>{format!("Count: {count}")}</div> }` |
| `.w(px(200.0)).h(px(100.0))` | `<div w={px(200.0)} h={px(100.0)} />` |
| `.flex().gap(px(4.0))` | `<div class="flex gap-4" />` |
| builder constructor with arguments | `<Button base={Button::new("id")} />` |

## Checklist

- [ ] The current Kit release is aliased as `gpui` and uses its matching snapshot.
- [ ] Startup uses `gpui::application()`, `gpui::init(cx)` and `gpui::open_window(...)`.
- [ ] `Cargo.lock` is committed for applications.
- [ ] Components are imported through `gpui::component` and share the GPUI instance.
- [ ] New RSX modules import `gpui_rsx::rsx`.
- [ ] Repeated stateful elements have `key={...}` or explicit `id={...}`.
- [ ] The migrated view passes `cargo check`.
- [ ] Interactions still call `cx.notify()` after state changes.
