#![allow(dead_code)]

//! 共享 Mock GPUI 类型
//!
//! 所有测试文件共用的 MockElement 和辅助函数，
//! 避免在 macro_tests.rs 和 coverage_tests.rs 中重复定义。

/// Mock Element，模拟 GPUI 的 Div / Stateful<Div>。
/// 所有 builder 方法返回 Self 以支持方法链。
#[derive(Debug)]
pub struct MockElement;

/// Styled trait，用于动态 class 的运行时解析
pub trait Styled: Sized {
    fn flex(self) -> Self;
    fn flex_col(self) -> Self;
    fn flex_row(self) -> Self;
    fn flex_1(self) -> Self;
    fn flex_wrap(self) -> Self;
    fn items_center(self) -> Self;
    fn items_start(self) -> Self;
    fn items_end(self) -> Self;
    fn justify_center(self) -> Self;
    fn justify_between(self) -> Self;
    fn justify_start(self) -> Self;
    fn justify_end(self) -> Self;
    fn gap<T>(self, v: T) -> Self;
    fn p<T>(self, v: T) -> Self;
    fn px<T>(self, v: T) -> Self;
    fn py<T>(self, v: T) -> Self;
    fn m<T>(self, v: T) -> Self;
    fn w_full(self) -> Self;
    fn h_full(self) -> Self;
    fn size_full(self) -> Self;
    fn text_xs(self) -> Self;
    fn text_sm(self) -> Self;
    fn text_base(self) -> Self;
    fn text_lg(self) -> Self;
    fn text_xl(self) -> Self;
    fn text_2xl(self) -> Self;
    fn text_3xl(self) -> Self;
    fn font_bold(self) -> Self;
    fn border_1(self) -> Self;
    fn border_2(self) -> Self;
    fn rounded_sm(self) -> Self;
    fn rounded_md(self) -> Self;
    fn rounded_lg(self) -> Self;
    fn rounded_full(self) -> Self;
    fn cursor_pointer(self) -> Self;
    fn overflow_hidden(self) -> Self;
    fn overflow_scroll(self) -> Self;
    fn absolute(self) -> Self;
    fn relative(self) -> Self;
    fn bg<T>(self, v: T) -> Self;
    fn text_color<T>(self, v: T) -> Self;
    fn border_color<T>(self, v: T) -> Self;
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
    pub fn id<T>(self, _: T) -> Self {
        self
    }

    // --- 布局 ---
    pub fn flex_grow<T>(self, _: T) -> Self {
        self
    }
    pub fn flex_shrink<T>(self, _: T) -> Self {
        self
    }

    // --- 间距（参数化 + 预设） ---
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
    pub fn pt<T>(self, _: T) -> Self {
        self
    }
    pub fn pb<T>(self, _: T) -> Self {
        self
    }
    pub fn pl<T>(self, _: T) -> Self {
        self
    }
    pub fn pr<T>(self, _: T) -> Self {
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
    pub fn mx<T>(self, _: T) -> Self {
        self
    }
    pub fn my<T>(self, _: T) -> Self {
        self
    }
    pub fn mt<T>(self, _: T) -> Self {
        self
    }
    pub fn mb<T>(self, _: T) -> Self {
        self
    }
    pub fn ml<T>(self, _: T) -> Self {
        self
    }
    pub fn mr<T>(self, _: T) -> Self {
        self
    }

    // --- 尺寸 ---
    pub fn w<T>(self, _: T) -> Self {
        self
    }
    pub fn h<T>(self, _: T) -> Self {
        self
    }
    pub fn min_w<T>(self, _: T) -> Self {
        self
    }
    pub fn min_h<T>(self, _: T) -> Self {
        self
    }
    pub fn max_w<T>(self, _: T) -> Self {
        self
    }
    pub fn max_h<T>(self, _: T) -> Self {
        self
    }

    // --- 文本 ---
    pub fn text_4xl(self) -> Self {
        self
    }
    pub fn text_5xl(self) -> Self {
        self
    }

    // --- 边框 ---
    pub fn rounded<T>(self, _: T) -> Self {
        self
    }
    pub fn border_4(self) -> Self {
        self
    }

    // --- 定位 ---
    pub fn overflow<T>(self, _: T) -> Self {
        self
    }
    pub fn overflow_visible(self) -> Self {
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

    // --- 光标 ---
    pub fn cursor_default(self) -> Self {
        self
    }
    pub fn cursor_text(self) -> Self {
        self
    }

    // --- 层级和透明度 ---
    pub fn z_index<T>(self, _: T) -> Self {
        self
    }
    pub fn opacity<T>(self, _: T) -> Self {
        self
    }

    // --- 可见性 ---
    pub fn visible<T>(self, _: T) -> Self {
        self
    }

    // --- 事件 ---
    pub fn on_click<T>(self, _: T) -> Self {
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
    pub fn on_key_down<T>(self, _: T) -> Self {
        self
    }
    pub fn on_key_up<T>(self, _: T) -> Self {
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
    pub fn on_drop<T>(self, _: T) -> Self {
        self
    }
    pub fn on_action<T>(self, _: T) -> Self {
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

    // --- 新增属性映射 ---
    pub fn gap_x<T>(self, _: T) -> Self {
        self
    }
    pub fn gap_y<T>(self, _: T) -> Self {
        self
    }
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
    pub fn flex_none(self) -> Self {
        self
    }
    pub fn flex_auto(self) -> Self {
        self
    }

    // --- 新增尺寸 ---
    pub fn shadow_sm(self) -> Self {
        self
    }
    pub fn shadow_md(self) -> Self {
        self
    }
    pub fn shadow_lg(self) -> Self {
        self
    }
    pub fn rounded_xl(self) -> Self {
        self
    }
    pub fn rounded_none(self) -> Self {
        self
    }

    // --- 杂项 ---
    pub fn placeholder<T>(self, _: T) -> Self {
        self
    }
}

// Styled trait 实现 — 供动态 class 运行时 match 表使用
impl Styled for MockElement {
    fn flex(self) -> Self {
        self
    }
    fn flex_col(self) -> Self {
        self
    }
    fn flex_row(self) -> Self {
        self
    }
    fn flex_1(self) -> Self {
        self
    }
    fn flex_wrap(self) -> Self {
        self
    }
    fn items_center(self) -> Self {
        self
    }
    fn items_start(self) -> Self {
        self
    }
    fn items_end(self) -> Self {
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
    fn gap<T>(self, _: T) -> Self {
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
    fn m<T>(self, _: T) -> Self {
        self
    }
    fn w_full(self) -> Self {
        self
    }
    fn h_full(self) -> Self {
        self
    }
    fn size_full(self) -> Self {
        self
    }
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
    fn font_bold(self) -> Self {
        self
    }
    fn border_1(self) -> Self {
        self
    }
    fn border_2(self) -> Self {
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
    fn bg<T>(self, _: T) -> Self {
        self
    }
    fn text_color<T>(self, _: T) -> Self {
        self
    }
    fn border_color<T>(self, _: T) -> Self {
        self
    }
}
