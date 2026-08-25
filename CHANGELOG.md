# Changelog

English | [简体中文](./CHANGELOG_CN.md)

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added aliases and real GPUI contracts for the latest accessibility, scroll-axis, external-drag,
  mouse pressure, pinch, grid min/max-content, and text ellipsis APIs.
- Added a pinned 42-method `StatefulInteractiveElement` snapshot plus a live source drift check for
  the latest compatibility workflow.

### Changed

- Documented crates.io package content verification in the release checklist.
- Added troubleshooting guidance for state class attributes and the explicitly typed
  `group_drag_over` fallback.
- Clarified performance documentation for stateful auto-ID triggers introduced in 0.6.0.
- Linked compatibility and release-checklist pages from the main documentation entry points.
- Upgraded the demo lockfile to Zed GPUI `e9735934` and gpui-component / gpui-base `7885c416`.
- Declared Rust 1.85 as the root proc-macro MSRV and retained Rust 1.95 for demo compatibility.
- Made compatibility issue reporting aggregate both platform results before changing issue state.

### Fixed

- Stopped treating `scrollbarWidth` as stateful, while preserving auto IDs for overflow scrolling.
- Restored `text-ellipsis-start` and added `text-ellipsis-middle` across static, strict, and dynamic
  class paths.
- Made class benchmarks consume observable non-zero builder results.

## [0.6.0] - 2026-06-12

### Added

- Added a pinned `demo/` crate that checks GPUI from the Zed git repository together with
  `gpui-component`, covering hello, counter, palette, task list, API-surface, and component
  integration examples.
- Added CI coverage for the demo crate with
  `cargo check --manifest-path demo/Cargo.toml --bins --locked`.
- Added GPUI compatibility workflow coverage against the demo manifest so future Zed GPUI changes
  are checked against the real GPUI API surface.
- Added static expansion for `class={match ...}` expressions whose arms all return class string
  literals.
- Added stricter static opacity validation and UI coverage for invalid `opacity-*` classes.
- Added `hoverClass`, `focusClass`, and `activeClass` attributes for GPUI state-style class
  expansion through `StyleRefinement` closures.
- Added more real GPUI interactive and accessibility mappings, including `focusVisible`,
  `tooltipShowDelay`, `onAuxClick`, `onA11yAction`, `role`, and `aria*` attributes.

### Changed

- Dynamic unknown class warnings in permissive mode now print at most once per generated call site
  and class value in debug builds, avoiding repeated stderr writes in render loops.
- Literal `key` values on stateful elements now use a compile-time `concat!` auto-ID path instead of
  runtime `format!`; dynamic `key={expr}` values keep the existing `Display`-based fallback.
- `visible={expr}` now evaluates the expression once while mapping to `.visible()` or `.invisible()`.
- Dynamic class documentation now calls out `if` and `match` literal expressions as preferred
  compile-time-expanded alternatives.
- Consolidated attribute method metadata so method mapping, auto-ID decisions, and tuple argument
  expansion share one lookup path.
- The Astro docs workflow now runs on documentation pull requests, performs `pnpm run check`, and
  only deploys Pages artifacts from `main` pushes with scoped Pages permissions.
- Documented the `groupHover` / `groupActive` ID boundary and the explicit
  `group_drag_over::<YourType>(...)` fallback required by GPUI.
- Reduced duplicate test mock implementations by generating `MockElement`'s `Styled` surface with
  the same helper macros used by `StyleRefinement`.
- Split shared test mock state and types into focused `tests/common` modules.
- Added `scripts/check.sh` to run the standard root and real-GPUI demo validation commands, with
  an extended `--release` mode for benchmark, docs, GPUI tree, and publish dry-run checks.
- Reduced the crates.io package contents to the files required to build and document the crate.

### Fixed

- Fixed `benches/class_performance.rs` so the benchmark target compiles cleanly again.
- Fixed for-loop parsing so iterator block expressions such as `{for item in { items.iter() } { ... }}`
  are accepted.
- Fixed dynamic invalid opacity handling so invalid runtime `opacity-*` values are ignored instead of
  producing invalid GPUI opacity calls.
- Fixed demo dependency guidance to avoid duplicate GPUI crate instances when `gpui-component` is
  present.
- Fixed GPUI state-style compatibility so `hover`, `focus`, and `active` closures receive
  `StyleRefinement` consistently with current GPUI.
- Improved state class diagnostics with concrete guidance for element-level classes such as
  `overflow-scroll` and `debug-outline`.
- Rejected `groupDragOver` attributes with an actionable diagnostic because GPUI requires an
  explicit drag data type.

## [0.5.1] - 2026-06-09

### Added

- Added GPUI 0.2.2 element support for `<img src={...}>`, `<canvas prepaint={...} paint={...}>`,
  and `<svg src={...}>`.
- Added GPUI 0.2.2 image and prepaint-related attribute mappings such as `objectFit`,
  `withFallback`, `withLoading`, `imageCache`, and `onChildrenPrepainted`.
- Added static and strict-mode class support for GPUI 0.2.2 rounded radius variants.

### Changed

