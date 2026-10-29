---
title: Performance
description: How GPUI-RSX code expands, where runtime work can appear, and how to keep renders cheap.
---

GPUI-RSX is designed so the common path is ordinary GPUI code generated at compile time. Most performance work is therefore about keeping markup static where possible and avoiding unnecessary allocation or state churn in render methods.

## Mental Model

Static RSX expands during macro execution:

```rust
rsx! {
    <div class="flex gap-4 bg-blue-500">
        {"Hello"}
    </div>
}
```

The generated expression is a GPUI builder chain, similar to:

```rust
div()
    .flex()
    .gap(px(4.0))
    .bg(rgb(0x3b82f6))
    .child("Hello")
```

Static tags, attributes, class strings, colors, and many layout utilities are resolved before your application runs.

## Prefer Static Class Strings

Use literal `class` values for stable UI structure:

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4 bg-white border border-gray-200 rounded-md" />
}
```

Literal classes have no runtime parser cost. They also give strict mode the most useful compile-time diagnostics.

## Keep Conditional Classes Compile-Time

`if` and `match` expressions directly inside `class={...}` can be optimized when every branch is a string literal:

```rust
rsx! {
    <div class={if active { "bg-blue-500 text-white" } else { "bg-gray-100 text-gray-900" }} />
}

rsx! {
    <div class={match state {
        State::Ready => "border-green-500",
        State::Error => "border-red-500",
        _ => "border-gray-300",
    }} />
}
```

This optimization applies to the expression passed to `class`. If you assign the result to a variable first, the macro sees a runtime expression and uses the dynamic matcher.

## Static edges of format templates

`class={format!("p-{} bg-red-500", n)}` expands `bg-red-500` at compile time and parses
only the dynamic portion at runtime. The original `format!` still runs once, including
named arguments and formatting specifications; this optimization does not remove its String
allocation. Complete static tokens before the first dynamic token and after the last are
eligible. Static tokens between dynamic tokens remain in the runtime string because values
can contain spaces. Class application order and expression side effects are preserved.

## Use Dynamic Classes Sparingly

Dynamic classes are useful when the class string is genuinely assembled at runtime:

```rust
let class_name = format!("w-[{}px] bg-{}", width, tone);

