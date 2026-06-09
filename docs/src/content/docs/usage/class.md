---
title: Class Handling
description: Static and dynamic class handling, supported utility patterns, and performance guidance.
---

The `class` attribute maps a Tailwind-like utility subset to GPUI builder methods.

```rust
rsx! {
    <div class="flex flex-col gap-4 p-4 bg-blue-500 text-white">
        {"Content"}
    </div>
}
```

## Static Classes

Prefer string literals when the class list is known at compile time. They are parsed by the macro and expand directly to GPUI method calls:

```rust
rsx! {
    <div class="flex items-center gap-2 px-4 py-2 rounded-md" />
}
```

In permissive mode, unsupported static classes that cannot be emitted safely are ignored. Invalid arbitrary values still produce compile errors. In strict mode, unsupported static classes are compile errors:

```rust
use gpui_rsx::rsx_strict;

rsx_strict! {
    <div class="hover:bg-blue-500" />
}
```

## Compile-Time Conditional Classes

`if` and `match` expressions whose branches are string literals can also be optimized by the macro:

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

This keeps conditional styling ergonomic without paying the dynamic-class matcher cost for simple literal branches. The optimization applies to the expression inside `class={...}`; assigning the result to a variable first makes it a normal dynamic class expression.

## Dynamic Classes

Use dynamic classes when the class string is assembled at runtime:

```rust
let class_name = format!("w-[{}px] bg-{}", width, tone);

rsx! {
    <div class={class_name} />
}
```

Dynamic classes use a generated runtime matcher. The matcher supports common layout, spacing, sizing, color, opacity, typography, and border utilities, including arbitrary lengths and colors in the supported formats.

Unknown dynamic classes are ignored in permissive mode. In debug builds, each unknown class warning is emitted only once per generated call site. In strict mode, unsupported dynamic classes panic when evaluated.

| Capability | Static class | Dynamic class |
| --- | --- | --- |
| Layout and alignment | Supported | Supported subset |
| Numeric spacing and sizing | Supported | Supported |
| Arbitrary lengths | Supported | Supported |
| Percent and fraction sizing | Supported | Supported |
| Full Tailwind color palette | Supported | Supported |
| Arbitrary hex / RGB / RGBA colors | Supported | Supported |
| Static stateful scroll classes | Supported with auto ID | Not supported |
| Tailwind variants such as `hover:*` | Ignored or strict error | Ignored or strict panic |

## Supported Utility Areas

| Area | Notes |
| --- | --- |
| Layout | `flex`, `flex-col`, `flex-row`, `flex-wrap`, `grid`, `hidden`, `absolute`, `relative`, grow/shrink helpers. |
| Alignment | `items-*`, `justify-*`, `content-*`, and `self-*` helpers supported by GPUI. |
| Spacing | `gap-*`, `gap-x-*`, `gap-y-*`, `p-*`, `px-*`, `m-*`, and directional variants. |
| Sizing | `w-*`, `h-*`, `size-*`, `min-*`, `max-*`, `*-full`, `*-auto`, percentages, fractions, arbitrary lengths. |
| Colors | Full Tailwind color names plus arbitrary `hex`, `rgb`, and `rgba` forms. |
| Typography | Text sizes, font weights, text alignment, decoration, truncation, whitespace, and line clamp. |
| Borders | Width, directional width, color, rounded utilities, and dashed border helpers. |
| Misc | Cursor helpers, `debug-outline`, shadows, opacity, grid placement aliases. |

## Length Semantics

GPUI-RSX uses direct pixel semantics for numeric utilities:

```rust
rsx! {
    <div class="w-64 gap-4" />
}
```

This expands like `w(px(64.0)).gap(px(4.0))`. It does not use Tailwind CSS's rem-based spacing scale.

Arbitrary length examples:

```rust
rsx! {
    <div class="w-[280px] h-[18rem] max-w-[37.5%] gap-[14px]" />
}
```

Percentages and fractions are supported for sizing families:

```rust
rsx! {
    <aside class="w-6/24 max-w-[40%]" />
}
```

Percentage spacing such as `gap-[10%]` is rejected because GPUI spacing APIs expect definite lengths.

## Color Formats

Named Tailwind colors use `family-shade`:

```rust
rsx! {
    <div class="bg-blue-500 text-white border-gray-300" />
}
```

Supported families are `slate`, `gray`, `zinc`, `neutral`, `stone`, `red`, `orange`, `amber`, `yellow`, `lime`, `green`, `emerald`, `teal`, `cyan`, `sky`, `blue`, `indigo`, `violet`, `purple`, `fuchsia`, `pink`, and `rose`, with shades `50` through `950`, plus `black` and `white`.

Arbitrary colors support hex, short hex, RGB, and RGBA:

```rust
rsx! {
    <div class="bg-[#ff0000] text-[#f00] border-[rgba(15,23,42,0.35)]" />
}
```

## Strict and Preview Macros

Use `rsx_strict!` when unsupported static classes should fail the build:

```rust
use gpui_rsx::{rsx_expand, rsx_permissive, rsx_strict};

rsx_strict! { <div class="flex w-[280px]" /> }
rsx_permissive! { <div class="hover:bg-blue-500 flex" /> }

let preview = rsx_expand! {
    <div class="flex gap-4 bg-blue-500" />
};
```

`rsx_expand!` returns a string preview of the generated GPUI builder chain. It is for debugging macro output and does not type-check the generated GPUI expression.

## Performance Guidance

- Use literal classes for stable UI structure.
- Use literal `if` or `match` branches for small state-based variants.
- Keep runtime-generated classes for genuinely dynamic dimensions or colors.
- Prefer direct GPUI attributes for hot paths when a value is already a Rust expression, for example `w={px(width)}`.
- Use `when` or `whenClass` for small conditional variants instead of concatenating long runtime class strings.

For render-loop allocation, ID/key behavior, and validation commands, see the [Performance guide](/gpui-rsx/guides/performance/).
