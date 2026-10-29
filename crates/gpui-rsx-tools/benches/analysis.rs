use criterion::{Criterion, criterion_group, criterion_main};
use gpui_rsx_core::ClassMode;
use gpui_rsx_tools::{analyze_rust_document, release_span_memory, update_rust_document};
use std::hint::black_box;

fn analysis(c: &mut Criterion) {
    let source = (0..80)
        .map(|i| format!("fn view_{i}(){{rsx!{{<div class=\"flex gap-4 bg-blue-500\"><span class=\"text-sm\"/></div>}};}}\n"))
        .collect::<String>();
    let previous = analyze_rust_document(&source, ClassMode::PERMISSIVE);
    let edited = source.replacen("gap-4", "gap-8", 1);
    release_span_memory();
    c.bench_function("analysis/full_80_macros", |b| {
        b.iter(|| {
            black_box(analyze_rust_document(
                black_box(&edited),
                ClassMode::PERMISSIVE,
            ));
            release_span_memory();
        })
    });
    let dynamic = (0..80)
        .map(|i| {
            format!(
                "fn view_{i}(c:&str){{rsx!{{<div class={{c}}><span class=\"text-sm\"/></div>}};}}\n"
            )
        })
        .collect::<String>();
    c.bench_function("analysis/full_80_dynamic_macros", |b| {
        b.iter(|| {
            black_box(analyze_rust_document(
                black_box(&dynamic),
                ClassMode::PERMISSIVE,
            ));
            release_span_memory();
        })
    });
    c.bench_function("analysis/edit_one_of_80_macros", |b| {
        b.iter(|| {
            black_box(update_rust_document(
                black_box(&edited),
                ClassMode::PERMISSIVE,
                &source,
                &previous,
            ));
            release_span_memory();
        })
    });
    let large = (0..5000)
        .map(|i| {
            if i % 100 == 0 {
                format!("fn view_{i}(){{rsx!{{<div class=\"flex gap-4 bg-blue-500\"/>}};}}\n")
            } else {
                format!("// ordinary Rust source line {i}\n")
            }
        })
        .collect::<String>();
    let large_previous = analyze_rust_document(&large, ClassMode::PERMISSIVE);
    let large_edit = large.replacen("gap-4", "gap-8", 1);
    release_span_memory();
    for incremental in [false, true] {
        c.bench_function(
            if incremental {
                "analysis/edit_5000_lines"
            } else {
                "analysis/full_5000_lines"
            },
            |b| {
                b.iter(|| {
                    black_box(if incremental {
                        update_rust_document(
                            black_box(&large_edit),
                            ClassMode::PERMISSIVE,
                            &large,
                            &large_previous,
                        )
                    } else {
                        analyze_rust_document(black_box(&large_edit), ClassMode::PERMISSIVE)
                    });
                    release_span_memory();
                })
            },
        );
    }
}

criterion_group!(benches, analysis);
criterion_main!(benches);
