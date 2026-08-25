//! Class 性能基准测试
//!
//! 对比静态 class 和动态 class 的运行时性能。
//!
//! 运行：`cargo bench`

use gpui_rsx::rsx;
use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

// Mock GPUI 类型（与 tests/macro_tests.rs 相同）
#[derive(Debug, Default)]
struct MockElement {
    checksum: u64,
}

#[derive(Clone, Copy, Debug)]
struct FontWeight;

impl FontWeight {
    const THIN: FontWeight = FontWeight;
    const EXTRA_LIGHT: FontWeight = FontWeight;
    const LIGHT: FontWeight = FontWeight;
    const NORMAL: FontWeight = FontWeight;
    const MEDIUM: FontWeight = FontWeight;
    const SEMIBOLD: FontWeight = FontWeight;
    const BOLD: FontWeight = FontWeight;
    const EXTRA_BOLD: FontWeight = FontWeight;
    const BLACK: FontWeight = FontWeight;
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

type JustifyContent = AlignContent;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
enum AlignContent {
    Stretch,
    SpaceEvenly,
}

#[allow(dead_code)]
#[derive(Default, Debug)]
struct StyleRefinement {
    align_items: Option<AlignItems>,
    align_self: Option<AlignItems>,
    justify_content: Option<JustifyContent>,
    align_content: Option<AlignContent>,
    aspect_ratio: Option<f32>,
    flex_grow: Option<f32>,
}

fn div() -> MockElement {
    MockElement {
        checksum: black_box(0),
    }
}

fn rgb(hex: u32) -> u32 {
    hex
}
fn rgba(hex: u32) -> u32 {
    hex
}

fn px(val: f32) -> f32 {
    val
}
fn rems(val: f32) -> f32 {
    val
}
fn relative(val: f32) -> f32 {
    val
}

macro_rules! mock_no_arg_methods {
    ($($name:ident),* $(,)?) => {
        $(fn $name(self) -> Self { self })*
    };
}

macro_rules! mock_value_methods {
    ($($name:ident),* $(,)?) => {
        $(fn $name<T>(self, _: T) -> Self { self })*
    };
}

#[allow(dead_code)] // benchmark 中并非所有方法都被使用
impl MockElement {
    fn touch(mut self, operation: u64) -> Self {
        self.checksum = self.checksum.wrapping_mul(16777619) ^ operation;
        self
    }

    fn id<T>(self, _: T) -> Self {
        self.touch(1)
    }
    fn flex(self) -> Self {
        self.touch(2)
    }
    fn flex_col(self) -> Self {
        self.touch(3)
    }
    mock_no_arg_methods!(flex_row, flex_1, flex_wrap);

    fn gap<T>(self, _: T) -> Self {
        self.touch(4)
    }
    fn gap_4(self) -> Self {
        self.touch(5)
    }
    mock_no_arg_methods!(gap_2, gap_6);

    fn p<T>(self, _: T) -> Self {
        self.touch(6)
    }
    fn p_4(self) -> Self {
        self.touch(7)
    }
    mock_no_arg_methods!(p_2);

    fn bg<T>(self, _: T) -> Self {
        self.touch(8)
    }
    fn text_color<T>(self, _: T) -> Self {
        self.touch(9)
    }
    mock_value_methods!(px, py, m, font_weight);
    mock_no_arg_methods!(px_2, px_4, py_2, py_4, m_2, m_4);

    fn rounded_md(self) -> Self {
        self.touch(10)
    }
    mock_no_arg_methods!(
        rounded_lg,
        rounded_full,
        items_center,
        items_start,
        items_end,
        justify_center,
        justify_between,
        justify_start,
        justify_end,
        w_full,
        h_full,
        size_full,
        text_xs,
        text_sm,
        text_base,
        text_lg,
        text_xl,
        text_2xl,
        text_3xl,
        font_bold,
        border,
        border_1,
        border_2,
        cursor_pointer,
        overflow_hidden,
        overflow_scroll,
        rounded_sm,
        absolute,
        relative,
        debug,
    );

