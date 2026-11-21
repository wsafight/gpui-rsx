use super::*;

include!("styled_impl_methods_1.rs");
include!("styled_impl_methods_2.rs");

impl StyleRefinement {
    pub(crate) fn touch(mut self, operation: u64) -> Self {
        self.checksum = self.checksum.wrapping_mul(16777619) ^ operation;
        self
    }
}

impl Styled for StyleRefinement {
    fn style(&mut self) -> &mut StyleRefinement {
        self
    }

    styled_impl_methods_1!();
    styled_impl_methods_2!();
}

impl Styled for MockElement {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }

    styled_impl_methods_1!();
    styled_impl_methods_2!();
}
