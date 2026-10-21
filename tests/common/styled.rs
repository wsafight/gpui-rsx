use super::*;

macro_rules! styled_no_arg_methods {
    ($($name:ident),* $(,)?) => {
        $(fn $name(self) -> Self { self })*
    };
}

macro_rules! styled_value_methods {
    ($($name:ident),* $(,)?) => {
        $(fn $name<T>(self, _: T) -> Self { self })*
    };
}

impl Styled for StyleRefinement {
    fn style(&mut self) -> &mut StyleRefinement {
        self
    }

    styled_no_arg_methods!(
        flex,
        flex_col,
        flex_col_reverse,
        flex_row,
        flex_row_reverse,
        flex_1,
        flex_auto,
        flex_initial,
        flex_none,
        flex_grow,
        flex_grow_0,
        flex_grow_1,
        flex_wrap,
        flex_wrap_reverse,
        flex_nowrap,
        flex_shrink,
        flex_shrink_0,
        flex_shrink_1,
        block,
        grid,
        hidden,
        aspect_square,
        items_center,
        items_start,
        items_end,
        items_baseline,
        items_stretch,
        justify_center,
        justify_between,
        justify_start,
        justify_end,
        justify_around,
        justify_evenly,
        content_normal,
        content_center,
        content_start,
        content_end,
        content_between,
        content_around,
        content_evenly,
        content_stretch,
        self_start,
        self_end,
        self_flex_start,
        self_flex_end,
        self_center,
        self_baseline,
        self_stretch,
        w_full,
        w_px,
        w_auto,
        w_1_2,
        w_1_3,
        h_full,
        h_px,
        h_auto,
        h_1_2,
        h_1_3,
        size_full,
        size_px,
        size_1_2,
        text_xs,
        text_sm,
        text_base,
        text_lg,
        text_xl,
        text_2xl,
        text_3xl,
        text_left,
        text_center,
        text_right,
        whitespace_normal,
        whitespace_nowrap,
        truncate,
        text_ellipsis,
        text_ellipsis_start,
        text_ellipsis_middle,
        italic,
        not_italic,
        underline,
        line_through,
        text_decoration_none,
        text_decoration_solid,
        text_decoration_wavy,
        text_decoration_0,
        text_decoration_1,
        text_decoration_2,
        text_decoration_4,
        text_decoration_8,
        font_bold,
        border_1,
        border_2,
        border_dashed,
        border_t_1,
        border_b_1,
        border_l_1,
        border_r_1,
        border_x_1,
        border_y_1,
        border_t_2,
        border_b_2,
        border_l_2,
        border_r_2,
        border_x_2,
        border_y_2,
        rounded_sm,
        rounded_md,
        rounded_lg,
        rounded_2xl,
        rounded_3xl,
        rounded_full,
        rounded_t_lg,
        rounded_b_lg,
        rounded_r_lg,
        rounded_l_lg,
        cursor_pointer,
        overflow_hidden,
        overflow_x_hidden,
        overflow_y_hidden,
        absolute,
        relative,
        col_span_full,
        col_start_auto,
        col_end_auto,
        row_span_full,
        row_start_auto,
        row_end_auto,
        cursor_default,
        cursor_text,
        cursor_move,
        cursor_not_allowed,
        cursor_context_menu,
        cursor_crosshair,
        cursor_vertical_text,
        cursor_alias,
        cursor_copy,
        cursor_no_drop,
        cursor_grab,
        cursor_grabbing,
        cursor_ew_resize,
        cursor_ns_resize,
        cursor_nesw_resize,
        cursor_nwse_resize,
        cursor_col_resize,
        cursor_row_resize,
        cursor_n_resize,
        cursor_e_resize,
        cursor_s_resize,
        cursor_w_resize,
        debug,
        shadow_none,
        shadow_2xs,
        shadow_xs,
        shadow_sm,
        shadow_md,
        shadow_lg,
        shadow_xl,
        shadow_2xl,
        rounded_none,
        rounded_xl,
    );

    styled_value_methods!(
        gap,
        gap_x,
        gap_y,
        p,
        px,
        py,
        pt,
        pb,
        pl,
        pr,
        m,
        mx,
        my,
        mt,
        mb,
        ml,
        mr,
        size,
        w,
        h,
        min_w,
        max_w,
        min_h,
        max_h,
        font_family,
        bg,
        text_color,
        border_color,
        opacity,
    );

    fn line_clamp(self, _: usize) -> Self {
        self
    }

    fn font_weight<T>(self, _: T) -> Self
    where
        T: Into<FontWeight>,
    {
        self
    }

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

// Styled trait 实现 — 供动态 class 运行时 match 表使用
impl Styled for MockElement {
    fn style(&mut self) -> &mut StyleRefinement {
        Box::leak(Box::new(StyleRefinement::default()))
    }

