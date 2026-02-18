# Changelog

English | [简体中文](./CHANGELOG_CN.md)

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.2.1]: https://github.com/wsafight/gpui-rsx/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/wsafight/gpui-rsx/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/wsafight/gpui-rsx/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wsafight/gpui-rsx/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wsafight/gpui-rsx/releases/tag/v0.1.0