    fn child<T>(self, _: T) -> Self {
        self.touch(11)
    }
    fn children<I: IntoIterator>(self, _: I) -> Self {
        self.touch(12)
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
    fn style(&mut self) -> &mut StyleRefinement {
        Box::leak(Box::new(StyleRefinement::default()))
    }
}

// Styled trait - 动态 class 代码生成需要此 trait
// 必须与 generate_common_class_matches 生成的所有方法调用保持一致。
#[allow(dead_code)] // benchmark 中并非所有 trait 方法都被使用
trait Styled: Sized {
    fn style(&mut self) -> &mut StyleRefinement;

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
    fn flex_grow(self) -> Self;
    fn flex_wrap(self) -> Self;
    fn flex_wrap_reverse(self) -> Self;
    fn flex_nowrap(self) -> Self;
    fn flex_grow_0(self) -> Self;
    fn flex_grow_1(self) -> Self;
    fn flex_shrink(self) -> Self;
    fn flex_shrink_0(self) -> Self;
    fn flex_shrink_1(self) -> Self;
    // --- layout ---
    fn block(self) -> Self;
    fn grid(self) -> Self;
    fn hidden(self) -> Self;
    fn aspect_square(self) -> Self;
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
    fn content_normal(self) -> Self;
    fn content_center(self) -> Self;
    fn content_start(self) -> Self;
    fn content_end(self) -> Self;
    fn content_between(self) -> Self;
    fn content_around(self) -> Self;
    fn content_evenly(self) -> Self;
    fn content_stretch(self) -> Self;
    fn self_start(self) -> Self;
    fn self_end(self) -> Self;
    fn self_flex_start(self) -> Self;
    fn self_flex_end(self) -> Self;
    fn self_center(self) -> Self;
    fn self_baseline(self) -> Self;
    fn self_stretch(self) -> Self;
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
    fn w_px(self) -> Self;
    fn w_auto(self) -> Self;
    fn w_1_2(self) -> Self;
    fn w_1_3(self) -> Self;
    fn h_full(self) -> Self;
    fn h_px(self) -> Self;
    fn h_auto(self) -> Self;
    fn h_1_2(self) -> Self;
    fn h_1_3(self) -> Self;
    fn size_full(self) -> Self;
    fn size_px(self) -> Self;
    fn size_1_2(self) -> Self;
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
    fn whitespace_normal(self) -> Self;
    fn whitespace_nowrap(self) -> Self;
    fn truncate(self) -> Self;
    fn text_ellipsis(self) -> Self;
    fn text_ellipsis_start(self) -> Self;
    fn text_ellipsis_middle(self) -> Self;
    fn line_clamp(self, lines: usize) -> Self;
    fn italic(self) -> Self;
    fn not_italic(self) -> Self;
    fn underline(self) -> Self;
    fn line_through(self) -> Self;
    fn text_decoration_none(self) -> Self;
    fn text_decoration_solid(self) -> Self;
    fn text_decoration_wavy(self) -> Self;
    fn text_decoration_0(self) -> Self;
    fn text_decoration_1(self) -> Self;
    fn text_decoration_2(self) -> Self;
    fn text_decoration_4(self) -> Self;
    fn text_decoration_8(self) -> Self;
    // --- font ---
    fn font_bold(self) -> Self;
    fn font_weight<T>(self, weight: T) -> Self;
    // --- border ---
    fn border_1(self) -> Self;
    fn border_2(self) -> Self;
    fn border_dashed(self) -> Self;
    fn border_t_1(self) -> Self;
    fn border_b_1(self) -> Self;
    fn border_l_1(self) -> Self;
    fn border_r_1(self) -> Self;
    fn border_x_1(self) -> Self;
    fn border_y_1(self) -> Self;
    fn border_t_2(self) -> Self;
    fn border_b_2(self) -> Self;
    fn border_l_2(self) -> Self;
    fn border_r_2(self) -> Self;
    fn border_x_2(self) -> Self;
    fn border_y_2(self) -> Self;
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
    fn overflow_x_hidden(self) -> Self;
    fn overflow_y_hidden(self) -> Self;
    fn absolute(self) -> Self;
    fn relative(self) -> Self;
    // --- shadow ---
    fn shadow_none(self) -> Self;
    fn shadow_2xs(self) -> Self;
    fn shadow_xs(self) -> Self;
    fn shadow_sm(self) -> Self;
    fn shadow_md(self) -> Self;
    fn shadow_lg(self) -> Self;
    fn shadow_xl(self) -> Self;
    fn shadow_2xl(self) -> Self;
    // --- cursor extra ---
    fn cursor_move(self) -> Self;
    fn cursor_not_allowed(self) -> Self;
    fn cursor_context_menu(self) -> Self;
    fn cursor_crosshair(self) -> Self;
    fn cursor_vertical_text(self) -> Self;
    fn cursor_alias(self) -> Self;
    fn cursor_copy(self) -> Self;
    fn cursor_no_drop(self) -> Self;
    fn cursor_grab(self) -> Self;
    fn cursor_grabbing(self) -> Self;
    fn cursor_ew_resize(self) -> Self;
    fn cursor_ns_resize(self) -> Self;
    fn cursor_nesw_resize(self) -> Self;
    fn cursor_nwse_resize(self) -> Self;
    fn cursor_col_resize(self) -> Self;
    fn cursor_row_resize(self) -> Self;
    fn cursor_n_resize(self) -> Self;
    fn cursor_e_resize(self) -> Self;
    fn cursor_s_resize(self) -> Self;
    fn cursor_w_resize(self) -> Self;
    fn debug(self) -> Self;
    // --- color / opacity ---
    fn bg<T>(self, color: T) -> Self;
    fn text_color<T>(self, color: T) -> Self;
    fn border_color<T>(self, color: T) -> Self;
    fn opacity(self, val: f32) -> Self;
    // --- grid ---
    fn grid_cols(self, v: u16) -> Self;
    fn grid_rows(self, v: u16) -> Self;
    fn grid_cols_min_content(self, v: u16) -> Self;
    fn grid_cols_max_content(self, v: u16) -> Self;
    fn grid_rows_min_content(self, v: u16) -> Self;
    fn grid_rows_max_content(self, v: u16) -> Self;
    fn col_span(self, v: u16) -> Self;
    fn col_span_full(self) -> Self;
    fn col_start(self, v: i16) -> Self;
    fn col_start_auto(self) -> Self;
    fn col_end(self, v: i16) -> Self;
    fn col_end_auto(self) -> Self;
    fn row_span(self, v: u16) -> Self;
    fn row_span_full(self) -> Self;
    fn row_start(self, v: i16) -> Self;
    fn row_start_auto(self) -> Self;
    fn row_end(self, v: i16) -> Self;
    fn row_end_auto(self) -> Self;
}

impl Styled for MockElement {
    fn style(&mut self) -> &mut StyleRefinement {
        Box::leak(Box::new(StyleRefinement::default()))
    }

