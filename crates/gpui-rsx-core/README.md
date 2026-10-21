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
