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
│                          │                   │                  │
│                          └─────────┬─────────┘                  │
│                                    ▼                            │
│                  ┌──────────────────────────┐                   │
│                  │  element.rs (Generation) │                   │
│                  └────────────┬─────────────┘                   │
│                               │                                 │
│              ┌────────────────┘                                 │
│              ▼                                                   │
│  ┌──────────────────┐                                           │
│  │   runtime.rs     │  (dynamic class only)                     │
│  └──────────────────┘                                           │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Generated GPUI Code                          │
│  div().id("__rsx_div_0").flex().gap(px(4.0)).on_click(handler) │
└─────────────────────────────────────────────────────────────────┘
```

## Module Organization

```
src/
├── lib.rs                     (~123 lines) - Macro entry point
├── parser.rs                  (~311 lines) - RSX → AST
├── diagnostics.rs             (~110 lines) - Error messages
└── codegen/
    ├── mod.rs                 (~24 lines)  - Module orchestration
    ├── tables.rs              (~421 lines) - O(1) match-based lookup tables
    ├── class.rs               (~147 lines) - CSS class parsing
    ├── attribute.rs           (~79 lines)  - Attribute → method
    ├── element.rs             (~254 lines) - Element generation + auto ID
    └── runtime.rs             (~188 lines) - Dynamic class code generation
