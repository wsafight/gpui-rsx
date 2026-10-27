//! Macro-expansion cost (what users pay at compile time per `rsx!` invocation).
//!
//! Run: `cargo bench -p gpui-rsx-core --bench expansion`

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use gpui_rsx_core::{ClassMode, expand};
use proc_macro2::TokenStream;
use std::hint::black_box;

const DASHBOARD: &str = r#"
<div class="size-full flex flex-col gap-4 p-6 bg-zinc-950">
    <header class="flex items-center justify-between gap-2 border-b border-zinc-800 pb-4">
        <h1 class="text-3xl font-bold text-white">{"Dashboard"}</h1>
        <button class="px-3 py-2 rounded-md bg-blue-500 hover:bg-blue-600 active:opacity-75 text-white"
            onClick={handler}>{"New"}</button>
    </header>
    <section class="grid gap-3 w-full">
        {for row in rows {
            <div key={row.id} class="flex items-center gap-3 px-4 py-2 rounded-lg bg-zinc-900 hover:bg-zinc-800"
                onClick={row.on_click}>
                <span class="text-sm text-zinc-400 w-[120px] truncate">{row.name.clone()}</span>
                <span class="text-sm font-medium text-emerald-400 ml-auto">{row.value}</span>
            </div>
        }}
    </section>
</div>
"#;

const DYNAMIC: &str = r#"<div class={classes} />"#;

fn bench(c: &mut Criterion) {
    for (name, source, mode) in [
        ("expand/static_dashboard", DASHBOARD, ClassMode::PERMISSIVE),
        ("expand/strict_dashboard", DASHBOARD, ClassMode::STRICT),
        ("expand/dynamic_class", DYNAMIC, ClassMode::PERMISSIVE),
    ] {
        c.bench_function(name, |b| {
            b.iter_batched(
                || {
                    // Parsing the generated dynamic helper appends to proc-macro2's
                    // source map. Each invocation needs fresh span state, otherwise
                    // the measured cost depends on all earlier benchmark iterations.
                    // PerIteration drops the previous input/output before this reset.
                    proc_macro2::extra::invalidate_current_thread_spans();
                    source.parse::<TokenStream>().unwrap()
                },
                |input| expand(black_box(input), mode),
                BatchSize::PerIteration,
            )
        });
    }
}

fn configuration() -> Criterion {
    // Cargo launches this binary in the crate directory; keep its reports in the
    // workspace target directory even when Cargo does not export a CARGO variable.
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap();
    let output = std::env::var_os("CRITERION_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("CARGO_TARGET_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| workspace.join("target"))
                .join("criterion")
        });
    Criterion::default().output_directory(&workspace.join(output))
}

criterion_group! { name = benches; config = configuration(); targets = bench }
criterion_main!(benches);