rsx! {
    <div class={class_name} />
}
```

That path generates a runtime matcher over the class tokens. It supports common layout, spacing, sizing, colors, opacity, typography, borders, arbitrary lengths, and arbitrary colors, but it is still work done during rendering.

Prefer direct GPUI attributes when values are already Rust expressions:

```rust
rsx! {
    <aside class="flex flex-col min-w-0" w={px(self.sidebar_width)} />
}
```

Prefer `when` or `whenClass` for small state variants:

```rust
rsx! {
    <button
        class="px-3 py-2 rounded-md"
        whenClass={(selected, "bg-blue-500 text-white")}
        whenClass={(!selected, "bg-gray-100 text-gray-900")}
    >
        {label.as_str()}
    </button>
}
```

## Share the Dynamic Class Runtime

By default, each macro invocation with a dynamic class generates its own matcher.
For projects with many such call sites, generate the matcher once at the crate root:

```rust title="src/lib.rs"
gpui_rsx::rsx_runtime!();
```

Enable it for the package:

```toml title="rsx.toml"
[options]
dynamic-runtime = "rsx_runtime"
```

The generated public module contains `px` and `tailwind` helpers. Both operate on
GPUI's `StyleRefinement`, so different element types share the same compiled code.
`rsx!` and `rsx_tw!` choose the appropriate helper. Existing styles and per-element
`onUnknownClass` callbacks are preserved, and strict mode still rejects unknown
dynamic classes at runtime. Variants and theme tokens still require static strings.
This setting reduces compilation work; dynamic strings are still parsed during rendering.

Library code uses `crate::rsx_runtime`; integration tests, examples and binaries in
the same package automatically use the library's module, including a custom `[lib].name`.
In a package with only binaries, put `rsx_runtime!()` at each target's crate root,
including integration tests and examples that use dynamic classes. An explicit
`dynamic-runtime = "crate::rsx_runtime"` always selects the current target, and
`"::my_library::rsx_runtime"` can select an existing module directly.
If GPUI is renamed, pass its import path: `gpui_rsx::rsx_runtime!(my_gpui);`.

This optimization is optional in 0.10. Without `dynamic-runtime`, generated code
keeps the existing behavior. Lint and LSP information diagnostics suggest the option
when they encounter dynamic classes without a configured shared runtime.

A local macOS ARM64 / Rust 1.99.1 comparison used 50 public functions, each with
one `class={classes}` attribute, against this repository's GPUI stub. Dependencies
were built first, incremental compilation was disabled, and only the application
source was touched before each measurement (one run per command):

| Workload | Inline helper | Shared runtime |
| --- | --- | --- |
| `cargo check` | 2.180 s | 0.259 s |
| Debug build | 4.112 s | 0.386 s |
| Release build | 12.841 s | 0.885 s |
| Debug `.rlib` | 11.86 MB | 2.06 MB |

This isolates generated-code compilation costs. Real application cold builds also
compile GPUI and other dependencies, so the relative whole-build gain will differ.

## Avoid Render-Loop Allocation

Renders can run often. Keep per-render allocation intentional:

```rust
rsx! {
    <ul>
        {for item in &self.items {
            <li>{item.name.as_str()}</li>
        }}
    </ul>
}
```

Avoid cloning large strings or building repeated formatted class names inside loops unless the value is genuinely needed.

Small `format!` calls for visible text are normal:

```rust
rsx! {
    <span>{format!("Count: {}", self.count)}</span>
}
```

For repeated labels, prefer borrowed strings or precomputed display text.

## Batch State Notifications

State updates should usually trigger one render notification:

```rust
onClick={cx.listener(|view, _, _window, cx| {
    view.count += 1;
    view.last_update = Instant::now();
    view.recompute_summary();
    cx.notify();
})}
```

Avoid calling `cx.notify()` after each individual field assignment.

## Understand ID and Key Cost

Auto IDs are generated only for elements that need GPUI stateful identity. Common triggers include
`onClick`, `onHover`, `onDrag`, `onAuxClick`, `onA11yAction`, `active`, `activeClass`,
`groupActive`, `tooltip`, `hoverableTooltip`, `tooltipShowDelay`, `focusable`, `role`, `aria*`,
`trackScroll`, `overflowScroll`, `overflowXScroll`, `overflowYScroll`, and static
`overflow-scroll` class variants.

`hover`, `hoverClass`, `focus`, `focusClass`, `group`, `groupHover`, and `scrollbarWidth` are
style-refinement paths and do not trigger auto ID injection by themselves.

Plain layout elements do not become stateful just because they have `key={...}`:

```rust
rsx! {
    <div key={task.id} class="flex items-center">
        {task.title.as_str()}
    </div>
}
```

The `key` above is ignored because the element does not need an ID. Add `key` in loops when the repeated element is stateful:

```rust
rsx! {
    <button key={task.id} onClick={handler}>
        {task.title.as_str()}
    </button>
}
```

Literal keys use `concat!`. Concrete dynamic integer values up to 64 bits (including
`usize` / `isize`) use GPUI's `(&'static str, u64)` ID without formatting or allocating a
string. Other `Display` values retain the formatted-string fallback, including `u128` /
`i128` to avoid truncation. Borrowed integer references and generic `T: Display` values may
also use that fallback. The expression is borrowed and evaluated once, so a String key is
not moved. Prefer an owned integer value such as `key={task.id}` for large stateful lists.

## Keep RSX Blocks Scannable

Large macro inputs increase compile-time work and make compiler errors harder to localize. Split views around real UI concepts:

```rust
fn render_sidebar(&self) -> impl IntoElement {
    rsx! {
        <aside class="flex flex-col w-[280px] min-w-[280px] border-r border-gray-200">
            {self.render_sidebar_header()}
            {self.render_project_list()}
        </aside>
    }
}
```

This is primarily a maintainability move, but it also keeps incremental edits smaller.

## Inspect Generated Code

Use `rsx_expand!` to check whether a class path stayed static or went through the dynamic matcher:

```rust
let preview = gpui_rsx::rsx_expand! {
    <div class={if active { "flex gap-4" } else { "block" }} />
};
```

For full compiler validation, use `cargo check`.

## Verify Performance-Sensitive Changes

### 0.9 development measurements

On macOS ARM64 with Rust 1.99.1, short Criterion runs (10 samples, 1 second warmup
and measurement) produced these slope estimates:

| Workload | Before | After |
| --- | --- | --- |
| Static dashboard macro expansion | 66.8 µs | 66.7 µs |
| One dynamic class attribute | 489 µs | 477 µs |
| Four dynamic class attributes in one invocation | 1.99 ms | 0.495 ms |
| Analyze 80 macros / reuse 79 unchanged bodies | 1.00 ms (full) | 223 µs (reuse) |

These are local microbenchmarks, not whole-project cold build timings. The reuse
estimate had a 95% confidence interval of about 199–268 µs. That earlier incremental implementation scanned the Rust token stream and skipped RSX
parsing only for unchanged bodies; the 0.10 measurements below include confined-body scans.
Dynamic helpers are shared within a macro invocation, while each attribute keeps
its own expression evaluation and unknown-class handler.

```bash
cargo bench -p gpui-rsx-tools --bench analysis --no-default-features -- \
  --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo bench -p gpui-rsx-core --bench expansion -- \
  --save-baseline before09 --sample-size 10 --warm-up-time 1 --measurement-time 1
# After applying the helper-sharing change, using the same benchmark inputs:
cargo bench -p gpui-rsx-core --bench expansion -- \
  --baseline before09 --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Use the same benchmark harness on both revisions and retain Criterion's report
directory when comparing them. Longer runs on an idle machine are appropriate
before making a release performance claim.

The repository includes a benchmark target for class parsing/code generation:

```bash
cargo bench --bench class_performance --no-run
```

For real GPUI integration, check the demo crate:

```bash
cargo check --manifest-path demo/Cargo.toml --bins --locked
```

For documentation examples and navigation, build the docs site:

```bash
cd docs
bun run build
```

## Editor and matcher measurements (2026-10-29)

On this macOS ARM64 machine, a 5000-line file with 50 macros took about **1.12ms** for
full analysis and **0.15ms** when editing one balanced macro. The fixture includes ordinary
comment lines and small RSX bodies; it is not a bound on arbitrary Rust files. Reproduce with
`cargo bench -p gpui-rsx-tools --bench analysis -- 5000_lines`. Edits outside a balanced
macro use a full scan; files without `rsx` skip analysis, and the LSP coalesces queued edits.
Class classifications and colors are cached for semantic-token and color requests.

The common runtime matcher now dispatches by first byte. A short 10-sample stub benchmark
showed `dynamic_class_simple` at about 16→14ns and `dynamic_class_complex` at 129→89ns;
`gap-4` alone regressed from about 9→11ns and numeric fallback stayed near 13ns. These are
small synthetic workloads, not a uniform speedup guarantee. Reproduce with
`cargo bench --bench class_performance -- dynamic_`.
