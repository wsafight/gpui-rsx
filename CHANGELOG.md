# Changelog

English | [简体中文](./CHANGELOG_CN.md)

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2026-02-21

### 🐛 Fixed

#### Auto ID — `is_stateful_attr` misclassification
- **`hover`, `active`, `focus`, `group` removed from stateful attribute detection** —
  These are `Styled` trait methods (accept `StyleRefinement`), not `StatefulInteractiveElement`
  methods, and do not require `.id()`. Previously they caused unnecessary `.id()` injection,
  silently changing element types from `Div` to `Stateful<Div>`.
  Only `on_*` / `capture_*` event handlers, `tooltip`, and `track_focus` now trigger injection.

#### Auto ID — loop ID collisions
- **Compile error on stateful elements in `for` loops without `id` or `key`** —
  All iterations of a for-loop share the same source location, so auto-generated IDs would be
  identical across iterations, causing GPUI state conflicts. The macro now emits a clear
  compile-time error pointing to the offending element with actionable fix suggestions.

### ✨ Added

#### `key` attribute — composite auto ID for loops
- **`key={expr}` attribute** — A new macro-level attribute (never emitted as a `.key()` method
  call) that participates in auto ID generation for stateful elements inside for-loops:
  ```rust
  // ❌ compile error — all <li> would share the same auto ID
  {for item in &self.items { <li onClick={handler}>{item}</li> }}

  // ✅ key makes every ID unique per iteration
  {for item in &self.items { <li key={item.id} onClick={handler}>{item}</li> }}
  // → div().id(format!("src/list.rs::__rsx_li_L42C8_{}", item.id)).on_click(handler)…
  ```
  - ID format: `format!(concat!(file!(), "::{prefix}_{}"), key_expr)` — prefix is compile-time,
    key is runtime, no extra `String` allocation beyond the prefix.
  - `key` accepts any type implementing `Display` (integers, `&str`, UUIDs, …).
  - **`key` is only effective when the element has stateful attributes (`needs_id = true`).
    On non-stateful elements, `key` is silently ignored and no `.id()` is injected.**
  - Priority order: explicit `id` > stateful + `key` > stateful without `key` > not stateful.

### ♻️ Refactoring

- **`next_auto_id` renamed and split** — `next_auto_id` is replaced by:
  - `make_auto_id(tag_ident)` — source-location only, compile-time `concat!`
  - `make_keyed_auto_id(tag_ident, key_expr)` — source-location prefix + runtime key `format!`

### ✅ Testing
- All 288 tests pass (227 macro + 35 coverage + 24 unit + 2 diagnostic)
- Updated test assertions: `hover`/`active`/`focus`/`group` no longer assert stateful detection;
  new tests for `tooltip`/`track_focus` as true stateful attributes

---

## [0.3.0] - 2026-02-21

### ♻️ Refactoring

#### Test Infrastructure
- **Eliminated duplicate methods in `tests/common/mod.rs`** — `impl MockElement` previously
  defined ~60 methods that were already provided by `impl Styled for MockElement`. The direct
  `impl` block now contains only the methods that are *not* part of the `Styled` trait
  (event handlers, state-style methods, conditional helpers, etc.). File reduced from 823 to
  456 lines; a single source of truth per method removes any risk of silent divergence.

#### Code Generator
- **Simplified black/white entry generation in `runtime.rs`** — The loop that emits
  black/white color match arms previously used `class_str.starts_with("text-")` /
  `starts_with("bg-")` at runtime to select the method identifier. The method name is now
  encoded directly in the data array, eliminating the runtime branch entirely:
  ```rust
  // Before: method ident derived at runtime
  for (class_str, hex) in [("text-black", 0x000000u32), …] {
      let (method_ident, hex) = if class_str.starts_with("text-") { … };
  }
  // After: method ident encoded in data
  for (class_str, method_ident, hex) in [
      ("text-black", &text_color_ident, 0x000000u32), …
  ] { … }
  ```

