# GPUI-RSX Architecture

## Overview

GPUI-RSX is a procedural macro that provides JSX-like syntax for the GPUI UI framework. It transforms HTML-like markup into idiomatic GPUI method chains at compile time, achieving **zero runtime overhead** through compile-time code generation.

### Core Philosophy

- **Zero-cost abstraction**: All transformations happen at compile time
- **Type safety**: Generated code leverages Rust's type system
- **GPUI-native**: Output matches handwritten GPUI code patterns
- **Tailwind-inspired**: Familiar utility-class styling system

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Code (RSX)                         │
│  rsx! { <div class="flex gap-4" onClick={handler}> ... </div> }│
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Parser (parser.rs)                           │
│  • Tokenization                                                 │
│  • Recursive descent parsing                                    │
│  • AST construction                                             │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
                    ┌────────┐
                    │  AST   │
                    └────┬───┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                 Code Generator (codegen/)                       │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   tables.rs  │  │   class.rs   │  │ attribute.rs │         │
│  │ (Lookups)    │◄─┤ (Parsing)    │◄─┤ (Methods)    │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                              │                  │
│                         ┌────────────────────┘                  │
│                         │                                       │
│                         ▼                                       │
│                  ┌──────────────┐                               │
│                  │  element.rs  │                               │
│                  │ (Generation) │                               │
│                  └──────────────┘                               │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Generated GPUI Code                          │
│  div().id("auto_0").flex().gap(px(4.0)).on_click(handler).child│
└─────────────────────────────────────────────────────────────────┘
```

## Module Organization

```
src/
├── lib.rs                     (124 lines)  - Macro entry point
├── parser.rs                  (371 lines)  - RSX → AST
└── codegen/
    ├── mod.rs                 (~20 lines)  - Module orchestration
    ├── tables.rs              (~450 lines) - Static lookup tables
    ├── class.rs               (~150 lines) - CSS class parsing
    ├── attribute.rs           (~80 lines)  - Attribute → method
    └── element.rs             (~230 lines) - Element generation
```

### Module Responsibilities

| Module | Purpose | Dependencies | Key Functions |
|--------|---------|--------------|---------------|
| `lib.rs` | Macro entry point | `parser`, `codegen` | `rsx!` macro |
| `parser.rs` | RSX syntax parsing | `syn`, `quote` | `parse()`, AST types |
| `codegen/tables.rs` | Const tables | None | `lookup_color()` |
| `codegen/class.rs` | Class parsing | `tables` | `parse_class_string()`, `parse_color_with_method()` |
| `codegen/attribute.rs` | Attribute processing | `tables`, `class` | `generate_attr_methods()` |
| `codegen/element.rs` | Element generation | All above | `generate_body()`, `generate_element()` |

## Data Flow

### 1. Macro Invocation

```rust
rsx! {
    <div class="flex gap-4 bg-blue-500" onClick={handler}>
        {"Hello"}
    </div>
}
```

### 2. Parser Phase

**Input**: `TokenStream` from `rsx!` macro
**Output**: `RsxBody` AST

```rust
RsxBody::Single(
    RsxElement {
        name: Ident("div"),
        attributes: [
            RsxAttribute::Value {
                name: "class",
                value: Lit("flex gap-4 bg-blue-500")
            },
            RsxAttribute::Value {
                name: "onClick",
                value: Expr(handler)
            }
        ],
        children: [
            RsxNode::Expr(Lit("Hello"))
        ]
    }
)
```

### 3. Code Generation Phase

**Step 3a**: Element base construction

```rust
generate_base() → div().id("__rsx_div_a1b2c3d4")
```

**Step 3b**: Class parsing (with deduplication)

```rust
parse_class_string("flex gap-4 bg-blue-500") → [
    .flex(),
    .gap(px(4.0)),
    parse_color_with_method("blue_500", "bg") → .bg(rgb(0x3b82f6))
]
```

**Step 3c**: Attribute conversion

```rust
generate_attr_methods(onClick={handler}) → .on_click(handler)
```

**Step 3d**: Children processing

```rust
generate_children_methods([Expr("Hello")]) → .child("Hello")
```

### 4. Final Output

```rust
div()
    .id("__rsx_div_a1b2c3d4")
    .flex()
    .gap(px(4.0))
    .bg(rgb(0x3b82f6))
    .on_click(handler)
    .child("Hello")
