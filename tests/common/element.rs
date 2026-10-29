use super::*;

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

#[allow(dead_code)]
impl MockElement {
    // --- 身份 ---
    /// 接受静态或动态字符串，捕获 auto-ID 供测试验证。
    /// 新格式："{file}::__rsx_{tag}_L{line}C{col}"（含文件路径前缀）
    pub fn id(self, id: impl MockId) -> Self {
        let id = id.mock_id();
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

pub trait MockId {
    fn mock_id(self) -> String;
}
impl MockId for &str {
    fn mock_id(self) -> String {
        self.to_owned()
    }
}
impl MockId for String {
    fn mock_id(self) -> String {
        self
    }
}
impl MockId for &String {
    fn mock_id(self) -> String {
        self.clone()
    }
}
impl MockId for (&'static str, u64) {
    fn mock_id(self) -> String {
        format!("{}{}", self.0, self.1)
    }
}
