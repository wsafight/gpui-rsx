//! # gpui-rsx
//!
//! A procedural macro that brings JSX-like syntax to [GPUI], making UI code more concise
//! and readable while generating **zero-overhead** native GPUI method chains at compile time.
//!
//! [GPUI]: https://www.gpui.rs/
//!
//! ## At a Glance
//!
//! ```ignore
//! use gpui::*;
//! use gpui::prelude::*;
//! use gpui_rsx::rsx;
//!
//! // Before — verbose GPUI method chain
//! div()
//!     .flex()
//!     .flex_col()
//!     .gap(px(16.0))
//!     .p(px(16.0))
//!     .bg(rgb(0x3b82f6))
//!     .child(div().text_xl().font_weight(FontWeight::BOLD).child("Hello GPUI"))
//!     .child(
//!         div()
//!             .cursor_pointer()
//!             .id("btn")
//!             .on_click(cx.listener(|_, _, _window, cx| cx.notify()))
//!             .child("Click me"),
//!     )
//!
//! // After — concise RSX (~50% less code)
//! rsx! {
//!     <div class="flex flex-col gap-4 p-4 bg-blue-500">
//!         <h1 class="text-xl font-bold">{"Hello GPUI"}</h1>
//!         <button
//!             cursor_pointer
//!             onClick={cx.listener(|_, _, _window, cx| cx.notify())}
//!         >
//!             {"Click me"}
//!         </button>
//!     </div>
//! }
//! ```
//!
//! The two snippets produce **identical** compiled output.
//!
//! ## Features
//!
//! | Feature | Description |
//! |---------|-------------|
//! | HTML-like tags | `<div>`, `<span>`, `<button>`, … → all map to `div()` |
//! | Boolean attributes | `<div flex flex_col />` → `.flex().flex_col()` |
//! | Value attributes | `<div gap={px(4.0)} />` → `.gap(px(4.0))` |
//! | `class` (static) | Tailwind-like subset, parsed at compile time |
//! | `class` (dynamic) | Runtime expression with colors, sizing, spacing, and common utilities |
//! | Full color palette | 242 Tailwind colors + arbitrary hex/RGB/RGBA (`bg-[#ff0000]`) |
//! | Desktop sizing | `w-[280px]`, `w-[37.5%]`, `w-6/24`, `min-w-0` |
//! | Fragments | `<>...</>` — returns `vec![...]` |
//! | For-loop sugar | `{for item in iter { ... }}` |
//! | Spread | `{...iterator}` |
//! | Conditional | `when` / `whenSome` attributes |
//! | State classes | `hoverClass`, `focusClass`, `activeClass` |
//! | Styled defaults | `<h1 styled>` injects sensible tag defaults |
//! | camelCase mapping | `onClick` → `.on_click()`, `fontSize` → `.text_size()` |
//! | Custom constructors | `base={Button::new("id")}` starts a component method chain |
//! | `key` (loop ID) | Composite auto-ID for stateful elements in loops |
//!
//! ## Syntax Reference
//!
//! ### Basic Elements
//!
//! Any HTML tag maps to `div()`. Self-closing and pair tags are both valid:
//!
//! ```ignore
//! rsx! { <div /> }                    // div()
//! rsx! { <span></span> }              // div()
//! rsx! { <button>{"OK"}</button> }    // div().child("OK")
//! ```
//!
//! ### Fragment
//!
//! Return multiple root elements without a wrapper. Expands to `vec![...]`:
//!
//! ```ignore
//! rsx! {
//!     <>
//!         <div>{"First"}</div>
//!         <div>{"Second"}</div>
//!     </>
//! }
//! // → vec![div().child("First"), div().child("Second")]
//! ```
//!
//! ### Boolean Attributes (Flags)
//!
//! Bare attribute names become zero-argument method calls:
//!
//! ```ignore
//! rsx! { <div flex flex_col items_center /> }
//! // → div().flex().flex_col().items_center()
//! ```
//!
//! ### Value Attributes
//!
//! `name={expr}` passes the expression as the method argument:
//!
//! ```ignore
//! rsx! { <div gap={px(16.0)} bg={rgb(0x3b82f6)} opacity={0.8} /> }
//! // → div().gap(px(16.0)).bg(rgb(0x3b82f6)).opacity(0.8)
//! ```
//!
//! ### Custom Constructor with `base`
//!
//! `base={expr}` replaces the constructor inferred from the tag name, then the remaining
//! attributes continue as a normal method chain:
//!
//! ```ignore
//! rsx! { <Button base={Button::new("save")} label={"Save"} small /> }
//! // → Button::new("save").label("Save").small()
//! ```
//!
//! Path-qualified tags are supported for module-scoped components:
//!
//! ```ignore
//! rsx! { <ui::TaskCard base={ui::TaskCard::new(task.id)} title={task.title.clone()} /> }
//! ```
//!
//! ### The `class` Attribute — Static (Compile-time, Recommended)
//!
//! A Tailwind-inspired subset that expands to method calls at compile time.
//! It maps directly to GPUI APIs and is not a full Tailwind CSS engine.
//!
//! ```ignore
//! rsx! { <div class="flex flex-col gap-4 p-4 bg-blue-500 text-white rounded-md" /> }
//! // → div().flex().flex_col().gap(px(4.0)).p(px(4.0)).bg(rgb(0x3b82f6))
//! //         .text_color(rgb(0xffffff)).rounded_md()
//! ```
//!
//! #### Supported class patterns
//!
//! **Layout:** `flex`, `flex-col`, `flex-row`, `flex-1`, `flex-wrap`, `flex-none`,
//! `min-w-0`, `min-h-0`, `block`, `grid`, `hidden`, `absolute`, `relative`
//!
//! **Alignment:** `items-center`, `items-start`, `items-end`, `justify-center`,
//! `justify-between`, `justify-start`, `justify-end`, `justify-around`,
//! `content-center`, `content-between`, …
//!
//! **Spacing** (numeric value → `px(n.0)`):
//! `gap-4` → `.gap(px(4.0))`, `p-4`, `px-4`, `py-4`, `pt-4`, `pb-4`, `pl-4`, `pr-4`,
//! `m-4`, `mx-4`, `my-4`, `mt-4`, `mb-4`, arbitrary lengths such as `gap-[14px]`
//! and `mx-[1.25rem]`
//!
//! **Sizing:** `w-full`, `h-full`, `size-full`, `w-64`, `h-32`, `w-[280px]`,
//! `w-[18rem]`, `w-[37.5%]`, `w-6/24`
//!
//! **Text:** `text-xs`, `text-sm`, `text-base`, `text-lg`, `text-xl`, `text-2xl`,
//! `text-3xl`, `font-thin` through `font-black`, `italic`, `underline`, `truncate`,
//! `text-left`, `text-center`
//!
//! **Border:** `border` → `.border_1()`, `border-2`, `rounded-sm`, `rounded-md`,
//! `rounded-lg`, `rounded-xl`, `rounded-full`, `rounded-none`
//!
//! **Colors (full Tailwind palette):**
//! `text-red-500` → `.text_color(rgb(0xef4444))`,
//! `bg-blue-600` → `.bg(rgb(0x2563eb))`,
//! `border-green-500` → `.border_color(rgb(0x22c55e))`
//!
//! Supported families: `slate`, `gray`, `zinc`, `neutral`, `stone`, `red`, `orange`,
//! `amber`, `yellow`, `lime`, `green`, `emerald`, `teal`, `cyan`, `sky`, `blue`,
//! `indigo`, `violet`, `purple`, `fuchsia`, `pink`, `rose` (shades 50–950) +
//! `black`, `white`
//!
//! **Arbitrary colors:**
//! `bg-[#ff0000]` → `.bg(rgb(0xff0000))`,
//! `text-[#f00]` → `.text_color(rgb(0xff0000))`,
//! `border-[#11223344]` → `.border_color(rgba(0x11223344))`,
//! `bg-[rgb(15,23,42)]`, `text-[rgba(15,23,42,0.8)]`
//!
//! ### The `class` Attribute — Dynamic (Runtime)
//!
//! When `class` receives an expression, a runtime matcher is generated.
//! Supported classes: common layout/spacing/typography utilities, the full Tailwind
//! color palette, arbitrary colors (`bg-[#ff0000]`, `text-[#f00]`,
//! `bg-[rgba(15,23,42,0.8)]`), arbitrary lengths (`w-[280px]`, `gap-[14px]`),
//! fraction sizing (`w-6/24`), and numeric values for spacing/sizing/opacity via prefix fallback. Truly
//! unsupported classes (e.g. Tailwind variants, unknown utilities) are silently ignored
//! in release and print a warning in debug builds.
//!
//! ```ignore
//! let active = true;
//! rsx! { <div class={if active { "flex gap-4" } else { "block" }} /> }
//! ```
//!
//! Prefer static strings or the `when` attribute instead:
//!
//! ```ignore
//! // static literal — compile-time, documented subset
//! rsx! { <div class="flex gap-4" /> }
//!
//! // ✅ conditional literal — still static
//! let cls = if active { "flex gap-4" } else { "block" };
//! rsx! { <div class={cls} /> }
//!
//! // ✅ when attribute — compile-time, fully flexible
//! rsx! { <div when={(active, |el| el.flex().gap(px(4.0)))} /> }
//!
//! // Dynamic expression — runtime parser, narrower coverage than static literals
//! rsx! { <div class={format!("gap-{} bg-[#f00]", spacing)} /> }
//! ```
//!
//! ### Event Handling
//!
//! Event attributes (camelCase or snake_case) are mapped to GPUI listeners.
//! Elements with event handlers automatically receive a deterministic `.id()`.
//!
//! ```ignore
//! rsx! {
//!     <button onClick={cx.listener(|view, _, _window, cx| {
//!         view.count += 1;
//!         cx.notify();
//!     })}>
//!         {"Increment"}
//!     </button>
//! }
//! // → div().id("src/main.rs::__rsx_button_L42C8").on_click(cx.listener(...)).child("Increment")
//! ```
//!
//! #### `key` — Unique IDs for stateful elements in loops
//!
//! `key={expr}` is a **macro-level** attribute consumed at compile time; it never
//! becomes a `.key()` method call on the GPUI element.
//!
//! **`key` only takes effect when the element already needs an `.id()`** (i.e. it
//! carries `onClick`, `onHover`, `onDrag`, `tooltip`, `focusable`, `overflowScroll`,
//! `trackScroll`, or another attribute that requires a stateful element).
//! On elements without any stateful attributes, `key` is silently ignored and no
//! `.id()` is injected — the element stays a plain `Div`.
//!
//! | Situation | Result |
//! |-----------|--------|
//! | stateful attrs + `key` | composite ID: auto-prefix + key (runtime) |
//! | stateful attrs, no `key` | pure source-location auto ID (compile-time) |
//! | no stateful attrs + `key` | **`key` ignored**, no `.id()` injected |
//! | explicit `id` | always used, `key` has no effect |
//!
//! **Loop safety:** Inside a `{for ...}` loop, every iteration shares the same source
//! location. The macro **emits a compile error** when a stateful element is found inside
//! a loop without an explicit `id` or `key`:
//!
//! ```ignore
//! // ❌ compile error — all <li> would share the same auto ID
//! {for item in &self.items { <li onClick={handler}>{item}</li> }}
//!
//! // ✅ key produces a unique ID per iteration
//! rsx! {
//!     <ul>
//!         {for item in &self.items {
//!             <li key={item.id} onClick={handler}>{item.name.clone()}</li>
//!         }}
//!     </ul>
//! }
//! // → div().id(format!("src/main.rs::__rsx_li_L42C8_{}", item.id)).on_click(handler)…
//! //   e.g. "src/main.rs::__rsx_li_L42C8_1", "src/main.rs::__rsx_li_L42C8_2", …
//! ```
//!
//! `key` accepts any type that implements `Display` (integers, `&str`, UUIDs, …).
//! For a fully stable custom ID that survives refactors, use the `id` attribute instead.
//!
//! | Attribute | GPUI method |
//! |-----------|-------------|
//! | `onClick` / `on_click` | `.on_click(h)` |
//! | `onMouseDown` / `on_mouse_down` | `.on_mouse_down(button, h)` |
//! | `onMouseUp` / `on_mouse_up` | `.on_mouse_up(button, h)` |
//! | `onMouseMove` / `on_mouse_move` | `.on_mouse_move(h)` |
//! | `onMouseDownOut` / `on_mouse_down_out` | `.on_mouse_down_out(h)` |
//! | `onMouseUpOut` / `on_mouse_up_out` | `.on_mouse_up_out(button, h)` |
//! | `onAnyMouseDown` / `on_any_mouse_down` | `.on_any_mouse_down(h)` |
//! | `onAnyMouseUp` / `on_any_mouse_up` | `.on_any_mouse_up(h)` |
//! | `onKeyDown` / `on_key_down` | `.on_key_down(h)` |
//! | `onKeyUp` / `on_key_up` | `.on_key_up(h)` |
//! | `onModifiersChanged` / `on_modifiers_changed` | `.on_modifiers_changed(h)` |
//! | `onHover` / `on_hover` | `.on_hover(h)` |
//! | `onScrollWheel` / `on_scroll_wheel` | `.on_scroll_wheel(h)` |
//! | `onDrag` / `on_drag` | `.on_drag(value, constructor)` |
//! | `onDragMove` / `on_drag_move` | `.on_drag_move(h)` |
//! | `onDrop` / `on_drop` | `.on_drop(h)` |
//! | `onAction` / `on_action` | `.on_action(h)` |
//! | `onBoxedAction` / `on_boxed_action` | `.on_boxed_action(action, h)` |
//! | `captureAnyMouseDown` / `capture_any_mouse_down` | `.capture_any_mouse_down(h)` |
//! | `captureAnyMouseUp` / `capture_any_mouse_up` | `.capture_any_mouse_up(h)` |
//! | `captureKeyDown` / `capture_key_down` | `.capture_key_down(h)` |
//! | `captureKeyUp` / `capture_key_up` | `.capture_key_up(h)` |
//! | `captureAction` / `capture_action` | `.capture_action(h)` |
//!
//! Multi-argument GPUI methods use tuple syntax in RSX, e.g.
//! `onMouseDown={(MouseButton::Left, handler)}`.
//!
//! ### Expressions and Children
//!
//! ```ignore
//! rsx! {
//!     <div>
//!         {format!("Count: {}", self.count)}   // any expression
//!         {self.render_child()}                 // method returning IntoElement
//!         {if self.show {
//!             rsx! { <span>{"Visible"}</span> }
//!         } else {
//!             rsx! { <span>{"Hidden"}</span> }
//!         }}
//!     </div>
//! }
//! ```
//!
//! Two or more consecutive expressions are batched into a single `.children([...])`
//! call (stack-allocated array, zero heap allocation):
//!
//! ```ignore
//! rsx! { <div>{"a"}{"b"}{"c"}</div> }
//! // → div().children(["a", "b", "c"])
//! ```
//!
//! ### For-loop Syntax Sugar
//!
//! ```ignore
//! rsx! {
//!     <ul>
//!         {for item in &self.items {
//!             <li>{item.name.clone()}</li>
//!         }}
//!     </ul>
//! }
//! // → div().children((&self.items).into_iter().map(|item| {
//! //       div().child(item.name.clone())
//! //   }))
//! ```
//!
//! ### Spread Syntax
//!
//! ```ignore
//! rsx! {
//!     <div>
//!         {...self.items.iter().map(|i| rsx! { <span>{i}</span> })}
//!     </div>
//! }
//! ```
//!
//! ### Conditional Attributes: `when` and `whenSome`
//!
//! Apply style transformations based on a runtime condition without leaving compile-time
//! safety:
//!
//! ```ignore
//! rsx! {
//!     <button
//!         class="px-4 py-2 rounded-md"
//!         when={(is_selected, |el| el.bg(rgb(0x3b82f6)).text_color(rgb(0xffffff)))}
//!         when={(is_disabled, |el| el.opacity(0.5))}
//!         whenSome={(custom_color, |el, c| el.bg(rgb(c)))}
//!         whenClass={(is_focused, "border-blue-500 bg-blue-50")}
//!     >
//!         {"Button"}
//!     </button>
//! }
//! ```
//!
//! `whenClass` only accepts string literal classes and rejects stateful classes such as
//! `overflow-scroll`; use `when` for those explicit GPUI method calls.
//!
//! State-style classes can be attached to GPUI's style-refinement hooks:
//!
//! ```ignore
//! rsx! {
//!     <button
//!         class="px-4 py-2 rounded-md bg-blue-500 text-white"
//!         hoverClass="bg-blue-600"
//!         focusClass="border-blue-500"
//!         activeClass="opacity-75"
//!     />
//! }
//! ```
//!
//! These attributes accept only string literal classes. Element-only or stateful classes such as
//! `overflow-scroll` are rejected because GPUI passes a `StyleRefinement` into the closure.
//!
//! ### The `styled` Flag — Semantic Tag Defaults
//!
//! Adding `styled` injects sensible default classes for the tag name, applied before
//! any user-provided attributes:
//!
//! ```ignore
//! rsx! { <h1 styled>{"Title"}</h1> }
//! // → div().text_3xl().font_weight(FontWeight::BOLD).child("Title")
//!
//! rsx! { <button styled onClick={handler}>{"OK"}</button> }
//! // → div().cursor_pointer().id("…").on_click(handler).child("OK")
//! ```
//!
//! | Tag | Default classes |
//! |-----|----------------|
//! | `h1` | `text-3xl font-bold` |
//! | `h2` | `text-2xl font-bold` |
//! | `h3` | `text-xl font-bold` |
//! | `h4` | `text-lg font-bold` |
//! | `h5` | `text-base font-bold` |
//! | `h6` | `text-sm font-bold` |
//! | `button`, `a` | `cursor-pointer` |
//! | `input`, `textarea` | `px-2 py-1` |
//! | `ul`, `ol` | `flex flex-col` |
//! | `li` | `flex items-center` |
//! | `p` | `text-base` |
//! | `label` | `text-sm` |
//! | `form` | `flex flex-col gap-4` |
//!
//! ### Attribute Mapping Reference
//!
//! Most camelCase names are converted to snake_case GPUI methods. Attributes not in this
//! table are passed through unchanged (e.g., `bg={color}` → `.bg(color)`).
//!
//! | RSX attribute | Generated GPUI code |
//! |---------------|---------------------|
//! | `opacity` | `.opacity()` |
//! | `visible` / `invisible` | `.visible()` / `.invisible()` |
//! | `width` / `height` | `.w()` / `.h()` |
//! | `minWidth` / `maxWidth` | `.min_w()` / `.max_w()` |
//! | `minHeight` / `maxHeight` | `.min_h()` / `.max_h()` |
//! | `gapX` / `gapY` | `.gap_x()` / `.gap_y()` |
//! | `flexBasis` | `.flex_basis()` |
//! | `flexGrow` / `flexShrink` (flags) | `.flex_grow_1()` / `.flex_shrink_1()` |
//! | `fontSize` | `.text_size()` |
//! | `lineHeight` | `.line_height()` |
//! | `fontWeight` | `.font_weight()` |
//! | `fontFamily` | `.font_family()` |
//! | `textAlign` | `.text_align()` |
//! | `textColor` | `.text_color()` |
//! | `backgroundColor` | `.bg()` |
//! | `borderColor` | `.border_color()` |
//! | `borderTop` / `borderBottom` | `.border_t(value)` / `.border_b(value)` |
//! | `borderLeft` / `borderRight` | `.border_l(value)` / `.border_r(value)` |
//! | `border_t` / `border_b` / `border_l` / `border_r` (flags) | `.border_t_1()` / `.border_b_1()` / `.border_l_1()` / `.border_r_1()` |
//! | `roundedTop` / `roundedBottom` | `.rounded_t()` / `.rounded_b()` |
//! | `roundedTopLeft` / `roundedTopRight` | `.rounded_tl()` / `.rounded_tr()` |
//! | `roundedBottomLeft` / `roundedBottomRight` | `.rounded_bl()` / `.rounded_br()` |
//! | `boxShadow` | `.shadow()` |
//! | `inset` | `.inset()` |
//!
//! ## Auto ID Injection
//!
//! Elements that require a stateful identity (`onClick`, `onHover`, `onDrag`, `tooltip`,
//! `focusable`, `overflowScroll`, `trackScroll`, and static `overflow-scroll` classes)
//! automatically receive a deterministic `.id()` derived from the element's source
//! location. The ID is chosen by the following priority:
//!
//! 1. **Explicit `id`** — always wins, `key` is ignored.
//! 2. **`key` present** (and element is stateful) — composite ID. Literal keys use
//!    `concat!(...)`; dynamic keys use `format!(concat!(file!(), "::{prefix}_{}"), key_expr)`.
//! 3. **No `key`** (and element is stateful) — pure compile-time source-location ID:
//!    `concat!(file!(), "::", "__rsx_{tag}_L{line}C{col}")`
//! 4. **Not stateful** — no `.id()` injected; `key` is silently ignored.
//!
//! ```text
//! // Format (no key): concat!(file!(), "::", "__rsx_{tag}_L{line}C{col}")
//! // Example:         "src/views/counter.rs::__rsx_button_L42C8"
//! //
//! // Format (literal key): concat!(file!(), "::__rsx_{tag}_L{line}C{col}_", "42")
//! // Format (dynamic key): format!(concat!(file!(), "::__rsx_{tag}_L{line}C{col}_{}"), key)
//! // Example:              "src/views/list.rs::__rsx_li_L55C12_42"
//! ```
//!
//! The source-location ID is stable across incremental rebuilds as long as the
//! element's position in the file does not change. For IDs that must survive
//! refactors, use the explicit `id` attribute.
//!
//! > **Note on style attributes:** `hover`, `focus`, `group`, and `groupHover` are
//! > *`Styled` trait* methods and do **not** trigger auto ID injection. `active`,
//! > `activeClass`, and `groupActive` map to stateful GPUI hooks in the current target
//! > and do require an ID.
//!
//! ### Loop safety
//!
//! Elements inside `{for ...}` loops share the same source location and would receive
//! identical auto IDs across iterations. **A compile error is emitted** in this case.
//! Add `key={expr}` (any `Display` type) to produce a unique ID per iteration:
//!
//! ```ignore
//! // ❌ compile error — all <li> would share the same auto ID
//! {for item in &self.items { <li onClick={handler}>{item}</li> }}
//!
//! // ✅ key produces a unique ID per iteration
//! {for item in &self.items { <li key={item.id} onClick={handler}>{item}</li> }}
//! ```
//!
//! ## Performance
//!
//! - **Compile-time** — All class parsing, color lookup, and attribute mapping happen
//!   during macro expansion. The generated code is identical to hand-written GPUI.
//! - **O(1) lookups** — Colors, attributes, and spacing prefixes use `match` statements
//!   that the compiler turns into jump tables.
//! - **Zero allocation** — Static classes generate no heap allocation. Dynamic classes
//!   use `AsRef<str>` (zero-copy for `&str` inputs).
//! - **Binary size** — Dynamic class helpers use `#[inline(never)]` + LLVM ICF to avoid
//!   code bloat when multiple `class={expr}` appear in the same component.
//!
//! ## Further Reading
//!
//! - [Architecture guide (ARCHITECTURE.md)](https://github.com/wsafight/gpui-rsx/blob/main/ARCHITECTURE.md) —
//!   module design, data flow, extension points
//! - [Getting started](https://wsafight.github.io/gpui-rsx/getting-started/)
//! - [API reference](https://wsafight.github.io/gpui-rsx/reference/api/)
//! - [Best practices](https://wsafight.github.io/gpui-rsx/guides/best-practices/)
//! - [Changelog](https://github.com/wsafight/gpui-rsx/blob/main/CHANGELOG.md)

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod codegen;
mod diagnostics;
mod parser;

