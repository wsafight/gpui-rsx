use super::*;

include!("styled_impl_methods_1.rs");
include!("styled_impl_methods_2.rs");

impl Styled for MockElement {
    fn style(&mut self) -> &mut StyleRefinement {
        Box::leak(Box::new(StyleRefinement::default()))
    }

    styled_impl_methods_1!();
    styled_impl_methods_2!();
}
