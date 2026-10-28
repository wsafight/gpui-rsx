# GPUI-RSX 0.9.0 Roadmap

The `0.9.0-dev` branch starts from the published `v0.8.3` tag. Version 0.9.0 adds the
breaking API changes and migration guidance described below.

## Implemented in development order

1. Owned, bounded Theme cache: canonical paths, content fingerprints, 64-entry LRU eviction,
   shared `Arc<Theme>` handles and config rebuild tracking.
2. Public API boundary: private configuration fields, constructors/builders/getters,
   crate-root facade, `syntax` module and non-exhaustive syntax/result enums. Historical
   parser/codegen module paths remain accessible but hidden from generated documentation.
3. Design tokens: text size, line height, opacity and shadow sections; programmatic validation,
   duplicate/collision errors, and `cargo rsx theme check/list/explain` with JSON output.
   LSP config diagnostics and static theme color swatches use the same resolution.
4. Incremental LSP: ordered UTF-16 range edits, stale-version protection, unchanged macro-body
   analysis reuse and semantic tokens. Tests compare incremental results with full analysis
   after Unicode, structural, duplicate-macro and malformed-source edits.
5. Shared style operations: macro generation and preview resolve lengths, colors, opacity,
   typography and presets through core; preview supports static theme tokens and styled defaults.
6. Optional conversion: tools default features exclude SWC; CLI and WASM explicitly enable
   `convert`. CI and the local check script exercise tooling without conversion too.
7. Dynamic helper sharing: one helper per macro invocation, including nested children,
   fragments and loops; each attribute retains its expression and unknown-class callback.
   Expansion snapshots, runtime behavior tests and the real GPUI API contract cover the change.
8. Bilingual migration, Theme tooling and performance documentation. Compatibility remains
   GPUI Kit 0.8.1 / gpui-pre 0.3.10 / gpui-component 0.8.1. Macro/core MSRV is Rust 1.88,
   tools MSRV is 1.98, and demo checks use 1.98.1.

## Measurement boundaries and release review

- Local short Criterion runs measured four dynamic class attributes at about 1.99 ms before
  sharing and 0.495 ms after sharing; the static dashboard remained around 66.7 µs. Full
  analysis of 80 macros was around 1.00 ms versus 223 µs for editing one macro body.
  Reproduction commands and limitations are in the performance guide.
- Incremental analysis still rescans the Rust token stream to discover macros safely. It
  avoids reparsing unchanged RSX bodies; it is not an incremental Rust parser.
- Helper sharing stops at each macro invocation. A separate runtime crate is deferred until
  whole-project cold/incremental build measurements justify the dependency and API cost.
- Browser preview remains an approximation. Runtime theme expressions, events and native
  GPUI behavior require application review; arbitrary Rust expressions are never executed.
- Existing pixel spacing and generated ID behavior are preserved. Public API and feature
  default migrations must be reviewed before release. CLI Theme validation uses exit status
  1 for config/token errors and 2 for usage errors.
- Final local checks cover workspace tests/Clippy, no-default-feature tooling, MSRVs, real
  GPUI demo, docs, Playground WASM, editor packages and publish dry-runs. Remote platform CI
  and native interactive review remain part of the eventual merge/release process.

## Release validation

The local release suite, macro/tooling MSRVs, strict Rust API documentation, Playground
WASM checks, editor package builds and all four publish dry-runs passed. Release proceeds
after the pull request passes remote CI, followed by registry and release artifact verification.
