# gpui-rsx-core

Parser, class analysis, and GPUI code generation shared by the `gpui-rsx` macros
and development tools. This is a normal Rust library; GPUI itself is not required
to parse RSX or inspect the generated tokens.

```rust
use gpui_rsx_core::{ClassMode, expand_str};

let expanded = expand_str(r#"<div class="flex gap-4" />"#, ClassMode::STRICT)?;
println!("{expanded}");
# Ok::<(), syn::Error>(())
```

See the [repository](https://github.com/wsafight/gpui-rsx) for the public macro API,
compatibility revisions, syntax specification, and contribution instructions.

Since 0.9, use the crate-root facade and `syntax` module. Construct `ClassMode`, `Theme`
and `MacroOptions` using builders or `with_*` methods; their implementation fields are
private. `ClassMode` is `Clone` and owns an `Arc<Theme>`. Syntax and result enums are
non-exhaustive, so downstream matches need a fallback arm. Legacy `parser` and `codegen`
paths remain available as hidden implementation details.

`resolve_style` returns a `StyleOperation` shared by code generation and preview tooling.
Its values preserve colors, lengths, opacity and opaque Rust expressions; inspecting a
style never evaluates application code. `Theme::builder().try_build()` validates token
types and expressions for in-memory configurations.
