# Dioxus / GPUI-RSX hot-reload reproduction

This standalone workspace pins GPUI Pre 0.3.11 and the Dioxus RSX parser and
change detector to 0.7.10. It does not change the demo's dependency matrix or
add a runtime dependency to the published macro crates.

## Change detection without a window

From the repository root:

```sh
cargo test --manifest-path tests/fixtures/hot-reload/Cargo.toml --test change_detection --target-dir target/hot-reload-detection
```

These tests call Dioxus's actual `diff_rsx` and `CallBody` parser. Native GPUI
dependencies are optional and are not compiled for this command.

| Source edit | Dioxus 0.7.10 change detection |
| --- | --- |
| `div().child("before")` → `"after"` | Requires Rust compilation |
| `rsx! { <div>{"before"}</div> }` → `"after"` | Collected as a template; its parser rejects GPUI markup |
| `gpui_rsx::rsx! { ... }` | Same name collision |
| `use gpui_rsx::rsx as gpui_view; gpui_view! { ... }` | Requires Rust compilation |
| `rsx_strict!`, `rsx_permissive!`, `rsx_tw!`, `rsx_fragment!` | Requires Rust compilation |
| Dioxus's own `rsx! { div { "before" } }` | Collected as a template and parses successfully |

The parser tests establish how edits are classified. The native comparison below
checks whether the resulting Rust patch is applied and preserves the view.

## Native comparison

Use Dioxus CLI 0.7.10 (`dx --version`). An older CLI does not exercise the same
hot-patch implementation. The application enables GPUI's `hot-reload` feature,
prints its transport logs, and enables the platform's `font-kit` feature so that
macOS can render text.

Run one mode at a time, from this directory. Stop the previous `dx serve` and its
window before switching modes. Use a separate target directory per mode: switching
features in the same directory produced a zero-byte `libdeps-*.a` archive and a
fat-binary linker failure in the macOS 0.7.10 CLI during this investigation.

```sh
# Ordinary GPUI builder baseline
CARGO_TARGET_DIR=../../../target/hot-reload-builder dx serve --desktop --renderer native --hot-patch --hot-reload true --features hotreload,builder --bin hot-reload --interactive false --open false

# Original GPUI-RSX macro name
CARGO_TARGET_DIR=../../../target/hot-reload-markup dx serve --desktop --renderer native --hot-patch --hot-reload true --features hotreload,markup --bin hot-reload --interactive false --open false

# Local alias for the same macro
CARGO_TARGET_DIR=../../../target/hot-reload-alias dx serve --desktop --renderer native --hot-patch --hot-reload true --features hotreload --bin hot-reload --interactive false --open false
```

In `src/main.rs`, change only the active label: `builder-v1`, `markup-v1`, or
`alias-v1` to its `-v2` counterpart. Leave the other labels and Rust outside the
active macro unchanged. Otherwise, a normal Rust edit can trigger a patch and
hide the macro-name collision. Keep the window visible when checking the rendered
label; a patch-application log and an actual rendered frame are separate observations.

Each render prints and displays:

```text
GPUI_RSX_PATCH pid=12345 renders=4 label=alias-v1
```

Check all three observations: the label changes, the PID stays the same, and the
view's render counter continues increasing. The PID and counter distinguish a
patch from restarting the application. Restore the edited label after each run.

To test the alias reported in issue #111, stop the server, change the default
mode's import to `use gpui_rsx::rsx_permissive as view;` and its invocation to
`view!`, then restart the alias command and repeat the label-only edit.

Do not add `--hot-reload false`: CLI 0.7.10 skips file-change processing when it
is disabled, including automatic Rust patches. An alias preserves the selected
macro's class handling mode. The default `rsx!` is permissive; alias
`gpui_rsx::rsx_strict` instead if strict class checking is wanted.

## Observed results

On 2026-11-06, the native comparison ran on macOS arm64 with Rust 1.99.1,
Dioxus CLI 0.7.10 (`57d6794`) and GPUI Pre 0.3.11:

| Mode | Edit | Observed result |
| --- | --- | --- |
| Builder | Only `builder-v1` → `builder-v2` | Patch applied; PID and view state preserved |
| Original macro | Only `markup-v1` → `markup-v2` | No patch during an 8-second observation |
| Original macro | Then change Rust outside the macro | Patch applied with `markup-v2`; PID and view state preserved |
| Alias | Only `alias-v1` → `alias-v2` | Patch applied; PID and view state preserved |
| `rsx_permissive as view` | Only `alias-v1` → `alias-v2` | Patch applied; PID and view state preserved |
| Alias with `--hot-reload false` | Only `alias-v1` → `alias-v2` | No patch during an 8-second observation |

The original macro's successful patch after an outside Rust edit demonstrates
that GPUI-RSX expansion can be compiled and patched. The lost macro-only edit
occurs in the CLI's change-classification path. The alias works in this controlled
configuration; this does not establish why it failed in a different installation.
The render counters are fixture state; these checks do not cover every possible
application state or Rust type change. This fixture does not include `gpui-kit`.

## Visible state-preserving demo

From this directory, using Dioxus CLI 0.7.10:

```sh
CARGO_TARGET_DIR=../../../target/hot-reload-live dx serve --desktop --renderer native --hot-patch --hot-reload true --features hotreload --bin live-hot-reload --interactive false --open false
```

The window displays a colored revision panel, PID, seconds elapsed, and a click
counter. Click the button, then edit only the panel's text, color class, and
revision label inside `gpui_view!` in `src/live.rs`. The checked-in source displays
green revision B; for another demonstration, switch it to blue revision A. The
source patch should change the panel while preserving the PID, elapsed seconds,
and click count. The ticking timer alone is ordinary runtime state, not evidence
of source hot-reloading.

On 2026-11-07 (Asia/Shanghai), a live source edit changed the panel from blue A to
green B. The window visibly changed, the PID stayed the same, and the timer and
click counter retained their state. The CLI also reported `GPUI hot reload applied`.

## Upstream fallback

Dioxus 0.7.10's `packages/rsx-hotreload/src/collect.rs` identifies macros by the
last path segment (`rsx` or `render`), without resolving their crate. In
`packages/cli/src/serve/runner.rs`, `handle_file_change` then parses every collected
body as a Dioxus `CallBody`. If parsing fails, it executes `continue`, leaving
`needs_full_rebuild` false for an edit confined to GPUI markup.

The upstream fix should skip unchanged collected macros, then set
`needs_full_rebuild = true` and leave the loop when either changed body cannot be
parsed. The existing rebuild branch already chooses Rust hot-patching when
`--hot-patch` is enabled. This also needs an upstream regression
test covering the CLI's fallback; the parser tests here do not claim to test the
entire CLI control flow.
