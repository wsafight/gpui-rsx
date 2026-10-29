//! `rsx_runtime!` accepts an explicit GPUI path, for codebases that import GPUI
//! under another path (for example `::gpui` after a package rename).
gpui_rsx::rsx_runtime!(::gpui);

#[test]
fn explicit_gpui_paths_generate_the_same_helpers() {
    let _element = rsx_runtime::px(gpui::StyleRefinement, "flex", &mut |_| {});
    let _element = rsx_runtime::tailwind(gpui::StyleRefinement, "gap-4", &mut |_| {});
}