```

### Module Responsibilities

| Module | Purpose | Dependencies | Key Functions |
|--------|---------|--------------|---------------|
| `lib.rs` | Macro entry point | `parser`, `codegen` | `rsx!` macro |
| `parser.rs` | RSX syntax parsing | `syn`, `quote` | `parse()`, AST types |
| `diagnostics.rs` | Error messages | `syn` | span-aware error constructors |
| `codegen/tables.rs` | O(1) match lookups | None | `lookup_color()`, `lookup_attr_method()` |
| `codegen/class.rs` | Class parsing | `tables` | `parse_class_string()`, `parse_color_with_method()` |
| `codegen/attribute.rs` | Attribute processing | `tables`, `class`, `runtime` | `generate_attr_methods()` |
| `codegen/element.rs` | Element generation | All above | `generate_body()`, `generate_element()` |
| `codegen/runtime.rs` | Dynamic class gen | `class` | `generate_dynamic_class_code()` |

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

**Step 3a**: Single-pass attribute scan extracts `user_id`, `has_styled`, `needs_id`

```rust
// needs_id = true because onClick is stateful
// → auto ID injected
generate_base() → div().id("__rsx_div_0")
```

**Step 3b**: Class parsing — string literal → compile-time expansion

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
    .id("__rsx_div_0")
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

#### tables.rs — O(1) Lookup Foundation

**Purpose**: Central source of truth for all compile-time mappings.

**All lookups use `match` statements** — the compiler generates efficient jump tables or trie
structures, giving O(1) worst-case performance without any runtime initialisation cost.

**Functions**:

| Function | Entries | Description |
|----------|---------|-------------|
| `lookup_color(name)` | 242 | Full Tailwind palette (all shades + black/white) |
| `lookup_attr_method(name)` | 15 events + 30+ attrs | camelCase/snake_case → GPUI method |
| `lookup_spacing_method(prefix)` | 17 | `"gap_"`, `"px_"`, … → GPUI method name |
| `is_valid_text_size(size)` | 9 | `"xs"` … `"5xl"` whitelist |
| `lookup_tag_default(tag)` | 11 | Semantic default class strings |
| `is_stateful_attr(name)` | — | `starts_with("on_")` + explicit match |

**Design**: Zero dependencies, pure functions, no heap allocation.

#### class.rs — Class String Parsing

**Purpose**: Parse Tailwind-style class strings into GPUI method call `TokenStream`s.

**Key Innovations**:

1. **`split_ascii_whitespace` over `split_whitespace`** — CSS class names are always
   ASCII; skipping Unicode whitespace detection shaves one check per token boundary.

   ```rust
   // parse_class_string
   class_str.split_ascii_whitespace().map(parse_single_class)
   ```

2. **Unified `text_` prefix handling** — `text_color` and text-size paths are merged
   under a single `strip_prefix("text_")` call; previously `parse_color_class` and the
   size check each did their own strip, doubling work for every `text-*` class.

   ```rust
   // Before (two separate strip_prefix calls for text-xl, text-red-500, …):
   if let Some(color_code) = parse_color_class(&method_name) { return color_code; }
   if let Some(size) = method_name.strip_prefix("text_") { … }

   // After (one strip_prefix, two branches):
   if let Some(rest) = method_name.strip_prefix("text_") {
       if let Some(token) = parse_color_with_method(rest, "text_color") {
           return token;                                 // text-red-500 etc.
       }
       if is_valid_text_size(rest) {
           let size_ident = syn::Ident::new(&method_name, Span::call_site());
           return quote! { .#size_ident() };            // text-xl etc.
       }
   }
   if let Some(rest) = method_name.strip_prefix("bg_") {
       if let Some(token) = parse_color_with_method(rest, "bg") { return token; }
   }
   ```

   `parse_color_class` has been removed; its logic is now inlined above.

3. **Unified `parse_color_with_method(color, method)`** — shared by `text_color`, `bg`,
   and `border_color` paths, eliminating three near-identical implementations.

4. **Prefix lookup via `rfind('_') + match`** — O(1) spacing prefix detection without
   scanning the full string.

5. **Zero-allocation 3-digit hex expansion** — `[#abc]` → `0xaabbcc` via bitwise
   nibble duplication, no `String` allocated.

6. **`Cow<str>` for `-` → `_` conversion** — borrows the original slice when no
   hyphens are present; only allocates on replacement.

**Supported Patterns**:
- Named colors: `text-red-500` → `.text_color(rgb(0xef4444))`
- Arbitrary hex 6-digit: `bg-[#ff0000]` → `.bg(rgb(0xff0000))`
- Arbitrary hex 3-digit: `text-[#f00]` → `.text_color(rgb(0xff0000))`
- Spacing: `gap-4` → `.gap(px(4.0))`
- Text sizes: `text-xl` → `.text_xl()`
- Border: `border` → `.border_1()`, `border-2` → `.border_2()`

#### attribute.rs — Attribute-to-Method Mapping

**Purpose**: RSX attributes → GPUI method call `TokenStream`s

**Attribute Types**:
1. **Flag**: `<div flex />` → `.flex()`
2. **Value**: `<div width={100} />` → `.w(100)`
3. **Class (static)**: `<div class="flex" />` → `.flex()` (compile-time)
4. **Class (dynamic)**: `<div class={expr} />` → runtime match via `runtime.rs`
5. **Events**: `<div onClick={h} />` → `.on_click(h)`
6. **Conditional**: `<div when={(cond, |el| el.flex())} />` → `.when(cond, …)`

**Special Cases**:
- `invisible` → `.visible(false)`
- `styled` → Inject tag defaults (processed in `element.rs` before user attrs)
- `id` → Skipped here; handled in `element.rs` base generation

#### element.rs — Orchestration and Generation

**Purpose**: Orchestrate all code generation into a complete method chain.

**Key Concepts**:

1. **Method Chaining** — GPUI uses a fluent API where each method returns `Self` (or a
   new type after `.id()`):
   ```rust
   div().flex().gap(px(4.0)).child(...)
   ```

2. **Type Transformation** — `.id()` changes the return type:
   ```rust
   Div → Stateful<Div>
   ```
   Generated code must chain `.id()` before any stateful method.

3. **Early Fast-Path for Empty Elements** — If the element has no attributes and no
   children, the function returns immediately before any variable initialisation or
   loop execution. The check is deliberately placed *before* the attribute-scan loop:

   ```rust
   // Before: fast-path was *after* the loop (loop still initialised variables)
   // After: fast-path is the first thing in the function
   pub(crate) fn generate_element(element: &RsxElement) -> TokenStream {
       let tag_str = element.name.to_string();

       if element.attributes.is_empty() && element.children.is_empty() {
           return generate_tag(&tag_str, &element.name);  // ← returns here
       }

       // … attribute scan, base construction, etc. only reached when needed …
   }
   ```

4. **Single-Pass Attribute Scan** — `user_id`, `has_styled`, and `needs_id` are all
   extracted in one loop before any code is emitted:

   ```rust
   for attr in &element.attributes {
       match attr {
           RsxAttribute::Value { name, value } if name == "id" => user_id = Some(value),
           RsxAttribute::Flag(name) if name == "styled" => has_styled = true,
           RsxAttribute::Value { name, .. } | RsxAttribute::Flag(name) => {
               if !needs_id { needs_id = is_stateful_attr(&name.to_string()); }
           }
           _ => {}
       }
   }
   ```

5. **`Vec::with_capacity` Over-Estimate** — The method buffer is pre-allocated with
   `attributes.len() * 2 + children.len()` rather than `attributes.len() + children.len()`.
   A single `class` attribute can expand into 3–4 method calls
   (e.g. `"flex flex-col gap-4"` → 3 calls), so multiplying by 2 halves the expected
   number of reallocation events on class-heavy elements:

   ```rust
   // Before
   Vec::with_capacity(element.attributes.len() + element.children.len())

   // After
   Vec::with_capacity(element.attributes.len() * 2 + element.children.len())
   ```

6. **Auto ID Injection** — Elements with stateful attributes receive a deterministic ID:
   ```rust
   <div onClick={h} />
   ↓
   div().id("__rsx_div_0").on_click(h)
   ```

7. **Child Aggregation** — 2+ consecutive `Expr` children are batched into a single
   `.children([...])` call backed by a stack-allocated array, reducing method dispatch
   count compared to individual `.child()` chains. The threshold was lowered from 3 to 2
   because arrays have no heap-allocation cost:

   ```rust
   // 2+ consecutive expressions → single .children([...]) with stack array
   .children([expr1, expr2])

   // 1 expression → individual .child() call
   .child(expr1)
   ```

   Implementation in `generate_children_methods`:
   ```rust
   // Before: threshold was 3
   if consecutive_exprs.len() >= 3 {

   // After: threshold is 2 — arrays are stack-allocated, so 2 is free
   if consecutive_exprs.len() >= 2 {
       methods.push(quote! { .children([#(#consecutive_exprs),*]) });
   } else {
       for expr in &consecutive_exprs {
           methods.push(quote! { .child(#expr) });
       }
   }
   ```

8. **For-loop Code Generation** — Single child uses `.map()`; multiple children use
   `.flat_map()` with `vec![]` to support mixed element types:
   ```rust
   // Single child
   (iter).into_iter().map(|binding| child_expr)

   // Multiple children (vec! allows different element types)
   (iter).into_iter().flat_map(|binding| vec![child1, child2])
   ```

**Auto ID Counter**:
```rust
// Thread-local counter; increments monotonically per compile process.
// Known limitation: incremental builds may change expansion order,
// producing different IDs. Use explicit `id` for state-sensitive elements.
thread_local! {
    static AUTO_ID_COUNTER: Cell<usize> = const { Cell::new(0) };
}

fn next_auto_id(tag: &str) -> String {
    AUTO_ID_COUNTER.with(|c| {
        let n = c.get();
        c.set(n + 1);
        format!("__rsx_{tag}_{n}")
    })
}
```

#### runtime.rs — Dynamic Class Code Generation

**Purpose**: Generate runtime code for `class={expression}` attributes.

**Important Limitation**: Only the ~58 pre-compiled common classes are recognised at
runtime. Unknown classes are **silently ignored**. Prefer static string literals for
full class support.

**Pre-compiled common classes** (selected examples):
```
flex, flex-col, flex-row, flex-1, items-center, justify-center,
gap-1..gap-8, p-1..p-8, px-2, px-4, py-1..py-4, m-2, m-4,
w-full, h-full, text-xs..text-3xl, font-bold, border, rounded-*,
cursor-pointer, overflow-hidden, bg-white, bg-black, …
```

**Generated code pattern**:
```rust
{
    #[inline(never)]  // prevents match table inlining; enables LLVM ICF
    fn __rsx_apply_class<E: Styled>(el: E, class: &str) -> E {
        match class {
            "flex" => el.flex(),
            "gap-4" => el.gap(px(4.0)),
            // … ~58 pre-compiled classes …
            _ => el,  // unknown class → silently ignored
        }
    }
    let __class_expr = <expression>;
    let __class_str: &str = __class_expr.as_ref();  // zero-copy for &str
    // split_ascii_whitespace: faster than split_whitespace for ASCII-only class names
    if __class_str.is_empty() {
        __el                                         // fast-path: skip iterator creation
    } else {
        __class_str.split_ascii_whitespace().fold(__el, __rsx_apply_class)
    }
}
```

## Design Patterns

### 1. Match-Based O(1) Lookup Tables

**Pattern**: `match` statements inside pure functions instead of runtime hashmaps or
linear-scan const arrays.

```rust
pub(crate) fn lookup_color(name: &str) -> Option<u32> {
    match name {
        "red_500" => Some(0xef4444),
        "blue_500" => Some(0x3b82f6),
        // … 242 entries …
        _ => None,
    }
}
```

The Rust compiler generates an efficient jump table or trie for these match statements,
giving O(1) lookup with no runtime initialisation and zero heap allocation.

### 2. Recursive Descent Parsing

**Pattern**: Each syntax construct implements `syn::parse::Parse`

```rust
impl Parse for RsxBody {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![<]) && input.peek2(Token![>]) {
            // Fragment <>...</>
        } else {
            // Single element
        }
    }
}
```

### 3. Token Streaming — Push to Caller

**Pattern**: Generate `TokenStream` incrementally; attribute methods push directly into
the caller's `Vec` to avoid intermediate allocations.

```rust
pub(crate) fn generate_attr_methods(attr: &RsxAttribute, out: &mut Vec<TokenStream>) {
    // ... push directly into `out`, no intermediate Vec
    out.push(quote! { .flex() });
}
```

### 4. Method Chain Building

**Pattern**: Generate fluent API calls, not mutation-style assignments.

```rust
// WRONG: mutation pattern (breaks when .id() changes type)
let mut el = div();
el = el.flex();

// CORRECT: method chain
div().flex().gap(px(4.0))
```

**Why**: GPUI's `.id()` returns `Stateful<T>`, a different type. Chaining is the only
correct pattern.

### 5. Thread-Local Caching for Proc-Macro Context

**Pattern**: `thread_local! + Cell/RefCell` for state that must be shared across macro
invocations in the same compilation unit.

```rust
// proc macro runs single-threaded; thread_local is correct and cheaper than AtomicUsize
thread_local! {
    static AUTO_ID_COUNTER: Cell<usize> = const { Cell::new(0) };
    static COMMON_CLASS_MATCHES: RefCell<Option<Rc<Vec<TokenStream>>>> = ...;
}
```

## Testing Strategy

### Test Pyramid

```
               ┌──────────────────┐
               │ diagnostic_tests │  2 compile-error format tests
               └──────────────────┘
              ┌────────────────────┐
              │  coverage_tests    │  31 edge case / behaviour tests
              └────────────────────┘
            ┌──────────────────────┐
            │    macro_tests       │  203 expansion correctness tests
            └──────────────────────┘
```

### Macro Tests (tests/macro_tests.rs)

**Coverage**: 203 test cases

**Categories**:
- Elements (29): Tags, nesting, self-closing, special tags
- Attributes (45): Flags, values, camelCase/snake_case
- Events (18): All 15 event handlers + auto ID
- Styling (32): Classes, colors, spacing, border
- Children (24): Expr, spread, for loops, aggregation
- Conditional (12): when, whenSome
- Edge cases (43): Auto IDs, styled tags, fragments, invisible

**Pattern**:
```rust
#[test]
fn test_feature() {
    let result = quote! { rsx! { <div class="flex" /> } };
    let expected = quote! { div().flex() };
    assert_eq!(result.to_string(), expected.to_string());
}
```

## Extension Points

### Adding New Colors

**File**: `src/codegen/tables.rs` → `lookup_color()`

Add a new match arm:
```rust
pub(crate) fn lookup_color(name: &str) -> Option<u32> {
    match name {
        // …existing colors…
        "my_brand_500" => Some(0xabcdef),  // add here
        _ => None,
    }
}
```

**Usage**: `class="text-my-brand-500"` → `.text_color(rgb(0xabcdef))`

### Adding New Attribute Mappings

**File**: `src/codegen/tables.rs` → `lookup_attr_method()`

```rust
pub(crate) fn lookup_attr_method(name: &str) -> Option<&'static str> {
    match name {
        // …existing mappings…
        "customAttr" | "custom_attr" => Some("custom_attr"),  // add here
        _ => None,
    }
}
```

**Usage**: `<div customAttr={value} />` → `.custom_attr(value)`

### Adding New Event Handlers

**File**: `src/codegen/tables.rs` — two changes needed:

1. Add to `lookup_attr_method()`:
   ```rust
   "onCustom" | "on_custom" => Some("on_custom"),
   ```

2. If the event requires stateful element (`.id()`), also update `is_stateful_attr()`:
   ```rust
   pub(crate) fn is_stateful_attr(name: &str) -> bool {
       // on_ prefix already handled; add explicit names for camelCase:
       matches!(name, "hover" | "active" | … | "onCustom")
   }
   ```

### Adding New Spacing Prefixes

**File**: `src/codegen/tables.rs` → `lookup_spacing_method()`

```rust
pub(crate) fn lookup_spacing_method(prefix: &str) -> Option<&'static str> {
    match prefix {
        // …existing prefixes…
        "inset_" => Some("inset"),  // add here
        _ => None,
    }
}
```

**Usage**: `class="inset-4"` → `.inset(px(4.0))`

### Adding New Tag Default Styles

**File**: `src/codegen/tables.rs` → `lookup_tag_default()`

```rust
pub(crate) fn lookup_tag_default(tag: &str) -> Option<&'static str> {
    match tag {
        // …existing defaults…
        "nav" => Some("flex items-center"),  // add here
        _ => None,
    }
}
```

**Usage**: `<nav styled />` → `div().flex().items_center()`

### Adding Common Dynamic Classes

**File**: `src/codegen/runtime.rs` → `generate_common_class_matches()`

```rust
let common_classes = [
    // …existing classes…
    "my-custom-class",  // add here; will be pre-compiled into the match table
];
```

## Performance Considerations

### Compile Time

**Macro-expansion optimisations**:

| # | Location | Technique | Benefit |
|---|----------|-----------|---------|
| 1 | `tables.rs` | O(1) `match` lookups for colors, methods, spacing | No linear scan; compiler generates jump-table/trie |
| 2 | `element.rs` | Early fast-path before attribute-scan loop | Empty elements skip all variable init and loop entry |
| 3 | `element.rs` | Single-pass attribute scan (`user_id`, `has_styled`, `needs_id`) | One traversal instead of three |
| 4 | `element.rs` | `Vec::with_capacity(attrs * 2 + children)` | Halves reallocation events on class-heavy elements |
| 5 | `class.rs` | `split_ascii_whitespace` replaces `split_whitespace` | Skips Unicode whitespace scanning; class names are always ASCII |
| 6 | `class.rs` | Merged `text_` prefix handling (single `strip_prefix`) | Eliminates a redundant call for every `text-*` class |
| 7 | `class.rs` | `Cow<str>` for `-` → `_` conversion | Borrows without allocating when no hyphens present |
| 8 | `class.rs` | `rfind('_') + match` for spacing prefix | O(1) prefix detection vs. O(17) linear scan |
| 9 | `attribute.rs` | Direct push into caller's `Vec` | No intermediate `Vec` allocation per attribute |
| 10 | `runtime.rs` | Thread-local `Rc` cache for common class match arms | Generated once per process; `Rc::clone` is O(1) |

**Detailed notes on key optimisations**:

**#2 — Early fast-path** (`element.rs`):
```rust
pub(crate) fn generate_element(element: &RsxElement) -> TokenStream {
    let tag_str = element.name.to_string();
    // Fast-path: skip all scanning for bare tags like <div />
    if element.attributes.is_empty() && element.children.is_empty() {
        return generate_tag(&tag_str, &element.name);
    }
    // … rest of generation …
}
```

**#4 — Capacity estimate** (`element.rs`):
```rust
// Before: under-estimates when class attribute expands to many methods
Vec::with_capacity(element.attributes.len() + element.children.len())

// After: accounts for class expansion (~3-4 methods per class attribute)
Vec::with_capacity(element.attributes.len() * 2 + element.children.len())
```

**#5 + #6 — ASCII split and merged text_ prefix** (`class.rs`):
```rust
pub(crate) fn parse_class_string(class_str: &str) -> impl Iterator<Item = TokenStream> + '_ {
    class_str.split_ascii_whitespace().map(parse_single_class)
    //        ^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //        ASCII-only: no Unicode whitespace table lookup needed
}

pub(crate) fn parse_single_class(class: &str) -> TokenStream {
    // … spacing, border … then:

    // Single strip_prefix("text_") covers both color and size cases
    if let Some(rest) = method_name.strip_prefix("text_") {
        if let Some(token) = parse_color_with_method(rest, "text_color") {
            return token;                            // text-red-500 → .text_color(rgb(...))
        }
        if is_valid_text_size(rest) {
            return quote! { .#size_ident() };        // text-xl → .text_xl()
        }
    }
    if let Some(rest) = method_name.strip_prefix("bg_") {
        if let Some(token) = parse_color_with_method(rest, "bg") {
            return token;                            // bg-blue-500 → .bg(rgb(...))
        }
    }
    // default: bare method call
}
```

### Runtime (Generated Code Quality)

**Child aggregation threshold** (`element.rs`):

Consecutive `Expr` children are batched into `.children([...])` (stack-allocated array)
when there are 2 or more, instead of emitting individual `.child()` calls:

```rust
// 1 expression — individual call
.child(expr1)

// 2+ expressions — array batch; no heap allocation; fewer method dispatches
.children([expr1, expr2])
.children([expr1, expr2, expr3])
```

The threshold was lowered from 3 → 2 because `[T; N]` arrays are stack-allocated and
have no additional overhead over a single `.child()` call.

**Zero cost** — the generated GPUI code is identical to hand-written code:
```rust
// RSX
rsx! { <div class="flex gap-4" onClick={handler} /> }

// Generated (identical to handwritten after monomorphisation)
div().id("__rsx_div_0").flex().gap(px(4.0)).on_click(handler)
```

No reflection, no string parsing, no dynamic dispatch at runtime.

**Dynamic class exception** — `class={expression}` generates a runtime `fold` + `match`.
Use static string literals for zero-overhead styling.

### Binary Size (proc-macro)

- No runtime library linked into user binaries
- String literals interned by the linker
- `#[inline(never)]` on dynamic class helper prevents match table duplication
- LLVM ICF merges identical monomorphisations across components
- `panic = "abort"` in `[profile.release]` removes unwind tables from the proc-macro
  binary, reducing its size and load time during compilation:

  ```toml
  [profile.release]
  lto = true
  codegen-units = 1
  panic = "abort"   # proc-macros never need stack unwinding
  ```

## Debugging Guide

### Viewing Generated Code

```bash
# Install cargo-expand
cargo install cargo-expand

# View all expanded macros
cargo expand --lib

# Specific test
cargo test test_name -- --nocapture
```

### Understanding Errors

**Common pattern**:
```
error[E0599]: no method named `flex_col` found for struct `Div`
```

**Diagnosis**: Typo in class name — the class `flex-col` is not in the pre-compiled list
and is being passed as a method name literally.

**Fix**: Check the class name spelling and confirm it is in the supported list.

### Common Issues

| Error | Cause | Fix |
|-------|-------|-----|
| `no method named X` | Invalid GPUI method name | Check GPUI docs |
| `mismatched types` | `.id()` type change not handled | Verify auto ID is injected |
| Dynamic class not applied | Class not in ~58 common list | Use static string literal |
| Auto ID changes on rebuild | Incremental compile order change | Add explicit `id` attribute |
| `expected &str, found String` | Wrong type passed to `class={}` | Use `.as_str()` or a literal |

### Testing Changes

**Workflow**:
1. Modify code in `src/codegen/`
2. Run `cargo test` (all 236 tests)
3. Check a specific test: `cargo test test_name`
4. View generated code: `cargo expand --test macro_tests`

## Future Improvements

### Short Term

1. **More dynamic class coverage** — expand the ~58 pre-compiled class list based on usage data
2. **Compile-time warning for unknown classes** — emit a `proc_macro_warning` for unrecognised
   static class names
3. **More Tailwind utilities** — shadows, transforms, animations
4. **Custom color palette** — user-defined color tokens

### Medium Term

1. **LSP integration** — autocomplete for class names and attributes
2. **Snapshot tests** — generated code regression detection via `insta`
3. **Source maps** — better error locations pointing into RSX syntax
4. **`trybuild` compile-fail tests** — restore error-message validation

### Long Term

1. **Theme system** — dark mode, CSS custom properties
2. **Responsive design** — `class="md:flex lg:grid"`
3. **Accessibility** — ARIA attributes, semantic HTML
4. **Performance profiling** — macro expansion metrics with `criterion`

## Migration Guide

### From 0.1.x to 0.2.x

**Breaking changes**: None (internal refactoring only)

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

**Benefits**: ~50% less code, HTML-like structure, Tailwind familiarity, identical performance.

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

See [CONTRIBUTING.md](CONTRIBUTING.md) for code style guidelines, PR process, testing
requirements, and release procedure.

---

**Last Updated**: 2026-02-18
**Version**: 0.2.1
**Maintainers**: @wangshian

### Optimisation Changelog

| Date | File | Change |
|------|------|--------|
| 2026-02-18 | `class.rs` | `split_whitespace` → `split_ascii_whitespace` |
| 2026-02-18 | `class.rs` | Merged `text_` prefix handling; removed `parse_color_class` |
| 2026-02-18 | `element.rs` | Moved empty-element fast-path before attribute-scan loop |
| 2026-02-18 | `element.rs` | `Vec::with_capacity` estimate: `attrs + children` → `attrs * 2 + children` |
| 2026-02-18 | `element.rs` | `.children([...])` aggregation threshold: 3 → 2 |
| 2026-02-18 | `Cargo.toml` | Added `panic = "abort"` to `[profile.release]` |