    styled_no_arg_methods!(
        flex,
        flex_col,
        flex_col_reverse,
        flex_row,
        flex_row_reverse,
        flex_1,
        flex_auto,
        flex_initial,
        flex_none,
        flex_grow,
        flex_grow_0,
        flex_grow_1,
        flex_wrap,
        flex_wrap_reverse,
        flex_nowrap,
        flex_shrink,
        flex_shrink_0,
        flex_shrink_1,
        block,
        grid,
        hidden,
        aspect_square,
        items_center,
        items_start,
        items_end,
        items_baseline,
        items_stretch,
        justify_center,
        justify_between,
        justify_start,
        justify_end,
        justify_around,
        justify_evenly,
        content_normal,
        content_center,
        content_start,
        content_end,
        content_between,
        content_around,
        content_evenly,
        content_stretch,
        self_start,
        self_end,
        self_flex_start,
        self_flex_end,
        self_center,
        self_baseline,
        self_stretch,
        w_full,
        w_px,
        w_auto,
        w_1_2,
        w_1_3,
        h_full,
        h_px,
        h_auto,
        h_1_2,
        h_1_3,
        size_full,
        size_px,
        size_1_2,
        text_xs,
        text_sm,
        text_base,
        text_lg,
        text_xl,
        text_2xl,
        text_3xl,
        text_left,
        text_center,
        text_right,
        whitespace_normal,
        whitespace_nowrap,
        truncate,
        text_ellipsis,
        text_ellipsis_start,
        text_ellipsis_middle,
        italic,
        not_italic,
        underline,
        line_through,
        text_decoration_none,
        text_decoration_solid,
        text_decoration_wavy,
        text_decoration_0,
        text_decoration_1,
        text_decoration_2,
        text_decoration_4,
        text_decoration_8,
        font_bold,
        border_1,
        border_2,
        border_dashed,
        border_t_2,
        border_b_2,
        border_l_2,
        border_r_2,
        border_x_2,
        border_y_2,
        rounded_sm,
        rounded_md,
        rounded_lg,
        rounded_2xl,
        rounded_3xl,
        rounded_full,
        rounded_t_lg,
        rounded_b_lg,
        rounded_r_lg,
        rounded_l_lg,
        cursor_pointer,
        overflow_hidden,
        overflow_x_hidden,
        overflow_y_hidden,
        absolute,
        relative,
        col_span_full,
        col_start_auto,
        col_end_auto,
        row_span_full,
        row_start_auto,
        row_end_auto,
        cursor_default,
        cursor_text,
        cursor_move,
        cursor_not_allowed,
        cursor_context_menu,
        cursor_crosshair,
        cursor_vertical_text,
        cursor_alias,
        cursor_copy,
        cursor_no_drop,
        cursor_grab,
        cursor_grabbing,
        cursor_ew_resize,
        cursor_ns_resize,
        cursor_nesw_resize,
        cursor_nwse_resize,
        cursor_col_resize,
        cursor_row_resize,
        cursor_n_resize,
        cursor_e_resize,
        cursor_s_resize,
        cursor_w_resize,
        debug,
        shadow_none,
        shadow_2xs,
        shadow_xs,
        shadow_sm,
        shadow_md,
        shadow_lg,
        shadow_xl,
        shadow_2xl,
        rounded_none,
        rounded_xl,
    );

    styled_value_methods!(
        gap,
        gap_x,
        gap_y,
        p,
        px,
        py,
        pt,
        pb,
        pl,
        pr,
        m,
        mx,
        my,
        mt,
        mb,
        ml,
        mr,
        size,
        w,
        h,
        min_w,
        max_w,
        min_h,
        max_h,
        font_family,
        bg,
        text_color,
        border_color,
        opacity,
    );

    fn line_clamp(self, lines: usize) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("line_clamp", lines as i32)));
        self
    }

    fn font_weight<T>(self, weight: T) -> Self
    where
        T: Into<FontWeight>,
    {
        FONT_WEIGHT_CALLS.with(|c| c.borrow_mut().push(weight.into().0));
        self
    }

    fn border_t_1(self) -> Self {
        BORDER_CALLS.with(|c| c.borrow_mut().push("border_t_1"));
        self
    }
    fn border_b_1(self) -> Self {
        BORDER_CALLS.with(|c| c.borrow_mut().push("border_b_1"));
        self
    }
    fn border_l_1(self) -> Self {
        BORDER_CALLS.with(|c| c.borrow_mut().push("border_l_1"));
        self
    }
    fn border_r_1(self) -> Self {
        BORDER_CALLS.with(|c| c.borrow_mut().push("border_r_1"));
        self
    }
    fn border_x_1(self) -> Self {
        BORDER_CALLS.with(|c| c.borrow_mut().push("border_x_1"));
        self
    }
    fn border_y_1(self) -> Self {
        BORDER_CALLS.with(|c| c.borrow_mut().push("border_y_1"));
        self
    }

    fn grid_cols(self, v: u16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("grid_cols", v as i32)));
        self
    }
    fn grid_rows(self, v: u16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("grid_rows", v as i32)));
        self
    }
    fn grid_cols_min_content(self, v: u16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("grid_cols_min_content", v as i32)));
        self
    }
    fn grid_cols_max_content(self, v: u16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("grid_cols_max_content", v as i32)));
        self
    }
    fn grid_rows_min_content(self, v: u16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("grid_rows_min_content", v as i32)));
        self
    }
    fn grid_rows_max_content(self, v: u16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("grid_rows_max_content", v as i32)));
        self
    }
    fn col_span(self, v: u16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("col_span", v as i32)));
        self
    }
    fn col_start(self, v: i16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("col_start", v as i32)));
        self
    }
    fn col_end(self, v: i16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("col_end", v as i32)));
        self
    }
    fn row_span(self, v: u16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("row_span", v as i32)));
        self
    }
    fn row_start(self, v: i16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("row_start", v as i32)));
        self
    }
    fn row_end(self, v: i16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("row_end", v as i32)));
        self
    }
}
