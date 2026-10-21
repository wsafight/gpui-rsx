macro_rules! styled_impl_methods_2 {
    () => {
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

    };
}