use codegen::class::ClassMode;
use codegen::element::{generate_body_expansion_preview, generate_body_with_mode};
use parser::RsxBody;

/// Transforms JSX-like markup into GPUI method chains at compile time.
///
/// # Syntax
///
/// ```text
/// rsx! { <tag attr1 attr2={expr} class="…"> {children} </tag> }
/// rsx! { <tag /> }          — self-closing
/// rsx! { <> … </> }         — fragment (returns vec![…])
/// ```
///
/// # Examples
///
/// ## Basic element with static classes
///
/// ```ignore
/// rsx! {
///     <div class="flex flex-col gap-4 p-4">
///         <h1 class="text-2xl font-bold">{"Hello"}</h1>
///     </div>
/// }
/// ```
///
/// ## Event handling (auto ID injected)
///
/// ```ignore
/// rsx! {
///     <button
///         class="bg-blue-500 text-white px-4 py-2 rounded-md"
///         onClick={cx.listener(|view, _, _window, cx| {
///             view.count += 1;
///             cx.notify();
///         })}
///     >
///         {"Click me"}
///     </button>
/// }
/// ```
///
/// ## For-loop rendering
///
/// ```ignore
/// rsx! {
///     <ul>
///         {for item in &self.items {
///             <li>{item.name.clone()}</li>
///         }}
///     </ul>
/// }
/// ```
///
/// ## Fragment (multiple root elements)
///
/// ```ignore
/// rsx! {
///     <>
///         <div>{"First"}</div>
///         <div>{"Second"}</div>
///     </>
/// }
/// ```
///
/// ## Conditional styling
///
/// ```ignore
/// rsx! {
///     <div
///         class="px-4 py-2 rounded-md"
///         when={(is_active, |el| el.bg(rgb(0x3b82f6)))}
///         whenSome={(custom_width, |el, w| el.w(px(w)))}
///     >
///         {"Content"}
///     </div>
/// }
/// ```
///
/// ## Styled defaults
///
/// ```ignore
/// rsx! { <h1 styled>{"Title"}</h1> }
/// // → div().text_3xl().font_weight(FontWeight::BOLD).child("Title")
/// ```
///
/// # Notes
///
/// - **Static `class`**: parsed entirely at compile time, supports the documented Tailwind-like subset.
/// - **Dynamic `class={expr}`**: runtime matcher; full Tailwind color palette,
///   arbitrary colors, common layout/spacing/typography utilities, arbitrary
///   lengths, fraction sizing, and numeric values via prefix fallback. Truly unsupported classes are silently ignored
///   in release builds. Prefer `when` or static strings for full coverage.
/// - **Auto ID**: elements with stateful event handlers receive a source-location-based
///   ID automatically. Provide an explicit `id` attribute for state-sensitive elements.
#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    let body = parse_macro_input!(input as RsxBody);
    let code = generate_body_with_mode(&body, ClassMode::Permissive);
    TokenStream::from(code)
}

