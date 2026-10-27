# GPUI-RSX 0.9.0 Roadmap

The `0.9.0-dev` branch starts from the published `v0.8.3` tag. It is a breaking-change
development line and must not be published until the migration surface is reviewed.

## Completed in the initial development slice

- Theme cache entries use `Arc<Theme>` instead of leaked `&'static Theme` values.
- `ClassMode` owns its optional theme handle and is explicitly cloneable rather than `Copy`.
- Macro expansion, tooling, LSP, preview and WASM callers were migrated to the owned theme.
- Theme cache tests verify shared allocation reuse and rebuild tracking.

## Planned before 0.9.0

1. Add stable constructors and builders for `ClassMode`, `Theme` and `MacroOptions`; stop
   requiring callers to construct public structs with implementation fields.
2. Define a stable `gpui-rsx-core` facade and mark syntax/result enums non-exhaustive. Keep
   parser and codegen implementation modules behind an explicitly internal boundary.
3. Decide whether SWC conversion remains a feature of `gpui-rsx-tools` or moves to a separate
   `gpui-rsx-convert` crate. The CLI and WASM playground must keep explicit feature wiring.
4. Add incremental LSP document updates and measure full-document versus range-based analysis
   on representative Rust files.
5. Benchmark dynamic class expansion before deciding whether a separate runtime crate reduces
   cold and incremental compile time.
6. Document the supported GPUI Kit/API window and provide migration notes for changed defaults,
   generated IDs, diagnostics and JSON output.

The branch may be tested and committed during this work, but it must not be merged into `main`
or published as `0.9.0` until all public API and migration checks are complete.
