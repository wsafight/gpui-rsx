#![allow(dead_code)]

//! 共享 Mock GPUI 类型
//!
//! 所有测试文件共用的 MockElement 和辅助函数，
//! 避免在 macro_tests.rs 和 coverage_tests.rs 中重复定义。

use std::cell::RefCell;

// 捕获最近一次自动生成的 ID（以 `__rsx_` 开头）。
// 供 auto-ID 格式验证测试使用。
thread_local! {
    pub static LAST_AUTO_ID: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// 返回最近捕获的 auto-ID，并清空缓存。
pub fn take_last_auto_id() -> Option<String> {
    LAST_AUTO_ID.with(|c| c.borrow_mut().take())
}

/// Mock Element，模拟 GPUI 的 Div / Stateful<Div>。
/// 所有 builder 方法返回 Self 以支持方法链。
#[derive(Debug)]
pub struct MockElement;

/// Styled trait，用于动态 class 的运行时解析
pub trait Styled: Sized {
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
    fn gap<T>(self, v: T) -> Self;
    fn gap_x<T>(self, v: T) -> Self;
    fn gap_y<T>(self, v: T) -> Self;
    fn p<T>(self, v: T) -> Self;
    fn px<T>(self, v: T) -> Self;
    fn py<T>(self, v: T) -> Self;
    fn pt<T>(self, v: T) -> Self;
    fn pb<T>(self, v: T) -> Self;
    fn pl<T>(self, v: T) -> Self;
    fn pr<T>(self, v: T) -> Self;
    fn m<T>(self, v: T) -> Self;
    fn mx<T>(self, v: T) -> Self;
    fn my<T>(self, v: T) -> Self;
    fn mt<T>(self, v: T) -> Self;
    fn mb<T>(self, v: T) -> Self;
    fn ml<T>(self, v: T) -> Self;
    fn mr<T>(self, v: T) -> Self;
    // --- sizing ---
    fn w_full(self) -> Self;
    fn h_full(self) -> Self;
    fn size_full(self) -> Self;
    fn size<T>(self, v: T) -> Self;
    fn w<T>(self, v: T) -> Self;
    fn h<T>(self, v: T) -> Self;
    fn min_w<T>(self, v: T) -> Self;
    fn max_w<T>(self, v: T) -> Self;
    fn min_h<T>(self, v: T) -> Self;
    fn max_h<T>(self, v: T) -> Self;
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
    fn rounded_sm(self) -> Self;
    fn rounded_md(self) -> Self;
    fn rounded_lg(self) -> Self;
    fn rounded_full(self) -> Self;
    // --- misc ---
    fn cursor_pointer(self) -> Self;
    fn overflow_hidden(self) -> Self;
    fn overflow_scroll(self) -> Self;
    fn absolute(self) -> Self;
    fn relative(self) -> Self;
    // --- color / opacity / z ---
    fn bg<T>(self, v: T) -> Self;
    fn text_color<T>(self, v: T) -> Self;
    fn border_color<T>(self, v: T) -> Self;
    fn opacity<T>(self, v: T) -> Self;
    fn z_index<T>(self, v: T) -> Self;
    // --- grid ---
    fn grid_cols(self, v: u16) -> Self;
    fn grid_rows(self, v: u16) -> Self;
    fn col_span(self, v: u16) -> Self;
    fn col_start(self, v: i16) -> Self;
    fn col_end(self, v: i16) -> Self;
    fn row_span(self, v: u16) -> Self;
    fn row_start(self, v: i16) -> Self;
    fn row_end(self, v: i16) -> Self;
    // --- cursor ---
    fn cursor_default(self) -> Self;
    fn cursor_text(self) -> Self;
    // --- shadow ---
    fn shadow_sm(self) -> Self;
    fn shadow_md(self) -> Self;
    fn shadow_lg(self) -> Self;
    // --- rounded extra ---
    fn rounded_none(self) -> Self;
    fn rounded_xl(self) -> Self;
    // --- overflow ---
    fn overflow_visible(self) -> Self;
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
pub fn img() -> MockElement {
    MockElement
}
#[allow(dead_code)]
pub fn canvas() -> MockElement {
    MockElement
}
pub fn rgb(_hex: u32) -> u32 {
    0
}
pub fn px(_val: f32) -> f32 {
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
    pub fn flex_grow<T>(self, _: T) -> Self {
        self
    }
    pub fn flex_shrink<T>(self, _: T) -> Self {
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

    // --- 可见性（不在 Styled 中）---
    pub fn visible<T>(self, _: T) -> Self {
        self
    }

    // --- 事件 ---
    pub fn on_click<T>(self, _: T) -> Self {
        self
    }
    pub fn on_aux_click<T>(self, _: T) -> Self {
        self
    }
    pub fn on_mouse_down<T>(self, _: T) -> Self {
        self
    }
    pub fn on_mouse_up<T>(self, _: T) -> Self {
        self
    }
    pub fn on_mouse_move<T>(self, _: T) -> Self {
        self
    }
    pub fn on_mouse_down_out<T>(self, _: T) -> Self {
        self
    }
    pub fn on_mouse_up_out<T>(self, _: T) -> Self {
        self
    }
    pub fn on_any_mouse_down<T>(self, _: T) -> Self {
        self
    }
    pub fn on_any_mouse_up<T>(self, _: T) -> Self {
        self
    }
    pub fn on_mouse_pressure<T>(self, _: T) -> Self {
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
    pub fn on_focus<T>(self, _: T) -> Self {
        self
    }
    pub fn on_blur<T>(self, _: T) -> Self {
        self
    }
    pub fn on_hover<T>(self, _: T) -> Self {
        self
    }
    pub fn on_scroll_wheel<T>(self, _: T) -> Self {
        self
    }
    pub fn on_drag<T>(self, _: T) -> Self {
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
    // --- 捕获阶段事件 ---
    pub fn capture_any_mouse_down<T>(self, _: T) -> Self {
        self
    }
    pub fn capture_any_mouse_up<T>(self, _: T) -> Self {
        self
    }
    pub fn capture_mouse_pressure<T>(self, _: T) -> Self {
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
    pub fn hover<F: FnOnce(Self) -> Self>(self, _f: F) -> Self {
        self
    }
    pub fn active<F: FnOnce(Self) -> Self>(self, _f: F) -> Self {
        self
    }
    pub fn focus<F: FnOnce(Self) -> Self>(self, _f: F) -> Self {
        self
    }
    pub fn tooltip<T>(self, _: T) -> Self {
        self
    }
    pub fn group<T>(self, _: T) -> Self {
        self
    }
    pub fn track_focus(self) -> Self {
        self
    }

    // --- 额外属性映射 ---
    pub fn font_size<T>(self, _: T) -> Self {
        self
    }
    pub fn line_height<T>(self, _: T) -> Self {
        self
    }
    pub fn font_weight<T>(self, _: T) -> Self {
        self
    }
    pub fn text_align<T>(self, _: T) -> Self {
        self
    }
    pub fn border_radius<T>(self, _: T) -> Self {
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
    pub fn map<F>(self, _f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        self
    }

    // --- 新增属性映射（不在 Styled 中）---
    pub fn basis<T>(self, _: T) -> Self {
        self
    }
    pub fn order<T>(self, _: T) -> Self {
        self
    }
    pub fn inset<T>(self, _: T) -> Self {
        self
    }
    pub fn text_decoration<T>(self, _: T) -> Self {
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
    fn gap<T>(self, _: T) -> Self {
        self
    }
    fn gap_x<T>(self, _: T) -> Self {
        self
    }
    fn gap_y<T>(self, _: T) -> Self {
        self
    }
    fn p<T>(self, _: T) -> Self {
        self
    }
    fn px<T>(self, _: T) -> Self {
        self
    }
    fn py<T>(self, _: T) -> Self {
        self
    }
    fn pt<T>(self, _: T) -> Self {
        self
    }
    fn pb<T>(self, _: T) -> Self {
        self
    }
    fn pl<T>(self, _: T) -> Self {
        self
    }
    fn pr<T>(self, _: T) -> Self {
        self
    }
    fn m<T>(self, _: T) -> Self {
        self
    }
    fn mx<T>(self, _: T) -> Self {
        self
    }
    fn my<T>(self, _: T) -> Self {
        self
    }
    fn mt<T>(self, _: T) -> Self {
        self
    }
    fn mb<T>(self, _: T) -> Self {
        self
    }
    fn ml<T>(self, _: T) -> Self {
        self
    }
    fn mr<T>(self, _: T) -> Self {
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
    fn size<T>(self, _: T) -> Self {
        self
    }
    fn w<T>(self, _: T) -> Self {
        self
    }
    fn h<T>(self, _: T) -> Self {
        self
    }
    fn min_w<T>(self, _: T) -> Self {
        self
    }
    fn max_w<T>(self, _: T) -> Self {
        self
    }
    fn min_h<T>(self, _: T) -> Self {
        self
    }
    fn max_h<T>(self, _: T) -> Self {
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
    fn rounded_sm(self) -> Self {
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
    // --- misc ---
    fn cursor_pointer(self) -> Self {
        self
    }
    fn overflow_hidden(self) -> Self {
        self
    }
    fn overflow_scroll(self) -> Self {
        self
    }
    fn absolute(self) -> Self {
        self
    }
    fn relative(self) -> Self {
        self
    }
    // --- color / opacity / z ---
    fn bg<T>(self, _: T) -> Self {
        self
    }
    fn text_color<T>(self, _: T) -> Self {
        self
    }
    fn border_color<T>(self, _: T) -> Self {
        self
    }
    fn opacity<T>(self, _: T) -> Self {
        self
    }
    fn z_index<T>(self, _: T) -> Self {
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
    // --- cursor ---
    fn cursor_default(self) -> Self {
        self
    }
    fn cursor_text(self) -> Self {
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
    // --- rounded extra ---
    fn rounded_none(self) -> Self {
        self
    }
    fn rounded_xl(self) -> Self {
        self
    }
    // --- overflow ---
    fn overflow_visible(self) -> Self {
        self
    }
}
