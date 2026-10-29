use gpui_rsx_core::{ClassMode, ClassOutcome, SpacingScale, analyze_class, expand_str};

#[test]
fn common_tailwind_values_and_variants_expand_to_gpui() {
    for (class, expected) in [
        ("bg-red-500/50", ". bg (rgba (4014228608u32))"),
        ("rounded-[4px]", ". rounded (px (4f32))"),
        ("text-[14px]", ". text_size (px (14f32))"),
        ("leading-6", ". line_height (px (24f32))"),
        ("text-[#f00]", ". text_color (rgb (16711680u32))"),
    ] {
        let code = expand_str(
            &format!("<div class={class:?}/>"),
            ClassMode::STRICT.with_scale(SpacingScale::Tailwind),
        )
        .unwrap()
        .to_string();
        assert!(code.contains(expected), "{class}: {code}");
        assert!(matches!(
            analyze_class(&format!("hover:{class}"), ClassMode::STRICT).outcome,
            ClassOutcome::Supported(_)
        ));
    }
    for class in [
        "bg-red-500/101",
        "bg-red-500/NaN",
        "bg-red-500/50/50",
        "text-[#1234]/50/50",
        "border-[rgba(1,2,3,0.5)]/50/50",
        "text-[NaNpx]",
        "rounded-[50%]",
    ] {
        assert!(
            matches!(
                analyze_class(class, ClassMode::PERMISSIVE).outcome,
                ClassOutcome::Error { .. }
            ),
            "{class}"
        );
    }
}

#[test]
fn css_only_utilities_have_specific_diagnostics() {
    for class in [
        "z-10",
        "space-x-4",
        "divide-y",
        "transition",
        "duration-100",
    ] {
        let ClassOutcome::Ignored { message } = analyze_class(class, ClassMode::PERMISSIVE).outcome
        else {
            panic!("{class}");
        };
        assert!(message.contains("GPUI does not support"));
        assert!(matches!(
            analyze_class(class, ClassMode::STRICT).outcome,
            ClassOutcome::Error { .. }
        ));
    }
}