- Optimized conditional literal classes such as `class={if active { "flex" } else { "block" }}`
  so they expand through the static class path instead of the dynamic class matcher.
- Reduced macro expansion work by skipping attribute analysis for element-level attributes already
  consumed by the code generator.
- Reduced static class parser allocations by parsing common numeric length, color, opacity,
  line-clamp, and directional border classes before falling back to normalized method names.
- Updated examples and docs to import `gpui::prelude::*` where GPUI 0.2.2 requires it.

### Fixed

- Fixed GPUI 0.2.2 compatibility for `aspect-square` by mutating `style().aspect_ratio` instead of
  calling a removed GPUI helper method.
- Fixed permissive/strict handling for unsupported `text-ellipsis-start` after its GPUI helper was
  removed.
- Fixed benchmark mocks to match the current GPUI 0.2.2-compatible API surface.

## [0.5.0] - 2026-05-14

### Added

- Added path-qualified component tags such as `<ui::TaskCard />`, including matching closing
  tag validation and auto-ID generation for stateful path components.
- Added macro-only `base={expr}` support so component method chains can start from custom
  constructors or builders instead of the default tag constructor.
- Added `whenClass={(condition, "class string")}` for conditional static class application.
- Added camelCase aliases for `fontFamily`, `textColor`, `backgroundColor`, and `borderColor`.

### Changed

- Improved `whenClass`, path tag mismatch, for-loop key, and unsupported `whiteSpace`
  diagnostics with more precise spans and actionable hints.
- Documented dynamic class capability boundaries, builder-backed components, path tags, and
  mixed fragment element guidance across the README and API docs.

### Removed

- Removed the stale standalone `gpui-rsx-optimization.md` document.

## [0.4.4] - 2026-05-14

### Changed

- Consolidated common class support metadata so strict class validation and dynamic class fast
  paths share one source of truth while preserving dynamic-stateful class behavior.
- Refactored dynamic numeric class fallback generation to use shared length-prefix metadata.
- Removed crate-level release profile overrides so applications and workspaces keep control over
  LTO, codegen units, and panic strategy.

## [0.4.3] - 2026-05-14

### Added

- Added `rsx_strict!`, `rsx_permissive!`, and `rsx_expand!` macros for stricter class
  validation, explicit permissive mode, and generated-code previews.
- Added arbitrary length support for spacing and sizing classes, including `px`, `rem`,
  percentages for sizing, and fraction sizing such as `w-6/24`.
- Added arbitrary RGB/RGBA color support and GPUI 0.2 font weight mappings.

### Changed

- Improved dynamic class handling for arbitrary colors, lengths, fraction sizing,
  `debug-outline`, and GPUI 0.2 compatibility helpers.
- Preserved permissive handling for unknown static classes while making `rsx_strict!`
  report unsupported classes clearly.

### Fixed

- Fixed dynamic `font-extralight` support so it maps to `FontWeight::EXTRA_LIGHT`.
- Rejected non-finite numeric class values such as `NaN` and `inf` before they can be
  passed to GPUI length helpers.
- Fixed dynamic directional border class handling and several invalid arbitrary-value
  diagnostics.

## [0.4.2] - 2026-05-12

### Changed

- Reused attribute scan results during code generation to avoid repeated attribute-name and
  static class string allocations.
- Folded for-loop key validation into recursive code generation, removing the separate
  pre-generation traversal while preserving loop-safe auto-ID diagnostics.
- Kept styled default classes ordered before user-provided attributes while generating
  user attribute methods in a single pass.

## [0.4.0] - 2026-05-11

### Changed

- Updated GPUI compatibility documentation and examples for GPUI 0.2.
- Updated event and interaction mappings to match GPUI 0.2 method signatures, including
  multi-argument mouse and drag APIs.

### Fixed

- Fixed directional border flag generation for GPUI 0.2 preset width methods
  (`border_t` → `.border_t_1()` and related directions).
- Preserved directional border value attributes such as `border_t={px(1.0)}` as
  `.border_t(value)` instead of incorrectly mapping them to preset width methods.
- Avoided generating `.children([...])` for consecutive expression children so mixed child
  types compile correctly.
- Updated auto-ID detection for GPUI 0.2 stateful interaction methods and static overflow
  scroll classes.

## [0.3.2] - 2026-02-22

### 🐛 Fixed

#### `parse_single_class` — panic on Tailwind variant syntax
- **Defensive guard against non-identifier class names** (`class.rs`) — Classes containing
  characters that are not alphanumeric or `_` (e.g. `hover:bg-blue-500`, `focus:text-red-500`)
  previously caused `syn::Ident::new` to panic at compile time. The default branch now validates
  that `method_name` consists solely of ASCII alphanumeric characters and underscores before
  constructing an `Ident`; invalid names silently produce an empty `TokenStream` so valid
  sibling classes in the same `class="…"` string are still applied.