    // --- flex ---
    fn flex(self) -> Self {
        self.touch(2)
    }
    fn flex_col(self) -> Self {
        self.touch(3)
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
    fn flex_grow(self) -> Self {
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
    fn flex_grow_0(self) -> Self {
        self
    }
    fn flex_grow_1(self) -> Self {
        self
    }
    fn flex_shrink(self) -> Self {
        self
    }
    fn flex_shrink_0(self) -> Self {
        self
    }
    fn flex_shrink_1(self) -> Self {
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
    fn aspect_square(self) -> Self {
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
    fn content_normal(self) -> Self {
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
    fn self_start(self) -> Self {
        self
    }
    fn self_end(self) -> Self {
        self
    }
    fn self_flex_start(self) -> Self {
        self
    }
    fn self_flex_end(self) -> Self {
        self
    }
    fn self_center(self) -> Self {
        self
    }
    fn self_baseline(self) -> Self {
        self
    }
    fn self_stretch(self) -> Self {
        self
    }
    // --- spacing ---
    fn gap(self, _: f32) -> Self {
        self.touch(4)
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
        self.touch(5)
    }
    fn gap_6(self) -> Self {
        self
    }
    fn p(self, _: f32) -> Self {
        self.touch(6)
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
        self.touch(7)
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
    fn w_px(self) -> Self {
        self
    }
    fn w_auto(self) -> Self {
        self
    }
    fn w_1_2(self) -> Self {
        self
    }
    fn w_1_3(self) -> Self {
        self
    }
    fn h_full(self) -> Self {
        self
    }
    fn h_px(self) -> Self {
        self
    }
    fn h_auto(self) -> Self {
        self
    }
    fn h_1_2(self) -> Self {
        self
    }
    fn h_1_3(self) -> Self {
        self
    }
    fn size_full(self) -> Self {
        self
    }
    fn size_px(self) -> Self {
        self
    }
    fn size_1_2(self) -> Self {
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
    fn whitespace_normal(self) -> Self {
        self
    }
    fn whitespace_nowrap(self) -> Self {
        self
    }
    fn truncate(self) -> Self {
        self
    }
    fn text_ellipsis(self) -> Self {
        self
    }
    fn text_ellipsis_start(self) -> Self {
        self
    }
    fn text_ellipsis_middle(self) -> Self {
        self
    }
    fn line_clamp(self, _: usize) -> Self {
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
    fn text_decoration_none(self) -> Self {
        self
    }
    fn text_decoration_solid(self) -> Self {
        self
    }
    fn text_decoration_wavy(self) -> Self {
        self
    }
    fn text_decoration_0(self) -> Self {
        self
    }
    fn text_decoration_1(self) -> Self {
        self
    }
    fn text_decoration_2(self) -> Self {
        self
    }
    fn text_decoration_4(self) -> Self {
        self
    }
    fn text_decoration_8(self) -> Self {
        self
    }
    // --- font ---
    fn font_bold(self) -> Self {
        self
    }
    fn font_weight<T>(self, _: T) -> Self {
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
    fn border_t_1(self) -> Self {
        self
    }
    fn border_b_1(self) -> Self {
        self
    }
    fn border_l_1(self) -> Self {
        self
    }
    fn border_r_1(self) -> Self {
        self
    }
    fn border_x_1(self) -> Self {
        self
    }
    fn border_y_1(self) -> Self {
        self
    }
    fn border_t_2(self) -> Self {
        self
    }
    fn border_b_2(self) -> Self {
        self
    }
    fn border_l_2(self) -> Self {
        self
    }
    fn border_r_2(self) -> Self {
        self
    }
    fn border_x_2(self) -> Self {
        self
    }
    fn border_y_2(self) -> Self {
        self
    }
    fn rounded_none(self) -> Self {
        self
    }
    fn rounded_sm(self) -> Self {
        self
    }
    fn rounded_md(self) -> Self {
        self.touch(10)
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
    fn overflow_x_hidden(self) -> Self {
        self
    }
    fn overflow_y_hidden(self) -> Self {
        self
    }
    fn absolute(self) -> Self {
        self
    }
    fn relative(self) -> Self {
        self
    }
    // --- shadow ---
    fn shadow_none(self) -> Self {
        self
    }
    fn shadow_2xs(self) -> Self {
        self
    }
    fn shadow_xs(self) -> Self {
        self
    }
    fn shadow_sm(self) -> Self {
        self
    }
    fn shadow_md(self) -> Self {
        self
    }
    fn shadow_lg(self) -> Self {
        self
    }
    fn shadow_xl(self) -> Self {
        self
    }
    fn shadow_2xl(self) -> Self {
        self
    }
    // --- cursor extra ---
    fn cursor_move(self) -> Self {
        self
    }
    fn cursor_not_allowed(self) -> Self {
        self
    }
    fn cursor_context_menu(self) -> Self {
        self
    }
    fn cursor_crosshair(self) -> Self {
        self
    }
    fn cursor_vertical_text(self) -> Self {
        self
    }
    fn cursor_alias(self) -> Self {
        self
    }
    fn cursor_copy(self) -> Self {
        self
    }
    fn cursor_no_drop(self) -> Self {
        self
    }
    fn cursor_grab(self) -> Self {
        self
    }
    fn cursor_grabbing(self) -> Self {
        self
    }
    fn cursor_ew_resize(self) -> Self {
        self
    }
    fn cursor_ns_resize(self) -> Self {
        self
    }
    fn cursor_nesw_resize(self) -> Self {
        self
    }
    fn cursor_nwse_resize(self) -> Self {
        self
    }
    fn cursor_col_resize(self) -> Self {
        self
    }
    fn cursor_row_resize(self) -> Self {
        self
    }
    fn cursor_n_resize(self) -> Self {
        self
    }
    fn cursor_e_resize(self) -> Self {
        self
    }
    fn cursor_s_resize(self) -> Self {
        self
    }
    fn cursor_w_resize(self) -> Self {
        self
    }
    fn debug(self) -> Self {
        self
    }
    // --- color / opacity ---
    fn bg<T>(self, _: T) -> Self {
        self.touch(8)
    }
    fn text_color<T>(self, _: T) -> Self {
        self.touch(9)
    }
    fn border_color<T>(self, _: T) -> Self {
        self
    }
    fn opacity(self, _: f32) -> Self {
        self
    }
    // --- grid ---
    fn grid_cols(self, _: u16) -> Self {
        self
    }
    fn grid_rows(self, _: u16) -> Self {
        self
    }
    fn grid_cols_min_content(self, _: u16) -> Self {
        self
    }
    fn grid_cols_max_content(self, _: u16) -> Self {
        self
    }
    fn grid_rows_min_content(self, _: u16) -> Self {
        self
    }
    fn grid_rows_max_content(self, _: u16) -> Self {
        self
    }
    fn col_span(self, _: u16) -> Self {
        self
    }
    fn col_span_full(self) -> Self {
        self
    }
    fn col_start(self, _: i16) -> Self {
        self
    }
    fn col_start_auto(self) -> Self {
        self
    }
    fn col_end(self, _: i16) -> Self {
        self
    }
    fn col_end_auto(self) -> Self {
        self
    }
    fn row_span(self, _: u16) -> Self {
        self
    }
    fn row_span_full(self) -> Self {
        self
    }
    fn row_start(self, _: i16) -> Self {
        self
    }
    fn row_start_auto(self) -> Self {
        self
    }
    fn row_end(self, _: i16) -> Self {
        self
    }
    fn row_end_auto(self) -> Self {
        self
    }
}

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
