#![allow(dead_code)]

//! 共享 Mock GPUI 类型
//!
//! 所有测试文件共用的 MockElement 和辅助函数，
//! 避免在 macro_tests.rs 和 coverage_tests.rs 中重复定义。

mod capture;
mod types;

pub use capture::*;
pub use types::*;

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

macro_rules! mock_no_arg_methods {
    ($($name:ident),* $(,)?) => {
        $(pub fn $name(self) -> Self { self })*
    };
}

macro_rules! mock_value_methods {
    ($($name:ident),* $(,)?) => {
        $(pub fn $name<T>(self, _: T) -> Self { self })*
    };
}

macro_rules! mock_two_value_methods {
    ($($name:ident),* $(,)?) => {
        $(pub fn $name<T, U>(self, _: T, _: U) -> Self { self })*
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

// 模拟 GPUI 构造函数
#[allow(dead_code)]
pub fn div() -> MockElement {
    MockElement
}
#[allow(dead_code)]
pub fn svg() -> MockElement {
    MockElement
}
#[allow(dead_code)]
pub fn img<T>(_source: T) -> MockElement {
    MockElement
}
#[allow(dead_code)]
pub fn canvas<P, Paint, T>(_prepaint: P, _paint: Paint) -> MockElement
where
    P: FnOnce(Bounds, &mut Window, &mut App) -> T,
    Paint: FnOnce(Bounds, T, &mut Window, &mut App),
{
    MockElement
}
pub fn rgb(hex: u32) -> u32 {
    RGB_CALLS.with(|c| c.borrow_mut().push(hex));
    hex
}
pub fn rgba(hex: u32) -> u32 {
    RGBA_CALLS.with(|c| c.borrow_mut().push(hex));
    hex
}
pub fn px(val: f32) -> f32 {
    LENGTH_CALLS.with(|c| c.borrow_mut().push(("px", val)));
    val
}
pub fn rems(val: f32) -> f32 {
    LENGTH_CALLS.with(|c| c.borrow_mut().push(("rems", val)));
    val
}
pub fn relative(val: f32) -> f32 {
    LENGTH_CALLS.with(|c| c.borrow_mut().push(("relative", val)));
    val
}
pub fn auto() -> f32 {
    0.0
}

// 模拟自定义组件构造函数
#[allow(non_snake_case, dead_code)]
pub fn MyComponent() -> MockElement {
    MockElement
}
#[allow(non_snake_case, dead_code)]
pub fn CustomWidget() -> MockElement {
    MockElement
}

#[allow(dead_code)]
impl MockElement {
    // --- 身份 ---
    /// 接受静态或动态字符串，捕获 auto-ID 供测试验证。
    /// 新格式："{file}::__rsx_{tag}_L{line}C{col}"（含文件路径前缀）
    pub fn id(self, id: impl AsRef<str>) -> Self {
        let id = id.as_ref();
        // 新格式含文件路径前缀，用 contains 兼容两种格式
        if id.contains("__rsx_") {
            LAST_AUTO_ID.with(|c| *c.borrow_mut() = Some(id.to_string()));
        }
        self
    }

    mock_no_arg_methods!(
        flex_grow,
        flex_shrink,
        gap_2,
        gap_3,
        gap_4,
        gap_6,
        p_2,
        p_3,
        p_4,
        px_2,
        px_3,
        px_4,
        px_6,
        py_1,
        py_2,
        border_4,
        visible,
        invisible,
    );

    mock_value_methods!(
        rounded, border_t, border_b, border_l, border_r, overflow, overflow_x, overflow_y, top,
        left, right, bottom,
    );

    // --- 事件 ---
    mock_value_methods!(
        on_click,
        on_mouse_move,
        on_mouse_exit,
        on_mouse_pressure,
        on_pinch,
        on_mouse_down_out,
        on_any_mouse_down,
        on_any_mouse_up,
        on_key_down,
        on_key_up,
        on_modifiers_changed,
        on_hover,
        on_scroll_wheel,
        on_drag_move,
        on_drop,
        on_action,
        capture_any_mouse_down,
        capture_any_mouse_up,
        capture_key_down,
        capture_key_up,
        capture_action,
        capture_mouse_pressure,
        capture_pinch,
    );
    mock_two_value_methods!(
        on_mouse_down,
        on_mouse_up,
        on_mouse_up_out,
        on_drag,
        on_boxed_action,
    );

    // --- 状态样式 ---
    pub fn hover<F: FnOnce(StyleRefinement) -> StyleRefinement>(self, _f: F) -> Self {
        self
    }
    pub fn active<F: FnOnce(StyleRefinement) -> StyleRefinement>(self, _f: F) -> Self {
        self
    }
    pub fn focus<F: FnOnce(StyleRefinement) -> StyleRefinement>(self, _f: F) -> Self {
        self
    }
    pub fn focus_visible<F: FnOnce(StyleRefinement) -> StyleRefinement>(self, _f: F) -> Self {
        self
    }
    pub fn in_focus<F: FnOnce(StyleRefinement) -> StyleRefinement>(self, _f: F) -> Self {
        self
    }
    mock_value_methods!(
        tooltip,
        group,
        track_focus,
        hoverable_tooltip,
        tooltip_show_delay,
        on_aux_click,
        scrollbar_width,
    );
    mock_two_value_methods!(on_a11y_action);
    mock_no_arg_methods!(
        overflow_scroll,
        overflow_x_scroll,
        overflow_y_scroll,
        restrict_scroll_to_axis,
    );
    pub fn group_hover<T, F: FnOnce(StyleRefinement) -> StyleRefinement>(
        self,
        _: T,
        _f: F,
    ) -> Self {
        self
    }
    pub fn group_active<T, F: FnOnce(StyleRefinement) -> StyleRefinement>(
        self,
        _: T,
        _f: F,
    ) -> Self {
        self
    }
    pub fn group_drag_over<T, F: FnOnce(StyleRefinement) -> StyleRefinement>(
        self,
        _: T,
        _f: F,
    ) -> Self {
        self
    }
    mock_no_arg_methods!(focusable, aria_active_descendant);
    mock_value_methods!(
        anchor_scroll,
        track_scroll,
        role,
        accessibility_id,
        aria_label,
        aria_description,
        aria_keyshortcuts,
        a11y_synthetic_children,
        aria_selected,
        aria_expanded,
        aria_toggled,
        aria_numeric_value,
        aria_numeric_value_step,
        aria_value,
        aria_placeholder,
        aria_min_numeric_value,
        aria_max_numeric_value,
        aria_orientation,
        aria_level,
        aria_position_in_set,
        aria_size_of_set,
        aria_row_index,
        aria_column_index,
        aria_row_count,
        aria_column_count,
        external_drag_payload,
    );

    // --- 额外属性映射 ---
    mock_value_methods!(text_size, line_height, font_family, text_align, shadow);
    pub fn font_weight<T>(self, weight: T) -> Self
    where
        T: Into<FontWeight>,
    {
        FONT_WEIGHT_CALLS.with(|c| c.borrow_mut().push(weight.into().0));
        self
    }
    mock_no_arg_methods!(font_display);

    // --- 子节点 ---
    pub fn child<T>(self, _: T) -> Self {
        self
    }
    pub fn children<I: IntoIterator>(self, _: I) -> Self {
        self
    }

    // --- 条件方法 ---
    pub fn when<F>(self, _condition: bool, _f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        self
    }

    pub fn when_some<T, F>(self, _option: Option<T>, _f: F) -> Self
    where
        F: FnOnce(Self, T) -> Self,
    {
        self
    }

    // --- 转换方法 ---
    pub fn map<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }

    pub fn style(&mut self) -> &mut StyleRefinement {
        Box::leak(Box::new(StyleRefinement::default()))
    }

    mock_no_arg_methods!(debug);
    mock_value_methods!(
        path,
        object_fit,
        with_fallback,
        with_loading,
        image_cache,
        on_children_prepainted,
        flex_basis,
        inset,
        rounded_t,
        rounded_b,
        rounded_tl,
        rounded_tr,
        rounded_bl,
        rounded_br,
        placeholder,
    );

    pub fn grayscale(self, _: bool) -> Self {
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
