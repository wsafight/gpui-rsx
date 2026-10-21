macro_rules! styled_impl_methods_1 {
    () => {
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

    };
}
