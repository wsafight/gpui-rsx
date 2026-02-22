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

#[allow(dead_code)] // benchmark 中并非所有方法都被使用
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
    fn overflow_scroll(self) -> Self {
        self
    }
    fn text_xs(self) -> Self {
        self
    }
    fn rounded_sm(self) -> Self {
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

// Styled trait - 动态 class 代码生成需要此 trait
// 必须与 generate_common_class_matches 生成的所有方法调用保持一致。
#[allow(dead_code)] // benchmark 中并非所有 trait 方法都被使用
trait Styled: Sized {
    // --- flex ---
    fn flex(self) -> Self;
    fn flex_col(self) -> Self;
    fn flex_col_reverse(self) -> Self;
    fn flex_row(self) -> Self;
    fn flex_row_reverse(self) -> Self;
    fn flex_1(self) -> Self;
    fn flex_auto(self) -> Self;
    fn flex_initial(self) -> Self;
    fn flex_none(self) -> Self;
    fn flex_wrap(self) -> Self;
    fn flex_wrap_reverse(self) -> Self;
    fn flex_nowrap(self) -> Self;
    fn flex_shrink_0(self) -> Self;
    // --- layout ---
    fn block(self) -> Self;
    fn grid(self) -> Self;
    fn hidden(self) -> Self;
    // --- alignment ---
    fn items_center(self) -> Self;
    fn items_start(self) -> Self;
    fn items_end(self) -> Self;
    fn items_baseline(self) -> Self;
    fn items_stretch(self) -> Self;
    fn justify_center(self) -> Self;
    fn justify_between(self) -> Self;
    fn justify_start(self) -> Self;
    fn justify_end(self) -> Self;
    fn justify_around(self) -> Self;
    fn justify_evenly(self) -> Self;
    fn content_center(self) -> Self;
    fn content_start(self) -> Self;
    fn content_end(self) -> Self;
    fn content_between(self) -> Self;
    fn content_around(self) -> Self;
    fn content_evenly(self) -> Self;
    fn content_stretch(self) -> Self;
    // --- spacing ---
    fn gap(self, val: f32) -> Self;
    fn gap_x(self, val: f32) -> Self;
    fn gap_y(self, val: f32) -> Self;
    fn gap_2(self) -> Self;
    fn gap_4(self) -> Self;
    fn gap_6(self) -> Self;
    fn p(self, val: f32) -> Self;
    fn px(self, val: f32) -> Self;
    fn py(self, val: f32) -> Self;
    fn pt(self, val: f32) -> Self;
    fn pb(self, val: f32) -> Self;
    fn pl(self, val: f32) -> Self;
    fn pr(self, val: f32) -> Self;
    fn p_2(self) -> Self;
    fn p_4(self) -> Self;
    fn m(self, val: f32) -> Self;
    fn mx(self, val: f32) -> Self;
    fn my(self, val: f32) -> Self;
    fn mt(self, val: f32) -> Self;
    fn mb(self, val: f32) -> Self;
    fn ml(self, val: f32) -> Self;
    fn mr(self, val: f32) -> Self;
    fn m_2(self) -> Self;
    fn m_4(self) -> Self;
    // --- sizing ---
    fn w_full(self) -> Self;
    fn h_full(self) -> Self;
    fn size_full(self) -> Self;
    fn size(self, val: f32) -> Self;
    fn w(self, val: f32) -> Self;
    fn h(self, val: f32) -> Self;
    fn min_w(self, val: f32) -> Self;
    fn max_w(self, val: f32) -> Self;
    fn min_h(self, val: f32) -> Self;
    fn max_h(self, val: f32) -> Self;
    // --- text size ---
    fn text_xs(self) -> Self;
    fn text_sm(self) -> Self;
    fn text_base(self) -> Self;
    fn text_lg(self) -> Self;
    fn text_xl(self) -> Self;
    fn text_2xl(self) -> Self;
    fn text_3xl(self) -> Self;
    // --- text alignment ---
    fn text_left(self) -> Self;
    fn text_center(self) -> Self;
    fn text_right(self) -> Self;
    // --- text decoration ---
    fn truncate(self) -> Self;
    fn text_ellipsis(self) -> Self;
    fn italic(self) -> Self;
    fn not_italic(self) -> Self;
    fn underline(self) -> Self;
    fn line_through(self) -> Self;
    // --- font ---
    fn font_bold(self) -> Self;
    // --- border ---
    fn border_1(self) -> Self;
    fn border_2(self) -> Self;
    fn border_dashed(self) -> Self;
    fn border_t(self) -> Self;
    fn border_b(self) -> Self;
    fn border_l(self) -> Self;
    fn border_r(self) -> Self;
    fn rounded_none(self) -> Self;
    fn rounded_sm(self) -> Self;
    fn rounded_md(self) -> Self;
    fn rounded_lg(self) -> Self;
    fn rounded_xl(self) -> Self;
    fn rounded_full(self) -> Self;
    // --- misc ---
    fn cursor_pointer(self) -> Self;
    fn cursor_default(self) -> Self;
    fn cursor_text(self) -> Self;
    fn overflow_hidden(self) -> Self;
    fn overflow_scroll(self) -> Self;
    fn overflow_visible(self) -> Self;
    fn absolute(self) -> Self;
    fn relative(self) -> Self;
    // --- shadow ---
    fn shadow_sm(self) -> Self;
    fn shadow_md(self) -> Self;
    fn shadow_lg(self) -> Self;
    // --- color / opacity / z ---
    fn bg(self, color: u32) -> Self;
    fn text_color(self, color: u32) -> Self;
    fn border_color(self, color: u32) -> Self;
    fn opacity(self, val: f32) -> Self;
    fn z_index(self, val: i32) -> Self;
    // --- grid ---
    fn grid_cols(self, v: u16) -> Self;
    fn grid_rows(self, v: u16) -> Self;
    fn col_span(self, v: u16) -> Self;
    fn col_start(self, v: i16) -> Self;
    fn col_end(self, v: i16) -> Self;
    fn row_span(self, v: u16) -> Self;
    fn row_start(self, v: i16) -> Self;
    fn row_end(self, v: i16) -> Self;
}

impl Styled for MockElement {
    // --- flex ---
    fn flex(self) -> Self {
        self
    }
    fn flex_col(self) -> Self {
        self
    }
    fn flex_col_reverse(self) -> Self {
        self
    }
    fn flex_row(self) -> Self {
        self
    }
    fn flex_row_reverse(self) -> Self {
        self
    }
    fn flex_1(self) -> Self {
        self
    }
    fn flex_auto(self) -> Self {
        self
    }
    fn flex_initial(self) -> Self {
        self
    }
    fn flex_none(self) -> Self {
        self
    }
    fn flex_wrap(self) -> Self {
        self
    }
    fn flex_wrap_reverse(self) -> Self {
        self
    }
    fn flex_nowrap(self) -> Self {
        self
    }
    fn flex_shrink_0(self) -> Self {
        self
    }
    // --- layout ---
    fn block(self) -> Self {
        self
    }
    fn grid(self) -> Self {
        self
    }
    fn hidden(self) -> Self {
        self
    }
    // --- alignment ---
    fn items_center(self) -> Self {
        self
    }
    fn items_start(self) -> Self {
        self
    }
    fn items_end(self) -> Self {
        self
    }
    fn items_baseline(self) -> Self {
        self
    }
    fn items_stretch(self) -> Self {
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
    fn justify_around(self) -> Self {
        self
    }
    fn justify_evenly(self) -> Self {
        self
    }
    fn content_center(self) -> Self {
        self
    }
    fn content_start(self) -> Self {
        self
    }
    fn content_end(self) -> Self {
        self
    }
    fn content_between(self) -> Self {
        self
    }
    fn content_around(self) -> Self {
        self
    }
    fn content_evenly(self) -> Self {
        self
    }
    fn content_stretch(self) -> Self {
        self
    }
    // --- spacing ---
    fn gap(self, _: f32) -> Self {
        self
    }
    fn gap_x(self, _: f32) -> Self {
        self
    }
    fn gap_y(self, _: f32) -> Self {
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
    fn p(self, _: f32) -> Self {
        self
    }
    fn px(self, _: f32) -> Self {
        self
    }
    fn py(self, _: f32) -> Self {
        self
    }
    fn pt(self, _: f32) -> Self {
        self
    }
    fn pb(self, _: f32) -> Self {
        self
    }
    fn pl(self, _: f32) -> Self {
        self
    }
    fn pr(self, _: f32) -> Self {
        self
    }
    fn p_2(self) -> Self {
        self
    }
    fn p_4(self) -> Self {
        self
    }
    fn m(self, _: f32) -> Self {
        self
    }
    fn mx(self, _: f32) -> Self {
        self
    }
    fn my(self, _: f32) -> Self {
        self
    }
    fn mt(self, _: f32) -> Self {
        self
    }
    fn mb(self, _: f32) -> Self {
        self
    }
    fn ml(self, _: f32) -> Self {
        self
    }
    fn mr(self, _: f32) -> Self {
        self
    }
    fn m_2(self) -> Self {
        self
    }
    fn m_4(self) -> Self {
        self
    }
    // --- sizing ---
    fn w_full(self) -> Self {
        self
    }
    fn h_full(self) -> Self {
        self
    }
    fn size_full(self) -> Self {
        self
    }
    fn size(self, _: f32) -> Self {
        self
    }
    fn w(self, _: f32) -> Self {
        self
    }
    fn h(self, _: f32) -> Self {
        self
    }
    fn min_w(self, _: f32) -> Self {
        self
    }
    fn max_w(self, _: f32) -> Self {
        self
    }
    fn min_h(self, _: f32) -> Self {
        self
    }
    fn max_h(self, _: f32) -> Self {
        self
    }
    // --- text size ---
    fn text_xs(self) -> Self {
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
    // --- text alignment ---
    fn text_left(self) -> Self {
        self
    }
    fn text_center(self) -> Self {
        self
    }
    fn text_right(self) -> Self {
        self
    }
    // --- text decoration ---
    fn truncate(self) -> Self {
        self
    }
    fn text_ellipsis(self) -> Self {
        self
    }
    fn italic(self) -> Self {
        self
    }
    fn not_italic(self) -> Self {
        self
    }
    fn underline(self) -> Self {
        self
    }
    fn line_through(self) -> Self {
        self
    }
    // --- font ---
    fn font_bold(self) -> Self {
        self
    }
    // --- border ---
    fn border_1(self) -> Self {
        self
    }
    fn border_2(self) -> Self {
        self
    }
    fn border_dashed(self) -> Self {
        self
    }
    fn border_t(self) -> Self {
        self
    }
    fn border_b(self) -> Self {
        self
    }
    fn border_l(self) -> Self {
        self
    }
    fn border_r(self) -> Self {
        self
    }
    fn rounded_none(self) -> Self {
        self
    }
    fn rounded_sm(self) -> Self {
        self
    }
    fn rounded_md(self) -> Self {
        self
    }
    fn rounded_lg(self) -> Self {
        self
    }
    fn rounded_xl(self) -> Self {
        self
    }
    fn rounded_full(self) -> Self {
        self
    }
    // --- misc ---
    fn cursor_pointer(self) -> Self {
        self
    }
    fn cursor_default(self) -> Self {
        self
    }
    fn cursor_text(self) -> Self {
        self
    }
    fn overflow_hidden(self) -> Self {
        self
    }
    fn overflow_scroll(self) -> Self {
        self
    }
    fn overflow_visible(self) -> Self {
        self
    }
    fn absolute(self) -> Self {
        self
    }
    fn relative(self) -> Self {
        self
    }
    // --- shadow ---
    fn shadow_sm(self) -> Self {
        self
    }
    fn shadow_md(self) -> Self {
        self
    }
    fn shadow_lg(self) -> Self {
        self
    }
    // --- color / opacity / z ---
    fn bg(self, _: u32) -> Self {
        self
    }
    fn text_color(self, _: u32) -> Self {
        self
    }
    fn border_color(self, _: u32) -> Self {
        self
    }
    fn opacity(self, _: f32) -> Self {
        self
    }
    fn z_index(self, _: i32) -> Self {
        self
    }
    // --- grid ---
    fn grid_cols(self, _: u16) -> Self {
        self
    }
    fn grid_rows(self, _: u16) -> Self {
        self
    }
    fn col_span(self, _: u16) -> Self {
        self
    }
    fn col_start(self, _: i16) -> Self {
        self
    }
    fn col_end(self, _: i16) -> Self {
        self
    }
    fn row_span(self, _: u16) -> Self {
        self
    }
    fn row_start(self, _: i16) -> Self {
        self
    }
    fn row_end(self, _: i16) -> Self {
        self
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
            let count_str = black_box(count).to_string();
            let _el = rsx! {
                <div>{"Count: "}{count_str.as_str()}</div>
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