/// Transforms RSX into GPUI method chains and rejects unsupported static classes.
///
/// Static unknown classes emit a compile error instead of being ignored or falling through
/// to a GPUI method name. Dynamic unknown classes panic when evaluated.
#[proc_macro]
pub fn rsx_strict(input: TokenStream) -> TokenStream {
    let body = parse_macro_input!(input as RsxBody);
    let code = generate_body_with_mode(&body, ClassMode::Strict);
    TokenStream::from(code)
}

/// Transforms RSX using the default permissive class handling.
///
/// This is equivalent to [`rsx!`], and exists as an explicit opt-in when codebases
/// also use [`rsx_strict!`].
#[proc_macro]
pub fn rsx_permissive(input: TokenStream) -> TokenStream {
    let body = parse_macro_input!(input as RsxBody);
    let code = generate_body_with_mode(&body, ClassMode::Permissive);
    TokenStream::from(code)
}

/// Returns a string preview of the generated GPUI method chain.
///
/// This macro is intended for debugging generated code. It does not type-check the
/// generated GPUI expression because it returns a `&'static str`.
#[proc_macro]
pub fn rsx_expand(input: TokenStream) -> TokenStream {
    let body = parse_macro_input!(input as RsxBody);
    let preview = generate_body_expansion_preview(&body, ClassMode::Permissive);
    let code = quote::quote! { #preview };
    TokenStream::from(code)
}
