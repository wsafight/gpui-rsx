# Generate GPUI-RSX

Use the accompanying `rsx-spec.json` from the same checkout as gpui-rsx. Prefer
`rsx_strict!` and static class strings. Use only listed classes or documented numeric,
arbitrary color and length families. These are GPUI builder calls, not browser CSS.

- Numeric spacing/sizing defaults to pixels: `gap-4` is 4px. With `rsx_tw!`,
  `tailwind-scale`, or `[options] spacing-scale = "tailwind"`, it is 16px.
- `hover:`, `focus:`, `active:` and group variants work in static class literals.
  Responsive and `dark:` variants are unsupported. Express conditions in Rust.
- Dynamic `class={expr}` supports fewer utilities; it cannot apply state variants
  or theme tokens. Prefer literal branches such as `class={if active { "flex" } else { "hidden" }}`.
- Events take Rust callbacks. In loops, supply a stable `key={item.id}` when an
  element uses a stateful method or `active:`. Do not invent callback types.
- `img`, `svg` and `canvas` have special constructors. Custom components can use
  `base={Button::new("save")}` or `#[component]` structs/functions with required props and children.
- Nested `<>...</>` groups add no wrapper element. Function components use concrete prop
  types, `#[prop(default)]` / `#[prop(default = expr)]`, `#[prop(callback)]` for
  `Rc<dyn TraitAlias>`, and `#[children] Vec<AnyElement>`.
  Use `#[context]` only for `&mut Window` / `&mut App`; contexts are supplied during rendering.
- Import GPUI's prelude and the concrete types/functions used by the expansion.
  Validate the application with `cargo check`; macro expansion alone is not type checking.

Follow the examples in the spec. If a requested utility has no GPUI equivalent,
explain the limitation and use a typed GPUI attribute or Rust conditional instead.
