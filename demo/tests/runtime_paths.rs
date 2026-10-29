//! An explicitly named GPUI facade must work with both generated runtime scales.
use gpui as engine;
use gpui::{StyleRefinement, Styled, div, px};

mod explicit {
    gpui_rsx::rsx_runtime!(crate::engine);
}

type ApplyClass = fn(StyleRefinement, &str, &mut dyn FnMut(&str)) -> StyleRefinement;

#[test]
fn explicitly_qualified_runtime_matches_real_gpui_for_both_scales() {
    for (helper, pixels) in [
        (explicit::rsx_runtime::px as ApplyClass, 4.),
        (explicit::rsx_runtime::tailwind, 16.),
    ] {
        let mut unknown = Vec::new();
        let actual = helper(StyleRefinement::default(), "p-4 unknown", &mut |c| {
            unknown.push(c.to_owned())
        });
        let mut expected = div().p(px(pixels));
        assert_eq!(format!("{actual:?}"), format!("{:?}", expected.style()));
        assert_eq!(unknown, ["unknown"]);
    }
}
