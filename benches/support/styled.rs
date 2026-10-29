use super::*;

include!("generated_styled.rs");

// Styled trait - 动态 class 代码生成需要此 trait
// 必须与 generate_common_class_matches 生成的所有方法调用保持一致。
#[allow(dead_code)] // benchmark 中并非所有 trait 方法都被使用
pub(crate) trait Styled: Sized {
    fn text_size(self, _: impl Sized) -> Self {
        self
    }
    fn rounded(self, _: impl Sized) -> Self {
        self
    }
    fn line_height(self, _: impl Sized) -> Self {
        self
    }
    generated_styled_defaults!();
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

#[path = "styled_impl.rs"]
mod styled_impl;
