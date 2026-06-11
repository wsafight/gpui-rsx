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
    /// 接受 &str，捕获 auto-ID 供测试验证。
    /// 新格式："{file}::__rsx_{tag}_L{line}C{col}"（含文件路径前缀）
    /// 所有测试中的 id 属性均为字符串类型，故 &str 签名覆盖全部情况。
    pub fn id(self, id: &str) -> Self {
        // 新格式含文件路径前缀，用 contains 兼容两种格式
        if id.contains("__rsx_") {
            LAST_AUTO_ID.with(|c| *c.borrow_mut() = Some(id.to_string()));
        }
        self
    }

    // --- flex（不在 Styled 中）---
    pub fn flex_grow(self) -> Self {
        self
    }
    pub fn flex_shrink(self) -> Self {
        self
    }

    // --- 间距固定值（不在 Styled 中）---
    pub fn gap_2(self) -> Self {
        self
    }
    pub fn gap_3(self) -> Self {
        self
    }
    pub fn gap_4(self) -> Self {
        self
    }
    pub fn gap_6(self) -> Self {
        self
    }
    pub fn p_2(self) -> Self {
        self
    }
    pub fn p_3(self) -> Self {
        self
    }
    pub fn p_4(self) -> Self {
        self
    }
    pub fn px_2(self) -> Self {
        self
    }
    pub fn px_3(self) -> Self {
        self
    }
    pub fn px_4(self) -> Self {
        self
    }
    pub fn px_6(self) -> Self {
        self
    }
    pub fn py_1(self) -> Self {
        self
    }
    pub fn py_2(self) -> Self {
        self
    }

    // --- 边框（不在 Styled 中）---
    pub fn rounded<T>(self, _: T) -> Self {
        self
    }
    pub fn border_4(self) -> Self {
        self
    }
    pub fn border_t<T>(self, _: T) -> Self {
        self
    }
    pub fn border_b<T>(self, _: T) -> Self {
        self
    }
    pub fn border_l<T>(self, _: T) -> Self {
        self
    }
    pub fn border_r<T>(self, _: T) -> Self {
        self
    }

    // --- 定位（不在 Styled 中）---
    pub fn overflow<T>(self, _: T) -> Self {
        self
    }
    pub fn overflow_x<T>(self, _: T) -> Self {
        self
    }
    pub fn overflow_y<T>(self, _: T) -> Self {
        self
    }
    pub fn top<T>(self, _: T) -> Self {
        self
    }
    pub fn left<T>(self, _: T) -> Self {
        self
    }
    pub fn right<T>(self, _: T) -> Self {
        self
    }
    pub fn bottom<T>(self, _: T) -> Self {
        self
    }

    // --- 可见性 ---
    pub fn visible(self) -> Self {
        self
    }
    pub fn invisible(self) -> Self {
        self
    }

    // --- 事件 ---
    pub fn on_click<T>(self, _: T) -> Self {
        self
    }
    pub fn on_mouse_down<T, U>(self, _: T, _: U) -> Self {
        self
    }
    pub fn on_mouse_up<T, U>(self, _: T, _: U) -> Self {
        self
    }
    pub fn on_mouse_move<T>(self, _: T) -> Self {
        self
    }
    pub fn on_mouse_down_out<T>(self, _: T) -> Self {
        self
    }
    pub fn on_mouse_up_out<T, U>(self, _: T, _: U) -> Self {
        self
    }
    pub fn on_any_mouse_down<T>(self, _: T) -> Self {
        self
    }
    pub fn on_any_mouse_up<T>(self, _: T) -> Self {
        self
    }
    pub fn on_key_down<T>(self, _: T) -> Self {
        self
    }
    pub fn on_key_up<T>(self, _: T) -> Self {
        self
    }
    pub fn on_modifiers_changed<T>(self, _: T) -> Self {
        self
    }
    pub fn on_hover<T>(self, _: T) -> Self {
        self
    }
    pub fn on_scroll_wheel<T>(self, _: T) -> Self {
        self
    }
    pub fn on_drag<T, U>(self, _: T, _: U) -> Self {
        self
    }
    pub fn on_drag_move<T>(self, _: T) -> Self {
        self
    }
    pub fn on_drop<T>(self, _: T) -> Self {
        self
    }
    pub fn on_action<T>(self, _: T) -> Self {
        self
    }
    pub fn on_boxed_action<T, U>(self, _: T, _: U) -> Self {
        self
    }
    // --- 捕获阶段事件 ---
    pub fn capture_any_mouse_down<T>(self, _: T) -> Self {
        self
    }
    pub fn capture_any_mouse_up<T>(self, _: T) -> Self {
        self
    }
    pub fn capture_key_down<T>(self, _: T) -> Self {
        self
    }
    pub fn capture_key_up<T>(self, _: T) -> Self {
        self
    }
    pub fn capture_action<T>(self, _: T) -> Self {
        self
    }

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
    pub fn tooltip<T>(self, _: T) -> Self {
        self
    }
    pub fn group<T>(self, _: T) -> Self {
        self
    }
    pub fn track_focus<T>(self, _: T) -> Self {
        self
    }
    pub fn hoverable_tooltip<T>(self, _: T) -> Self {
        self
    }
    pub fn tooltip_show_delay<T>(self, _: T) -> Self {
        self
    }
    pub fn on_aux_click<T>(self, _: T) -> Self {
        self
    }
    pub fn on_a11y_action<T, U>(self, _: T, _: U) -> Self {
        self
    }
    pub fn overflow_scroll(self) -> Self {
        self
    }
    pub fn overflow_x_scroll(self) -> Self {
        self
    }
    pub fn overflow_y_scroll(self) -> Self {
        self
    }
    pub fn scrollbar_width<T>(self, _: T) -> Self {
        self
    }
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
    pub fn anchor_scroll<T>(self, _: T) -> Self {
        self
    }
    pub fn focusable(self) -> Self {
        self
    }
    pub fn track_scroll<T>(self, _: T) -> Self {
        self
    }
    pub fn role<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_label<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_selected<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_expanded<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_toggled<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_numeric_value<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_min_numeric_value<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_max_numeric_value<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_orientation<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_level<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_position_in_set<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_size_of_set<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_row_index<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_column_index<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_row_count<T>(self, _: T) -> Self {
        self
    }
    pub fn aria_column_count<T>(self, _: T) -> Self {
        self
    }

    // --- 额外属性映射 ---
    pub fn text_size<T>(self, _: T) -> Self {
        self
    }
    pub fn line_height<T>(self, _: T) -> Self {
        self
    }
    pub fn font_family<T>(self, _: T) -> Self {
        self
    }
    pub fn font_weight<T>(self, weight: T) -> Self
    where
        T: Into<FontWeight>,
    {
        FONT_WEIGHT_CALLS.with(|c| c.borrow_mut().push(weight.into().0));
        self
    }
    pub fn font_display(self) -> Self {
        self
    }
    pub fn text_align<T>(self, _: T) -> Self {
        self
    }
    pub fn shadow<T>(self, _: T) -> Self {
        self
    }

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

    pub fn debug(self) -> Self {
        self
    }

    pub fn path<T>(self, _: T) -> Self {
        self
    }

    pub fn grayscale(self, _: bool) -> Self {
        self
    }

    pub fn object_fit<T>(self, _: T) -> Self {
        self
    }

    pub fn with_fallback<T>(self, _: T) -> Self {
        self
    }

    pub fn with_loading<T>(self, _: T) -> Self {
        self
    }

    pub fn image_cache<T>(self, _: T) -> Self {
        self
    }

    pub fn on_children_prepainted<T>(self, _: T) -> Self {
        self
    }

    // --- 新增属性映射（不在 Styled 中）---
    pub fn flex_basis<T>(self, _: T) -> Self {
        self
    }
    pub fn inset<T>(self, _: T) -> Self {
        self
    }
    pub fn rounded_t<T>(self, _: T) -> Self {
        self
    }
    pub fn rounded_b<T>(self, _: T) -> Self {
        self
    }
    pub fn rounded_tl<T>(self, _: T) -> Self {
        self
    }
    pub fn rounded_tr<T>(self, _: T) -> Self {
        self
    }
    pub fn rounded_bl<T>(self, _: T) -> Self {
        self
    }
    pub fn rounded_br<T>(self, _: T) -> Self {
        self
    }

    // --- 杂项 ---
    pub fn placeholder<T>(self, _: T) -> Self {
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
