use super::*;

#[allow(dead_code)] // benchmark 中并非所有方法都被使用
impl MockElement {
    pub(crate) fn touch(mut self, operation: u64) -> Self {
        self.checksum = self.checksum.wrapping_mul(16777619) ^ operation;
        self
    }

    pub(crate) fn id<T>(self, _: T) -> Self {
        self.touch(1)
    }
    pub(crate) fn flex(self) -> Self {
        self.touch(2)
    }
    pub(crate) fn flex_col(self) -> Self {
        self.touch(3)
    }
    mock_no_arg_methods!(flex_row, flex_1, flex_wrap);

    pub(crate) fn gap<T>(self, _: T) -> Self {
        self.touch(4)
    }
    pub(crate) fn gap_4(self) -> Self {
        self.touch(5)
    }
    mock_no_arg_methods!(gap_2, gap_6);

    pub(crate) fn p<T>(self, _: T) -> Self {
        self.touch(6)
    }
    pub(crate) fn p_4(self) -> Self {
        self.touch(7)
    }
    mock_no_arg_methods!(p_2);

    pub(crate) fn bg<T>(self, _: T) -> Self {
        self.touch(8)
    }
    pub(crate) fn text_color<T>(self, _: T) -> Self {
        self.touch(9)
    }
    mock_value_methods!(px, py, m, font_weight);
    mock_no_arg_methods!(px_2, px_4, py_2, py_4, m_2, m_4);

    pub(crate) fn rounded_md(self) -> Self {
        self.touch(10)
    }
    mock_no_arg_methods!(
        rounded_lg,
        rounded_full,
        items_center,
        items_start,
        items_end,
        justify_center,
        justify_between,
        justify_start,
        justify_end,
        w_full,
        h_full,
        size_full,
        text_xs,
        text_sm,
        text_base,
        text_lg,
        text_xl,
        text_2xl,
        text_3xl,
        font_bold,
        border,
        border_1,
        border_2,
        cursor_pointer,
        overflow_hidden,
        overflow_scroll,
        rounded_sm,
        absolute,
        relative,
        debug,
    );

    pub(crate) fn child<T>(self, _: T) -> Self {
        self.touch(11)
    }
    pub(crate) fn children<I: IntoIterator>(self, _: I) -> Self {
        self.touch(12)
    }
    pub(crate) fn map<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }
    pub(crate) fn when<F>(self, _condition: bool, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }
    pub(crate) fn style(&mut self) -> &mut StyleRefinement {
        Box::leak(Box::new(StyleRefinement::default()))
    }
}