- **Extracted `is_directional_border(rest)` in `class.rs`** — The logic distinguishing
  directional border classes (`border-t`, `border-t-2`) from color border classes
  (`border-red-500`) was inlined inside `parse_single_class`. It is now a dedicated
  `fn is_directional_border(rest: &str) -> bool` with its own doc comment explaining the
  edge cases, reducing the call-site from 11 lines of comment + code to a single readable
  predicate call.

### ✅ Testing
- All 287 tests pass (227 macro + 35 coverage + 23 unit + 2 diagnostic)
- Zero regressions

---

## [0.2.2] - 2026-02-18

### 🚀 Performance Optimizations

#### Compile-time Performance
- **`split_ascii_whitespace` in `parse_class_string`** - Class names are ASCII-only; replaced
  `split_whitespace` with `split_ascii_whitespace` to skip Unicode whitespace table lookups at
  every token boundary.
- **Unified `text_` prefix handling** - `parse_color_class` has been removed. Color lookup
  (`text-red-500`) and text-size lookup (`text-xl`) are now handled under a single
  `strip_prefix("text_")` call in `parse_single_class`, eliminating a redundant prefix strip for
  every `text-*` class.
- **Early fast-path for empty elements** - The "no attributes, no children" check in
  `generate_element` is now the very first operation, before any variable initialisation or loop
  entry. Bare self-closing tags like `<Icon />` return immediately without scanning attributes.
- **Larger `Vec::with_capacity` estimate** - The method buffer in `generate_element` now
  pre-allocates `attributes.len() * 2 + children.len()` instead of `attributes.len() +
  children.len()`. A single `class` attribute typically expands to 3-4 method calls, so the
  ×2 multiplier halves expected reallocation events on class-heavy elements.

#### Runtime Performance (Generated Code)
- **`.children([...])` aggregation threshold lowered 3 → 2** - Two or more consecutive `Expr`
  children are now batched into a single `.children([...])` call backed by a stack-allocated
  array. Arrays carry no heap-allocation cost, so the threshold of 3 was unnecessarily
  conservative; 2 reduces method dispatch count while keeping the same stack footprint.

#### Binary Size
- **`panic = "abort"` in `[profile.release]`** - proc-macro binaries never need stack
  unwinding. Enabling `panic = "abort"` removes unwind tables from the compiled binary,
  reducing its size and load time during host-side compilation.

### ✅ Testing
- All 236 tests pass (203 macro + 31 coverage + 2 diagnostic)
- Zero regressions

---

## [0.2.1] - 2026-02-18

### 🚀 Performance Optimizations

#### Compile-time Performance
- **Match-based O(1) color lookup** - Replaced `COLOR_MAP` const array + `.iter().find()` linear
  scan with a `match` statement in `lookup_color()`. The compiler generates an efficient jump
  table/trie, reducing worst-case comparisons from O(242) to O(1).
- **Match-based attribute / spacing / text-size lookups** - Same pattern applied to
  `lookup_attr_method()`, `lookup_spacing_method()`, and `is_valid_text_size()`, replacing four
  separate linear-scan arrays.
- **Single-pass attribute scanning in `generate_element`** - `user_id`, `has_styled`, and
  `needs_id` are now extracted in one loop instead of separate passes.
- **Cached `Ident::to_string()` results** - Tag and attribute name strings are computed once and
  reused within each code-generation call, avoiding repeated heap allocations.

#### Runtime Performance
- **Zero-copy dynamic class strings** - Changed from `String::into()` to `AsRef<str>`.
  `&str` inputs pass through without any allocation; `String` and `Cow<str>` are also supported.
- **`Vec` reuse for consecutive child expressions** - `consecutive_exprs` is allocated once
  outside the loop and cleared with `.clear()` each iteration instead of being reallocated.
