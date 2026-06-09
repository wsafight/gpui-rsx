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
    pub static RGB_CALLS: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
    pub static RGBA_CALLS: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
    pub static BORDER_CALLS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    pub static FONT_WEIGHT_CALLS: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
    pub static LENGTH_CALLS: RefCell<Vec<(&'static str, f32)>> = const { RefCell::new(Vec::new()) };
    pub static INTEGER_CALLS: RefCell<Vec<(&'static str, i32)>> = const { RefCell::new(Vec::new()) };
}

pub fn take_border_calls() -> Vec<&'static str> {
    BORDER_CALLS.with(|c| c.borrow_mut().drain(..).collect())
}

/// 返回最近捕获的 auto-ID，并清空缓存。
pub fn take_last_auto_id() -> Option<String> {
    LAST_AUTO_ID.with(|c| c.borrow_mut().take())
}

/// 返回测试期间捕获的 rgb() 入参，并清空缓存。
pub fn take_rgb_calls() -> Vec<u32> {
    RGB_CALLS.with(|c| c.borrow_mut().drain(..).collect())
}

/// 返回测试期间捕获的 rgba() 入参，并清空缓存。
pub fn take_rgba_calls() -> Vec<u32> {
    RGBA_CALLS.with(|c| c.borrow_mut().drain(..).collect())
}

/// 返回测试期间捕获的 font_weight() 入参，并清空缓存。
pub fn take_font_weight_calls() -> Vec<f32> {
    FONT_WEIGHT_CALLS.with(|c| c.borrow_mut().drain(..).collect())
}

/// 返回测试期间捕获的长度 helper 入参，并清空缓存。
pub fn take_length_calls() -> Vec<(&'static str, f32)> {
    LENGTH_CALLS.with(|c| c.borrow_mut().drain(..).collect())
}

/// 返回测试期间捕获的整数 helper 入参，并清空缓存。
pub fn take_integer_calls() -> Vec<(&'static str, i32)> {
    INTEGER_CALLS.with(|c| c.borrow_mut().drain(..).collect())
}

/// Mock Element，模拟 GPUI 的 Div / Stateful<Div>。
/// 所有 builder 方法返回 Self 以支持方法链。
#[derive(Debug)]
pub struct MockElement;

#[derive(Debug)]
pub struct AnyElement;

#[derive(Debug)]
pub struct Bounds;

#[derive(Debug)]
pub struct Window;

#[derive(Debug)]
pub struct App;

#[derive(Clone, Copy, Debug)]
pub struct FontWeight(pub f32);

impl FontWeight {
    pub const THIN: FontWeight = FontWeight(100.0);
    pub const EXTRA_LIGHT: FontWeight = FontWeight(200.0);
    pub const LIGHT: FontWeight = FontWeight(300.0);
    pub const NORMAL: FontWeight = FontWeight(400.0);
    pub const MEDIUM: FontWeight = FontWeight(500.0);
    pub const SEMIBOLD: FontWeight = FontWeight(600.0);
    pub const BOLD: FontWeight = FontWeight(700.0);
    pub const EXTRA_BOLD: FontWeight = FontWeight(800.0);
    pub const BLACK: FontWeight = FontWeight(900.0);
}

impl From<i32> for FontWeight {
    fn from(value: i32) -> Self {
        Self(value as f32)
    }
}

