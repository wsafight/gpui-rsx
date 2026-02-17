# Changelog

English | [简体中文](./CHANGELOG_CN.md)

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
- 🌐 **English as primary README** - Switched README.md to English, moved Chinese to README_CN.md

#### Infrastructure
- ⚙️ **GitHub Actions workflows**
  - CI pipeline (`.github/workflows/ci.yml`)
  - GPUI compatibility testing (`.github/workflows/gpui-compatibility.yml`)
  - Release automation (`.github/workflows/release.yml`)
- 🧪 **Compile tests**
  - 7 compile-fail tests (invalid syntax detection)
  - 7 compile-pass tests (valid syntax verification)
  - Test runner script (`test_syntax.sh`)

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

### 🔧 Enhancements
- Parser improvements for better syntax support
- Code generation optimizations in `src/codegen.rs`
- Enhanced macro error reporting

### 🛠️ Internal
- Added `trybuild` for compile test support
- Improved project structure and organization
- Updated Cargo.toml keywords and categories

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

[0.2.0]: https://github.com/wsafight/gpui-rsx/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/wsafight/gpui-rsx/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wsafight/gpui-rsx/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wsafight/gpui-rsx/releases/tag/v0.1.0