- **`split_ascii_whitespace` for dynamic class parsing** - Generated class iteration now uses
  `split_ascii_whitespace` instead of `split_whitespace`. Class names are ASCII-only, so this
  avoids the Unicode whitespace scanning overhead of the standard iterator.
- **Empty-string fast path for dynamic class** - An `is_empty()` guard before the fold skips
  iterator creation entirely for empty strings — the common case in patterns like
  `class={if cond { "flex" } else { "" }}`.

#### Memory / Allocation Improvements
- **`parse_class_string` returns an iterator** - Avoids an intermediate `Vec<TokenStream>`;
  callers consume via `.extend()` directly into the output buffer.
- **`generate_attr_methods` pushes directly into caller's `Vec`** - Eliminates one extra
  `Vec` allocation per attribute list.
- **`Vec::with_capacity` pre-allocation throughout** - Attribute lists, child lists, and method
  chains pre-allocate with realistic capacity hints.
- **`Cow<str>` for class string transformations** - Zero-copy borrow when the class name
  contains no hyphens; only allocates when a `-` → `_` replacement is needed.

#### Binary Size Optimization
- **Dynamic class match table extracted to `#[inline(never)]` local function** - Multiple
  `class={expr}` in the same component now share one function body. LLVM ICF can merge identical
  single-monomorphisation instances across components, reducing code bloat from repeated match
  table inlining.
- **Thread-local cache for common class match arms** - `generate_common_class_matches()` is
  called only once per compiler process (via `thread_local! + RefCell<Option<Rc<…>>>`). Shared
  ownership via `Rc` keeps subsequent calls to `get_cached_common_class_matches()` at O(1).

### ♻️ Code Quality Improvements
- **Merged `EVENT_HANDLERS` and `ATTRIBUTE_NAME_MAP`** into a single `lookup_attr_method()`
  match function, removing the redundant third tuple field (method name always equalled
  snake_case).
- **Unified color parsing** via `parse_color_with_method(color, method)` - Removed three
  near-identical implementations (text_color / bg / border_color) into one function.
- **`is_stateful_attr()` uses `starts_with` + `match`** instead of a two-step linear scan
  through `NEEDS_ID_ATTRS` and `EVENT_HANDLERS` arrays.
- **Removed dead code** - Deleted deprecated `class_dynamic_value_error()` diagnostic function.
- **Fixed `generate_for_loop` type constraint** (`element.rs`) - Multi-child for-loop bodies
  previously used a fixed-size array `[...]` in `flat_map`, which requires all elements to share
  the same type. Changed to `vec![...]` to correctly support mixed element types (e.g., `div()`
  alongside custom components) in the same for-loop body.
- **Zero-allocation 3-char hex expansion** (`class.rs`) - `parse_arbitrary_hex` now expands
  3-digit hex values (`[#rgb]` → `[#rrggbb]`) using bitwise arithmetic instead of allocating a
  `String`. Each nibble is duplicated via `d << 4 | d` without any heap allocation.
- **Refactored `border_` conditional logic** (`class.rs`) - Replaced the empty `if { }` +
  `else if` pattern with an explicit `is_directional` binding and a single `if !is_directional`
  guard, making the intent immediately clear without dead branches.
- **Idiomatic tuple extraction in `parse_condition_tuple`** (`parser.rs`) - Replaced iterator
  creation + two `.next().expect()` calls with `Punctuated::pop()`, which directly removes
  elements by index without constructing an intermediate iterator.

### 📖 Documentation
- **Dynamic class limitation now clearly documented** (`runtime.rs`) - The `generate_dynamic_class_code`
  doc comment and the `_ => el` wildcard arm now both explicitly state that only the ~58 pre-compiled
  common classes are recognised at runtime; unknown classes are silently ignored. Recommended
  alternatives (literal strings, conditional expressions) are listed in priority order.