```

## Key Components

### Parser (parser.rs)

**Architecture**: Recursive descent parser using `syn::parse::Parse`

**AST Types**:
- `RsxBody`: Top-level (Single element or Fragment)
- `RsxElement`: Tag with attributes and children
- `RsxNode`: Element | Expr | Spread | For
- `RsxAttribute`: Flag | Value | When | WhenSome

**Key Features**:
- Fragment support (`<>...</>`)
- For loop syntax (`{for item in items { ... }}`)
- Conditional rendering (`when`, `whenSome`)
- Expression children (`{expr}`)
- Spread syntax (`{...items}`)

### Code Generator (codegen/)

#### tables.rs - Foundation

**Purpose**: Central source of truth for all mappings

**Contents**:
- `COLOR_MAP`: 242 Tailwind colors (slate, gray, ... rose)
- `EVENT_HANDLERS`: 14 event mappings (onClick → on_click)
- `ATTRIBUTE_NAME_MAP`: 30+ camelCase → snake_case
- `TAG_DEFAULT_STYLES`: 11 semantic tag defaults
- `SPACING_PATTERNS`: 17 spacing/sizing prefixes
- `VALID_TEXT_SIZES`: 9 text size variants
- `lookup_color()`: Fast color table search

**Design**: Zero dependencies, pure const data

#### class.rs - Deduplication

**Purpose**: Parse class strings into method calls

**Key Innovation**: `parse_color_with_method()`

**Before refactoring** (duplicated 3 times):
```rust
// text_color
if let Some(color) = class.strip_prefix("text_") {
    for &(color_name, color_value) in COLOR_MAP {
        if color == color_name {
            return Some(quote! { .text_color(rgb(#color_value)) });
        }
    }
}

// bg (same logic repeated)
// border_color (same logic repeated again)
```

**After refactoring** (unified):
```rust
fn parse_color_with_method(color: &str, method: &str) -> Option<TokenStream> {
    // 1. Try color table
    if let Some(hex) = lookup_color(color) {
        let ident = syn::Ident::new(method, Span::call_site());
        return Some(quote! { .#ident(rgb(#hex)) });
    }
    // 2. Try arbitrary hex
    if let Some(hex) = parse_arbitrary_hex(color) {
        let ident = syn::Ident::new(method, Span::call_site());
        return Some(quote! { .#ident(rgb(#hex)) });
    }
    None
}

// Usage
parse_color_with_method(color, "text_color")
parse_color_with_method(color, "bg")
parse_color_with_method(color, "border_color")
```

**Benefits**:
- DRY: 3 implementations → 1
- Maintainability: Single point of change
- Consistency: All colors handled identically

**Supported Patterns**:
- Named colors: `text-red-500` → `.text_color(rgb(0xef4444))`
- Arbitrary hex: `bg-[#ff0000]` → `.bg(rgb(0xff0000))`
- Short hex: `text-[#f00]` → `.text_color(rgb(0xff0000))`
- Spacing: `gap-4` → `.gap(px(4.0))`
- Text sizes: `text-xl` → `.text_xl()`

#### attribute.rs - Mapping

**Purpose**: RSX attributes → GPUI methods

**Attribute Types**:
1. **Flag**: `<div flex />` → `.flex()`
2. **Value**: `<div width={100} />` → `.w(100)`
3. **Class**: `<div class="flex" />` → `.flex()`
4. **Events**: `<div onClick={h} />` → `.on_click(h)`
5. **Conditional**: `<div when={cond, |el| el.flex()} />` → `.when(cond, |el| el.flex())`

**Special Cases**:
- `invisible` → `.visible(false)`
- `styled` → Inject tag defaults (processed in element.rs)
- `id` → Skip (handled in element.rs base generation)
- `class` → Must be string literal (no dynamic values)

#### element.rs - Generation

**Purpose**: Orchestrate all code generation

**Key Concepts**:

1. **Method Chaining**: GPUI uses fluent API
   ```rust
   div().flex().gap(px(4.0)).child(...)
   ```

2. **Type Transformations**: `.id()` changes type
   ```rust
   Div → Stateful<Div>
   ```

3. **Auto ID Injection**: Events require stateful elements
   ```rust
   <div onClick={h} />
   ↓
   div().id("__rsx_div_a1b2c3d4").on_click(h)
   ```

4. **Child Aggregation**: Optimize consecutive children
   ```rust
   // 3+ expressions
   .children(vec![expr1, expr2, expr3])
   // vs
   .child(expr1).child(expr2)
   ```

5. **Default Styles**: `styled` flag injects semantics
   ```rust
   <h1 styled>{"Title"}</h1>
   ↓
   div().text_3xl().font_bold().child("Title")
   ```

**Auto ID Algorithm**:
```rust
fn next_auto_id(tag: &str, attrs: &[Attr]) -> String {
    let mut hasher = DefaultHasher::new();
    tag.hash(&mut hasher);
    for attr in attrs {
        attr.name.hash(&mut hasher);
    }
    let counter = AUTO_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    counter.hash(&mut hasher);
    format!("__rsx_{tag}_{:x}", hasher.finish())
}
```

## Design Patterns

### 1. Compile-Time Tables

**Pattern**: `const` lookup tables instead of runtime hashmaps

```rust
const COLOR_MAP: &[(&str, u32)] = &[
    ("red_500", 0xef4444),
    // ...
];

fn lookup_color(name: &str) -> Option<u32> {
    COLOR_MAP.iter().find(|(n, _)| *n == name).map(|(_, v)| *v)
}
```

**Benefits**:
- Zero runtime cost
- No allocations
- Binary size efficient (string interning)

### 2. Recursive Descent Parsing

**Pattern**: Each syntax construct has a dedicated parser

```rust
impl Parse for RsxBody {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![<]) {
            if input.peek2(Token![>]) {
                // Fragment
            } else {
                // Single element
            }
        }
    }
}
```

### 3. Token Streaming

**Pattern**: Generate `TokenStream` incrementally

```rust
let mut methods = Vec::new();
methods.push(quote! { .flex() });
methods.push(quote! { .gap(px(4.0)) });
quote! { div() #(#methods)* }
```

**Benefits**:
- Composable
- Type-safe
- Preserves span information for errors

### 4. Method Chain Building

**Pattern**: Generate fluent API calls

```rust
// WRONG: Mutation pattern
let mut el = div();
el = el.flex();
el = el.gap(px(4.0));

// RIGHT: Method chain
div().flex().gap(px(4.0))
```

**Why**: GPUI methods often change type (`.id()` returns `Stateful<T>`)

## Testing Strategy

### Test Pyramid

```
                   ┌──────────────┐
                   │ Integration  │  Examples (manual)
                   │   Tests      │
                   └──────────────┘
                  ┌────────────────┐
                  │  Macro Tests   │  203 expansion tests
                  │                │
                  └────────────────┘
```

### Macro Tests (tests/macro_tests.rs)

**Coverage**: 203 test cases

**Categories**:
- Elements (29): Tags, nesting, self-closing
- Attributes (45): Flags, values, camelCase/snake_case
- Events (18): All 14 event handlers
- Styling (32): Classes, colors, spacing
- Children (24): Expr, spread, for loops
- Conditional (12): when, whenSome
- Edge cases (43): Auto IDs (including onHover/onDrag/onDrop), styled tags, fragments

**Pattern**:
```rust
#[test]
fn test_feature() {
    let expanded = quote! {
        rsx! { <div class="flex" /> }
    };
    let expected = quote! {
        div().flex()
    };
    assert_eq!(expanded.to_string(), expected.to_string());
}
```

## Extension Points

### Adding New Colors

**File**: `src/codegen/tables.rs`

```rust
const COLOR_MAP: &[(&str, u32)] = &[
    // ...existing colors...
    ("my_custom_500", 0xabcdef),  // Add here
];
```

**Usage**: `class="text-my-custom-500"` → `.text_color(rgb(0xabcdef))`

### Adding New Attributes

**File**: `src/codegen/tables.rs`

```rust
const ATTRIBUTE_NAME_MAP: &[(&str, &str)] = &[
    // ...existing mappings...
    ("customAttr", "custom_attr"),  // Add here
];
```

**Usage**: `<div customAttr={value} />` → `.custom_attr(value)`

### Adding New Event Handlers

**File**: `src/codegen/tables.rs`

```rust
const EVENT_HANDLERS: &[(&str, &str, &str)] = &[
    // ...existing handlers...
    ("onCustom", "on_custom", "on_custom"),  // Add here
];
```

**Usage**: `<div onCustom={h} />` → `.on_custom(h)` (with auto ID)

### Adding New Spacing Patterns

**File**: `src/codegen/tables.rs`

```rust
const SPACING_PATTERNS: &[(&str, &str)] = &[
    // ...existing patterns...
    ("custom_", "custom"),  // Add here
];
```

**Usage**: `class="custom-4"` → `.custom(px(4.0))`

### Adding New Default Styles

**File**: `src/codegen/tables.rs`

```rust
const TAG_DEFAULT_STYLES: &[(&str, &str)] = &[
    // ...existing defaults...
    ("myTag", "flex gap-2"),  // Add here
];
```

**Usage**: `<myTag styled />` → `myTag().flex().gap(px(2.0))`

## Performance Considerations

### Compile Time

**Optimizations**:
1. **Const tables**: No runtime initialization
2. **Linear lookups**: Small tables (< 500 entries)
3. **No allocations**: Stack-based parsing
4. **Minimal cloning**: TokenStream reuse

**Benchmarks**: ~0.1s for 1000 element macro expansion

### Runtime

**Zero cost**:
- No reflection
- No string parsing
- No dynamic dispatch
- Identical to handwritten GPUI code

**Generated code**:
```rust
// RSX
rsx! { <div class="flex" /> }

// Handwritten (identical after monomorphization)
div().flex()
```

### Binary Size

**Impact**: Minimal

**Why**:
- No runtime library
- String literals interned
- Method calls inlined
- Dead code eliminated

## Debugging Guide

### Viewing Generated Code

```bash
# Install cargo-expand
cargo install cargo-expand

# View expanded macro
cargo expand --lib

# Specific test
cargo expand --test macro_tests --tests test_name
```

### Understanding Errors

**Compile error**:
```
error[E0599]: no method named `flex_col` found for struct `Div`
```

**Diagnosis**: Typo in class name (`flex-col` vs `flex_col`)

**Fix**: Use correct Tailwind class `flex-col`

### Testing Changes

**Workflow**:
1. Modify code in `src/codegen/`
2. Run `cargo test --test macro_tests`
3. Check specific test: `cargo test test_name`
4. View expansion: `cargo expand --test macro_tests`
5. Compare: `diff <(cargo expand) expected.rs`

### Common Issues

| Error | Cause | Fix |
|-------|-------|-----|
| "no method named X" | Invalid GPUI method | Check GPUI docs |
| "mismatched types" | `.id()` type change | Verify auto ID insertion |
| "expected struct `Div`" | Missing auto ID | Check `NEEDS_ID_ATTRS` |
| "cannot find value" | Scope issue | Check expression escaping |

## Future Improvements

### Short Term

1. **Component support**: `<MyComponent prop={value} />`
2. **Ref handling**: `ref={my_ref}`
3. **More Tailwind utilities**: Shadows, transforms, animations
4. **Custom color palette**: User-defined colors

### Medium Term

1. **LSP integration**: Autocomplete for classes
2. **Compile-time validation**: Warn on unknown classes
3. **Hot reload**: Fast iteration during development
4. **Source maps**: Better error locations

### Long Term

1. **Theme system**: Dark mode, color schemes
2. **Responsive design**: `class="md:flex lg:grid"`
3. **Accessibility**: ARIA attributes, semantic HTML
4. **Performance profiling**: Macro expansion metrics

## Migration Guide

### From 0.1.x to 0.2.x

**Breaking changes**: None (internal refactoring only)

**Verification**:
```bash
# Same output guaranteed
cargo expand --lib > before.rs
# Upgrade
cargo update -p gpui-rsx
cargo expand --lib > after.rs
diff before.rs after.rs  # Should be empty
```

### From Handwritten GPUI

**Before**:
```rust
div()
    .flex()
    .flex_col()
    .gap(px(16.0))
    .bg(rgb(0x3b82f6))
    .child("Hello")
```

**After**:
```rust
rsx! {
    <div class="flex flex-col gap-4 bg-blue-500">
        {"Hello"}
    </div>
}
```

**Benefits**:
- 40% less code
- HTML-like structure
- Tailwind familiarity
- Same performance

## References

### Documentation

- [GPUI Documentation](https://www.gpui.rs/)
- [Tailwind CSS](https://tailwindcss.com/)
- [syn crate](https://docs.rs/syn/)
- [quote crate](https://docs.rs/quote/)

### Related Projects

- [dioxus](https://dioxuslabs.com/): RSX for web/desktop
- [yew](https://yew.rs/): RSX for WebAssembly
- [leptos](https://leptos.dev/): RSX with signals

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Code style guidelines
- PR process
- Testing requirements
- Release procedure

---

**Last Updated**: 2026-02-17
**Version**: 0.2.0
**Maintainers**: @wangshian
