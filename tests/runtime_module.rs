//! The `rsx_runtime!` macro generates shared dynamic class helpers for a crate.
//!
//! This test crate invokes the macro once at the root, as a library would, and
//! checks the public helpers it generates.
gpui_rsx::rsx_runtime!();

#[test]
fn shared_helpers_reject_repeated_color_opacity_modifiers() {
    for helper in [rsx_runtime::px, rsx_runtime::tailwind] {
        let mut unknown = Vec::new();
        let _ = helper(
            gpui::StyleRefinement,
            "bg-red-500/50/50 bg-red-500/50",
            &mut |class| {
                unknown.push(class.to_owned());
            },
        );
        assert_eq!(unknown, ["bg-red-500/50/50"]);
    }
}

#[test]
fn generated_helpers_apply_known_and_report_unknown_classes() {
    let mut unknown = Vec::new();
    let _element = rsx_runtime::px(
        gpui::StyleRefinement,
        "flex not-a-real-class",
        &mut |class| unknown.push(class.to_owned()),
    );
    assert_eq!(unknown, ["not-a-real-class"]);

    // The Tailwind helper uses the same class matcher with its 4px scale.
    let _element = rsx_runtime::tailwind(gpui::StyleRefinement, "gap-4", &mut |_| {});
}

#[test]
fn generated_helpers_are_usable_as_function_values() {
    let helper = rsx_runtime::px;
    let _ = helper(gpui::StyleRefinement, "flex", &mut |_| {});
}