impl From<f32> for FontWeight {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AlignItems {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

pub type JustifyContent = AlignContent;

#[derive(Clone, Copy, Debug)]
pub enum AlignContent {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

#[derive(Default, Debug)]
pub struct StyleRefinement {
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
    pub align_content: Option<AlignContent>,
    pub aspect_ratio: Option<f32>,
    pub flex_grow: Option<f32>,
    pub debug: Option<bool>,
}

/// Minimal stand-in for GPUI's IntoElement trait.
pub trait IntoElement: Sized {
    fn into_any_element(self) -> AnyElement {
        AnyElement
    }
}

impl IntoElement for MockElement {}
impl IntoElement for &str {}
impl IntoElement for String {}
impl IntoElement for i32 {}
impl IntoElement for usize {}
impl IntoElement for u32 {}

#[derive(Clone, Copy, Debug)]
pub enum MouseButton {
    Left,
    Right,
}

/// Styled trait，用于动态 class 的运行时解析
pub trait Styled: Sized {
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
    fn flex_grow_0(self) -> Self;
    fn flex_wrap(self) -> Self;
    fn flex_wrap_reverse(self) -> Self;
    fn flex_nowrap(self) -> Self;
    fn flex_shrink(self) -> Self;
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
    fn whitespace_normal(self) -> Self;
    fn whitespace_nowrap(self) -> Self;
    fn truncate(self) -> Self;
    fn text_ellipsis(self) -> Self;
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
    fn font_family<T>(self, v: T) -> Self;
    fn font_weight<T>(self, v: T) -> Self
    where
        T: Into<FontWeight>;
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
    fn rounded_sm(self) -> Self;
    fn rounded_md(self) -> Self;
    fn rounded_lg(self) -> Self;
    fn rounded_2xl(self) -> Self;
    fn rounded_3xl(self) -> Self;
    fn rounded_full(self) -> Self;
    fn rounded_t_lg(self) -> Self;
    fn rounded_b_lg(self) -> Self;
    fn rounded_r_lg(self) -> Self;
    fn rounded_l_lg(self) -> Self;
    // --- misc ---
    fn cursor_pointer(self) -> Self;
    fn overflow_hidden(self) -> Self;
    fn overflow_x_hidden(self) -> Self;
    fn overflow_y_hidden(self) -> Self;
    fn absolute(self) -> Self;
    fn relative(self) -> Self;
    // --- color / opacity ---
    fn bg<T>(self, v: T) -> Self;
    fn text_color<T>(self, v: T) -> Self;
    fn border_color<T>(self, v: T) -> Self;
    fn opacity<T>(self, v: T) -> Self;
    // --- grid ---
    fn grid_cols(self, v: u16) -> Self;
    fn grid_rows(self, v: u16) -> Self;
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
    // --- cursor ---
    fn cursor_default(self) -> Self;
    fn cursor_text(self) -> Self;
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
    // --- shadow ---
    fn shadow_none(self) -> Self;
    fn shadow_2xs(self) -> Self;
    fn shadow_xs(self) -> Self;
    fn shadow_sm(self) -> Self;
    fn shadow_md(self) -> Self;
    fn shadow_lg(self) -> Self;
    fn shadow_xl(self) -> Self;
    fn shadow_2xl(self) -> Self;
    // --- rounded extra ---
    fn rounded_none(self) -> Self;
    fn rounded_xl(self) -> Self;
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
    pub fn track_focus<T>(self, _: T) -> Self {
        self
    }
    pub fn hoverable_tooltip<T>(self, _: T) -> Self {
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
    pub fn group_active<F: FnOnce(Self) -> Self>(self, _f: F) -> Self {
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
    fn flex_grow(self) -> Self {
        self
    }
    fn flex_grow_0(self) -> Self {
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
    fn flex_shrink(self) -> Self {
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
    fn line_clamp(self, lines: usize) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("line_clamp", lines as i32)));
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
    fn font_family<T>(self, _: T) -> Self {
        self
    }
    fn font_weight<T>(self, weight: T) -> Self
    where
        T: Into<FontWeight>,
    {
        FONT_WEIGHT_CALLS.with(|c| c.borrow_mut().push(weight.into().0));
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
    fn rounded_sm(self) -> Self {
        self
    }
    fn rounded_md(self) -> Self {
        self
    }
    fn rounded_lg(self) -> Self {
        self
    }
    fn rounded_2xl(self) -> Self {
        self
    }
    fn rounded_3xl(self) -> Self {
        self
    }
    fn rounded_full(self) -> Self {
        self
    }
    fn rounded_t_lg(self) -> Self {
        self
    }
    fn rounded_b_lg(self) -> Self {
        self
    }
    fn rounded_r_lg(self) -> Self {
        self
    }
    fn rounded_l_lg(self) -> Self {
        self
    }
    // --- misc ---
    fn cursor_pointer(self) -> Self {
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
    // --- color / opacity ---
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
    // --- grid ---
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
    fn col_span_full(self) -> Self {
        self
    }
    fn col_start(self, v: i16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("col_start", v as i32)));
        self
    }
    fn col_start_auto(self) -> Self {
        self
    }
    fn col_end(self, v: i16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("col_end", v as i32)));
        self
    }
    fn col_end_auto(self) -> Self {
        self
    }
    fn row_span(self, v: u16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("row_span", v as i32)));
        self
    }
    fn row_span_full(self) -> Self {
        self
    }
    fn row_start(self, v: i16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("row_start", v as i32)));
        self
    }
    fn row_start_auto(self) -> Self {
        self
    }
    fn row_end(self, v: i16) -> Self {
        INTEGER_CALLS.with(|c| c.borrow_mut().push(("row_end", v as i32)));
        self
    }
    fn row_end_auto(self) -> Self {
        self
    }
    // --- cursor ---
    fn cursor_default(self) -> Self {
        self
    }
    fn cursor_text(self) -> Self {
        self
    }
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
    // --- rounded extra ---
    fn rounded_none(self) -> Self {
        self
    }
    fn rounded_xl(self) -> Self {
        self
    }
}
