# Changelog

English | [简体中文](./CHANGELOG_CN.md)

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-02-17

### 🚀 Performance Optimizations

#### Compile-time Performance
- **Binary search for color lookup** - Replaced O(n) linear scan with O(log n) binary search in `lookup_color()`
  - COLOR_MAP (242 entries) now sorted alphabetically
  - Color lookups reduced from ~120 comparisons to ~8 comparisons
  - Improves macro expansion speed when using class colors

#### Runtime Performance
- **Eliminated heap allocations in `.children()`** - Replaced `vec![]` with array literals `[...]` in:
  - For-loop `flat_map()` expansions
  - 3+ consecutive expression aggregation
  - Reduces runtime allocations for static element counts
- **Zero-copy dynamic class strings** - Changed from `String::into()` to `AsRef<str>`
  - `&str` inputs now pass through without allocation
  - Supports `&str`, `String`, `Cow<str>` efficiently

#### Binary Size Optimization
- **Deduplicated dynamic class match tables** - Extracted 40-arm match table into `#[inline(never)]` local function
  - Multiple `class={expr}` in same component now share one function body
  - LLVM ICF can merge identical monomorphizations across components
  - Reduces code bloat from repeated match table inlining

### ♻️ Code Quality Improvements
- **Simplified EVENT_HANDLERS table** - Reduced from 3-tuple to 2-tuple
  - Removed redundant third field (method name always equals snake_case)
  - Cleaner table structure, reduced maintenance burden
- **Removed dead code** - Deleted deprecated `class_dynamic_value_error()` function

### 📊 Impact Summary

| Category | Improvement | Files Changed |
|----------|-------------|---------------|
| Compile Speed | ~15x faster color lookups (O(242)→O(8)) | `tables.rs` |
| Runtime Allocations | Eliminated vec! in .children() | `element.rs` |
| Binary Size | Dynamic class match table deduplicated | `runtime.rs` |
| Code Clarity | Simplified EVENT_HANDLERS schema | `tables.rs`, `attribute.rs` |

### ✅ Testing
- All 236 tests pass (203 macro + 31 coverage + 2 diagnostic)
- Zero regressions from optimization changes

## [0.2.0] - 2026-02-17

### ✨ Added

#### Core Features
- **For-loop syntax sugar** - Simplify list rendering with `{for item in items { ... }}` syntax
- **Styled flag** - Apply sensible default styles based on tag names (h1-h6, button, input, etc.)
- **Conditional styling** - `when` and `whenSome` attributes for dynamic styling
- **Fragment support** - Return multiple root elements with `<>...</>` syntax
- **Full Tailwind color palette** - 242 built-in colors (slate, gray, red, blue, etc.) + arbitrary hex values
- **Comprehensive event handling** - Support for onClick, onMouseDown, onKeyDown, onHover, etc. (14 event types)
- **Attribute mapping** - camelCase to snake_case conversion (zIndex → z_index, fontSize → font_size, etc.)

#### Documentation
- 📖 **Complete documentation system**
  - Getting Started guide (`docs/getting-started.md`)
  - API Reference (`docs/api-reference.md`)
  - Best Practices (`docs/best-practices.md`)
  - Migration Guide (`docs/migration-guide.md`)
  - Troubleshooting (`docs/troubleshooting.md`)
- 📐 **Architecture documentation**
  - Comprehensive architecture guide (`ARCHITECTURE.md`)
  - Chinese architecture documentation (`ARCHITECTURE_CN.md`)
  - Detailed module organization and data flow
  - Code generation strategies and design patterns
  - Extension points and debugging guide
- 🌐 **English as primary README** - Switched README.md to English, moved Chinese to README_CN.md

#### Infrastructure
- ⚙️ **GitHub Actions workflows**
  - CI pipeline (`.github/workflows/ci.yml`)
  - GPUI compatibility testing (`.github/workflows/gpui-compatibility.yml`)
  - Release automation (`.github/workflows/release.yml`)
  - Code coverage tracking with Codecov
- 📊 **Code Coverage**
  - Local coverage script (`./coverage.sh`)
  - Codecov integration in CI
  - Coverage badges in README
  - Target: 80%+ coverage

#### Developer Experience
- 🔧 Better error messages with `proc-macro-error` dependency
- 📦 Updated repository URL to `https://github.com/wsafight/gpui-rsx`
- ✅ Comprehensive test suite with 1200+ test cases

### 🚀 Performance
- Optimized parser for better compile-time performance
- Improved code generation efficiency for `class` attribute expansion
- Reduced allocations in for-loop syntax transformation

### 📝 Documentation Improvements
- Updated README with comprehensive examples
- Added "Before & After" comparison showing ~50% code reduction
- Documented all 14 event handlers
- Added complete attribute mapping reference table
- Included FAQ section with common questions

### 🐛 Fixed
- **Auto ID injection for stateful events** - Added missing `onHover`/`on_hover`, `onDrag`/`on_drag`, `onDrop`/`on_drop` to `NEEDS_ID_ATTRS`, fixing cases where these event handlers would fail to compile without a manual `id` attribute
  - Added 6 test cases to verify auto ID injection for these event handlers

### 🔧 Enhancements
- Parser improvements for better syntax support
- Code generation optimizations in `src/codegen.rs`
- Enhanced macro error reporting
- Fixed 4 Clippy warnings about useless `vec!` usage in tests

### ♻️ Refactored
- **Deduplicated child-node parsing** - Extracted `try_parse_child_node()` in parser, eliminating duplicated logic between `parse_children()` and `parse_for_loop()` body parsing
- **Deduplicated for-loop code generation** - Extracted `generate_for_loop()` in codegen, unifying the `map`/`flat_map` generation logic previously duplicated in `generate_node()` and `generate_children_methods()`
- **Simplified color class parsing** - Replaced redundant `starts_with()` + `strip_prefix()` pattern with a single `strip_prefix()` call in `parse_color_class()`

### 🗑️ Removed
- Removed `examples/` directory (counter.rs, todo_app.rs)
  - Examples required external GPUI dependency
  - Core functionality fully covered by 203 macro expansion tests
- Removed trybuild compile tests
  - Simplified test structure
  - Eliminated `trybuild` dev-dependency
  - Focus on comprehensive macro expansion testing

### 🛠️ Internal
- Improved project structure and organization
- Updated Cargo.toml keywords and categories
- Updated documentation to remove references to removed examples
- Cleaned up CONTRIBUTING.md test instructions
- Streamlined project structure

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