#### `Styled` trait — missing no-arg directional border methods
- **`border_t`, `border_b`, `border_l`, `border_r` added to `Styled` trait** (`tests/common/mod.rs`)
  — These four methods were defined only as generic `<T>` inherent methods on `MockElement` to
  support the `borderTop={val}` attribute form. However, `class="border-t"` (and the other three
  directions) correctly fell through `is_directional_border` to the default branch and generated
  `.border_t()` with **no arguments**, which would have failed to compile for any test using the
  class path. The four methods are now no-arg signatures in the `Styled` trait (matching real GPUI's
  API) and the generic inherent versions have been removed. The attribute tests were updated to use
  the flag form (`<div border_t />`), and four new tests (`test_class_border_t/b/l/r`) cover the
  previously untested code path.
- **`border-t`, `border-b`, `border-l`, `border-r` added to dynamic class match table** (`runtime.rs`)
  — These classes were absent from `static_classes`, so `class={expr}` containing e.g. `"border-t"`
  would silently emit a debug warning and do nothing at runtime. They are now in the pre-compiled
  match table alongside `border` and `border-2`.

#### `generate_numeric_fallback_code` — redundant `quote!` evaluation per call
- **Thread-local caching for numeric fallback** (`runtime.rs`) — `generate_numeric_fallback_code`
  re-executed `quote!` (~40 `if-let` statements) on every call to `generate_dynamic_class_code`,
  while `generate_common_class_matches` was already cached as a `thread_local` `String`. A new
  `NUMERIC_FALLBACK_STR` thread-local and `get_cached_numeric_fallback()` function apply the same
  caching pattern: the `TokenStream` is stringified once and re-parsed per proc-macro bridge
  invocation, eliminating repeated `quote!` allocation for files with multiple `class={expr}`
  attributes.

### ✨ Enhanced

#### Dynamic class match table — 8 additional classes
- **`rounded-none`, `rounded-xl`** — Added to the static match table in `runtime.rs`; these
  were previously only available in the static-string code path, causing silent no-ops when
  used in `class={expr}` expressions.
- **`cursor-default`, `cursor-text`** — Same as above.
- **`overflow-visible`** — Same as above.
- **`shadow-sm`, `shadow-md`, `shadow-lg`** — Same as above.
- The eight methods were simultaneously promoted from `impl MockElement` to the `Styled` trait
  in `tests/common/mod.rs`, ensuring they are reachable from the generic `E: Styled` bound used
  in the generated `__rsx_apply_class` helper.

### 📖 Documentation

- **Styled defaults table** (`lib.rs`, `README.md`) — Added missing entries: `li` → `flex items-center`,
  `p` → `text-base`, `label` → `text-sm`, `form` → `flex flex-col gap-4`. These defaults were
  already implemented in `tables::lookup_tag_default` but not documented.
- **Attribute mapping table** (`lib.rs`, `README.md`) — Added missing `roundedTop` → `.rounded_t()`
  and `roundedBottom` → `.rounded_b()` entries.
- **Dynamic class description** (`lib.rs`, `README.md`) — Replaced the inaccurate "~58 pre-compiled
  common classes" language with an accurate description: full Tailwind color palette (22 families ×
  11 shades × 3 prefixes = 726+ entries), common layout/spacing/typography utilities, and arbitrary
  numeric values for spacing/sizing/opacity/z-index via prefix fallback.
- **`overflowX` / `overflowY` method names** (`README.md`) — Fixed incorrect `.overflow_x_hidden()`
  / `.overflow_y_hidden()` to the actual GPUI methods `.overflow_x()` / `.overflow_y()`.
- **Text size list** (`README.md`) — Removed unsupported `text-4xl` and `text-5xl` from the
  supported class patterns (only `xs` through `3xl` are in `is_valid_text_size`).
- **Dynamic class diagnostic** (`tests/diagnostic_tests.rs`) — Renamed
  `test_class_dynamic_value` → `test_class_dynamic_value_is_supported` with a corrected
  commentary noting that `class={expr}` is valid RSX (not a compile error) and generates a
  runtime match.

### ✅ Testing
- All 293 tests pass (231 macro + 36 coverage + 24 unit + 2 diagnostic)
- Added `test_class_with_non_ident_chars_ignored` to `coverage_tests.rs`
- Added `test_class_border_t/b/l/r` to `macro_tests.rs` (previously untested code path)

---

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
- **Optional `panic = "abort"` for release builds** - applications may enable
  `panic = "abort"` in their own release profile to remove unwind tables when that tradeoff
  fits their workspace.

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

[Unreleased]: https://github.com/wsafight/gpui-rsx/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/wsafight/gpui-rsx/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/wsafight/gpui-rsx/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/wsafight/gpui-rsx/compare/v0.4.4...v0.5.0
[0.4.4]: https://github.com/wsafight/gpui-rsx/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/wsafight/gpui-rsx/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/wsafight/gpui-rsx/compare/v0.4.1...v0.4.2
[0.4.0]: https://github.com/wsafight/gpui-rsx/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/wsafight/gpui-rsx/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/wsafight/gpui-rsx/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/wsafight/gpui-rsx/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/wsafight/gpui-rsx/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/wsafight/gpui-rsx/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/wsafight/gpui-rsx/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/wsafight/gpui-rsx/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wsafight/gpui-rsx/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wsafight/gpui-rsx/releases/tag/v0.1.0
