//! Class 性能基准测试
//!
//! 对比静态 class 和动态 class 的运行时性能。
//!
//! 运行：`cargo bench`

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use gpui_rsx::rsx;

// Mock GPUI 类型（与 tests/macro_tests.rs 相同）
#[derive(Debug)]
struct MockElement;

fn div() -> MockElement {
    MockElement
}

fn rgb(_hex: u32) -> u32 {
    0
}

fn px(_val: f32) -> f32 {
    0.0
}

impl MockElement {
    fn id<T>(self, _: T) -> Self {
        self
    }
    fn flex(self) -> Self {
        self
    }
    fn flex_col(self) -> Self {
        self
    }
    fn flex_row(self) -> Self {
        self
    }
    fn flex_1(self) -> Self {
        self
    }
    fn flex_wrap(self) -> Self {
        self
    }
    fn gap<T>(self, _: T) -> Self {
        self
    }
    fn gap_2(self) -> Self {
        self
    }
    fn gap_4(self) -> Self {
        self
    }
    fn gap_6(self) -> Self {
        self
    }
    fn p<T>(self, _: T) -> Self {
        self
    }
    fn p_2(self) -> Self {
        self
    }
    fn p_4(self) -> Self {
        self
    }
    fn bg<T>(self, _: T) -> Self {
        self
    }
    fn text_color<T>(self, _: T) -> Self {
        self
    }
    fn px<T>(self, _: T) -> Self {
        self
    }
    fn px_2(self) -> Self {
        self
    }
    fn px_4(self) -> Self {
        self
    }
    fn py<T>(self, _: T) -> Self {
        self
    }
    fn py_2(self) -> Self {
        self
    }
    fn py_4(self) -> Self {
        self
    }
    fn m<T>(self, _: T) -> Self {
        self
    }
    fn m_2(self) -> Self {
        self
    }
    fn m_4(self) -> Self {
        self
    }
    fn rounded_md(self) -> Self {
        self
    }
    fn rounded_lg(self) -> Self {
        self
    }
    fn rounded_full(self) -> Self {
        self
    }
    fn items_center(self) -> Self {
        self
    }
    fn items_start(self) -> Self {
        self
    }
    fn items_end(self) -> Self {
        self
    }
    fn justify_center(self) -> Self {
        self
    }
    fn justify_between(self) -> Self {
        self
    }
    fn justify_start(self) -> Self {
        self
    }
    fn justify_end(self) -> Self {
        self
    }
    fn w_full(self) -> Self {
        self
    }
    fn h_full(self) -> Self {
        self
    }
    fn size_full(self) -> Self {
        self
    }
    fn text_sm(self) -> Self {
        self
    }
    fn text_base(self) -> Self {
        self
    }
    fn text_lg(self) -> Self {
        self
    }
    fn text_xl(self) -> Self {
        self
    }
    fn text_2xl(self) -> Self {
        self
    }
    fn text_3xl(self) -> Self {
        self
    }
    fn font_bold(self) -> Self {
        self
    }
    fn border(self) -> Self {
        self
    }
    fn border_1(self) -> Self {
        self
    }
    fn border_2(self) -> Self {
        self
    }
    fn cursor_pointer(self) -> Self {
        self
    }
    fn overflow_hidden(self) -> Self {
        self
    }
    fn absolute(self) -> Self {
        self
    }
    fn relative(self) -> Self {
        self
    }
    fn child<T>(self, _: T) -> Self {
        self
    }
    fn children<I: IntoIterator>(self, _: I) -> Self {
        self
    }
    fn map<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }
    fn when<F>(self, _condition: bool, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }
}

/// 基准测试：静态 class（编译期优化）
fn bench_static_class(c: &mut Criterion) {
    c.bench_function("static_class_simple", |b| {
        b.iter(|| {
            let _el = rsx! { <div class="flex gap-4" /> };
        })
    });

    c.bench_function("static_class_complex", |b| {
        b.iter(|| {
            let _el = rsx! {
                <div class="flex flex-col gap-4 p-4 bg-blue-500 text-white rounded-md" />
            };
        })
    });
}

/// 基准测试：动态 class（运行时解析）
fn bench_dynamic_class(c: &mut Criterion) {
    let simple_classes = "flex gap-4";
    c.bench_function("dynamic_class_simple", |b| {
        b.iter(|| {
            let _el = rsx! { <div class={black_box(simple_classes)} /> };
        })
    });

    let complex_classes = "flex flex-col gap-4 p-4 bg-blue-500 text-white rounded-md";
    c.bench_function("dynamic_class_complex", |b| {
        b.iter(|| {
            let _el = rsx! { <div class={black_box(complex_classes)} /> };
        })
    });
}

/// 基准测试：条件 class
fn bench_conditional_class(c: &mut Criterion) {
    let mut group = c.benchmark_group("conditional_class");

    // 静态条件
    group.bench_function("static_if_else", |b| {
        let is_active = black_box(true);
        b.iter(|| {
            let _el = rsx! {
                <div class={if is_active { "bg-blue-500" } else { "bg-gray-200" }} />
            };
        })
    });

    // when 属性（推荐）
    group.bench_function("when_attribute", |b| {
        let is_active = black_box(true);
        b.iter(|| {
            let _el = rsx! {
                <div
                    bg={rgb(0xe5e7eb)}
                    when={(is_active, |el| el.bg(rgb(0x3b82f6)))}
                />
            };
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
                        let _el = rsx! {
                            <div class="flex">
                                <div class="gap-4">{"Content"}</div>
                            </div>
                        };
                    }
                    5 => {
                        let _el = rsx! {
                            <div class="flex">
                                <div class="flex-col">
                                    <div class="gap-4">
                                        <div class="p-4">
                                            <div>{"Content"}</div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        };
                    }
                    10 => {
                        let _el = rsx! {
                            <div class="flex">
                                <div><div><div><div><div>
                                    <div><div><div><div>
                                        <div>{"Content"}</div>
                                    </div></div></div></div>
                                </div></div></div></div></div>
                            </div>
                        };
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
            let _el = rsx! {
                <div>
                    {for item in black_box(&items) {
                        <div>{*item}</div>
                    }}
                </div>
            };
        })
    });

    group.bench_function("iterator_map", |b| {
        b.iter(|| {
            let _el = rsx! {
                <div>
                    {black_box(&items).iter().map(|item| {
                        rsx! { <div>{*item}</div> }
                    }).collect::<Vec<_>>()}
                </div>
            };
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
            let _el = rsx! {
                <div>{format!("Count: {}", black_box(count))}</div>
            };
        })
    });

    group.bench_function("separate_children", |b| {
        b.iter(|| {
            let _el = rsx! {
                <div>{"Count: "}{black_box(count)}</div>
            };
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
