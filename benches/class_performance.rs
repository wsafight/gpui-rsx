//! Class 性能基准测试
//!
//! 对比静态 class 和动态 class 的运行时性能。
//!
//! 运行：`cargo bench`

use gpui_rsx::rsx;
use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

mod support;

use support::*;

/// 基准测试：静态 class（编译期优化）
fn bench_static_class(c: &mut Criterion) {
    c.bench_function("static_class_simple", |b| {
        b.iter(|| black_box(rsx! { <div class="flex gap-4" /> }))
    });

    c.bench_function("static_class_complex", |b| {
        b.iter(|| {
            black_box(rsx! {
                <div class="flex flex-col gap-4 p-4 bg-blue-500 text-white rounded-md" />
            })
        })
    });
}

/// 基准测试：动态 class（运行时解析）
fn bench_dynamic_class(c: &mut Criterion) {
    let simple_classes = "flex gap-4";
    c.bench_function("dynamic_class_simple", |b| {
        b.iter(|| black_box(rsx! { <div class={black_box(simple_classes)} /> }))
    });

    let complex_classes = "flex flex-col gap-4 p-4 bg-blue-500 text-white rounded-md";
    c.bench_function("dynamic_class_complex", |b| {
        b.iter(|| black_box(rsx! { <div class={black_box(complex_classes)} /> }))
    });

    let fast_spacing = "gap-4";
    c.bench_function("dynamic_spacing_fast_path", |b| {
        b.iter(|| black_box(rsx! { <div class={black_box(fast_spacing)} /> }))
    });

    let fallback_spacing = "gap-7";
    c.bench_function("dynamic_spacing_numeric_fallback", |b| {
        b.iter(|| black_box(rsx! { <div class={black_box(fallback_spacing)} /> }))
    });
}

/// 基准测试：条件 class
fn bench_conditional_class(c: &mut Criterion) {
    let mut group = c.benchmark_group("conditional_class");

    // 静态条件
    group.bench_function("static_if_else", |b| {
        let is_active = black_box(true);
        b.iter(|| {
            black_box(rsx! {
                <div class={if is_active { "bg-blue-500" } else { "bg-gray-200" }} />
            })
        })
    });

    // when 属性（推荐）
    group.bench_function("when_attribute", |b| {
        let is_active = black_box(true);
        b.iter(|| {
            black_box(rsx! {
                <div
                    bg={rgb(0xe5e7eb)}
                    when={(is_active, |el| el.bg(rgb(0x3b82f6)))}
                />
            })
        })
    });

    group.finish();
}

/// 基准测试：嵌套元素
fn bench_nested_elements(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_elements");

    for depth in [2, 5, 10].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, &depth| {
            b.iter(|| {
                // 动态生成不同深度的嵌套
                match depth {
                    2 => {
                        black_box(rsx! {
                            <div class="flex">
                                <div class="gap-4">{"Content"}</div>
                            </div>
                        });
                    }
                    5 => {
                        black_box(rsx! {
                            <div class="flex">
                                <div class="flex-col">
                                    <div class="gap-4">
                                        <div class="p-4">
                                            <div>{"Content"}</div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        });
                    }
                    10 => {
                        black_box(rsx! {
                            <div class="flex">
                                <div><div><div><div><div>
                                    <div><div><div><div>
                                        <div>{"Content"}</div>
                                    </div></div></div></div>
                                </div></div></div></div></div>
                            </div>
                        });
                    }
                    _ => unreachable!(),
                }
            })
        });
    }

    group.finish();
}

/// 基准测试：循环渲染
fn bench_loop_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("loop_rendering");

    let items = vec!["Item 1", "Item 2", "Item 3", "Item 4", "Item 5"];

    group.bench_function("for_loop", |b| {
        b.iter(|| {
            black_box(rsx! {
                <div>
                    {for item in black_box(&items) {
                        <div>{*item}</div>
                    }}
                </div>
            })
        })
    });

    group.bench_function("iterator_map", |b| {
        b.iter(|| {
            black_box(rsx! {
                <div>
                    {black_box(&items).iter().map(|item| {
                        rsx! { <div>{*item}</div> }
                    }).collect::<Vec<_>>()}
                </div>
            })
        })
    });

    group.finish();
}

/// 基准测试：字符串分配
fn bench_string_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_allocation");

    let count = 42;

    group.bench_function("format_macro", |b| {
        b.iter(|| {
            black_box(rsx! {
                <div>{format!("Count: {}", black_box(count))}</div>
            })
        })
    });

    group.bench_function("separate_children", |b| {
        b.iter(|| {
            let count_str = black_box(count).to_string();
            black_box(rsx! {
                <div>{"Count: "}{count_str.as_str()}</div>
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_static_class,
    bench_dynamic_class,
    bench_conditional_class,
    bench_nested_elements,
    bench_loop_rendering,
    bench_string_allocation,
);

criterion_main!(benches);
