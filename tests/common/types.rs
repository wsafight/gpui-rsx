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
    pub flex_shrink: Option<f32>,
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
    fn flex_grow_1(self) -> Self;
    fn flex_wrap(self) -> Self;
    fn flex_wrap_reverse(self) -> Self;
    fn flex_nowrap(self) -> Self;
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