- **`auto_id` counter incremental-compile caveat** (`element.rs`) - `AUTO_ID_COUNTER` and
  `next_auto_id` now document the known limitation: because the thread-local counter increments
  monotonically across all macro expansions in a single compile process, incremental rebuilds that
  change which files are recompiled may produce different IDs for the same element. Elements that
  rely on ID stability for focus/state tracking should use an explicit `id` attribute.

### ✅ Testing
- All 236 tests pass (203 macro + 31 coverage + 2 diagnostic)
- Zero regressions from optimisation changes

### 🔒 Security
- **Updated dependency** - Replaced unmaintained `proc-macro-error` with `proc-macro-error2`
  - Addresses RUSTSEC-2024-0370 advisory
  - Eliminates duplicate `syn 1.x` dependency tree
  - Fully compatible API (drop-in replacement)

---

## [0.2.0] - 2026-02-17

### ✨ Added

#### Core Features
- **For-loop syntax sugar** - Simplify list rendering with `{for item in items { ... }}` syntax
- **Styled flag** - Apply sensible default styles based on tag names (h1-h6, button, input, etc.)
- **Conditional styling** - `when` and `whenSome` attributes for dynamic styling
- **Fragment support** - Return multiple root elements with `<>...</>` syntax
- **Full Tailwind color palette** - 242 built-in colors (slate, gray, red, blue, etc.) + arbitrary hex values
- **Comprehensive event handling** - Support for onClick, onMouseDown, onKeyDown, onHover, etc. (15 event types)
- **Attribute mapping** - camelCase to snake_case conversion (zIndex → z_index, fontSize → font_size, etc.)

#### Documentation
- Complete documentation system: Getting Started, API Reference, Best Practices, Migration Guide, Troubleshooting
- Architecture documentation (`ARCHITECTURE.md` / `ARCHITECTURE_CN.md`)
- English as primary README; Chinese moved to `README_CN.md`

#### Infrastructure
- GitHub Actions CI, GPUI compatibility testing, and release automation workflows
- Codecov integration and local coverage script

#### Developer Experience
- Better error messages via `proc-macro-error` dependency
- Updated repository URL to `https://github.com/wsafight/gpui-rsx`
- Comprehensive test suite

### 🐛 Fixed
- **Auto ID injection for stateful events** - Added missing `onHover`/`on_hover`, `onDrag`/`on_drag`,
  `onDrop`/`on_drop` to stateful attribute detection, fixing compile failures when these event
  handlers were used without an explicit `id` attribute. Added 6 test cases to verify.

### ♻️ Refactored
- **Deduplicated child-node parsing** - Extracted `try_parse_child_node()`, eliminating duplicated
  logic between `parse_children()` and `parse_for_loop()`.
- **Deduplicated for-loop code generation** - Extracted `generate_for_loop()`, unifying
  `map`/`flat_map` generation logic.
- **Simplified color class parsing** - Replaced redundant `starts_with()` + `strip_prefix()`
  with a single `strip_prefix()` call.

### 🗑️ Removed
- `examples/` directory (required external GPUI dependency; functionality covered by tests)
- `trybuild` compile-fail tests (simplified test structure)

---

## [0.1.2] - 2026-02-16

### 🔧 Chore
- Updated repository URL
- Commented out todo_app example pending dependency resolution

## [0.1.1] - 2026-02-15

### 📖 Documentation
- Switched to English as primary README
- Added Chinese README (README_CN.md)
- Updated installation instructions

## [0.1.0] - 2026-02-15

### 🎉 Initial Release
- Basic RSX macro implementation
- Support for nested elements
- Attribute support (boolean and value attributes)
- Basic class attribute parsing
- Expression interpolation
- Event handling foundation

---

[0.3.1]: https://github.com/wsafight/gpui-rsx/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/wsafight/gpui-rsx/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/wsafight/gpui-rsx/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/wsafight/gpui-rsx/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/wsafight/gpui-rsx/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/wsafight/gpui-rsx/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wsafight/gpui-rsx/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wsafight/gpui-rsx/releases/tag/v0.1.0
