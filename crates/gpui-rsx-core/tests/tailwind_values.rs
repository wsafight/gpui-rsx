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
fn negative_spacing_and_sizing_classes_expand_negated() {
    for (class, expected) in [
        ("-m-4", ".m(px(-4f32))"),
        ("-mt-1.5", ".mt(px(-1.5f32))"),
        ("-ml-[13px]", ".ml(px(-13f32))"),
        ("-w-[0.5rem]", ".w(rems(-0.5f32))"),
        ("-w-1/2", ".w(relative(-0.5f32))"),
        ("-size-2", ".size(px(-2f32))"),
        ("-min-w-4", ".min_w(px(-4f32))"),
    ] {
        let code = expand_str(
            &format!("<div class={class:?}/>"),
            ClassMode::STRICT.with_scale(SpacingScale::Px),
        )
        .unwrap()
        .to_string();
        let compact = code.split_whitespace().collect::<String>();
        assert!(compact.contains(expected), "{class}: {code}");
        // Tailwind scale multiplies the same unit count.
        let scaled = expand_str(
            &format!("<div class={class:?}/>"),
            ClassMode::STRICT.with_scale(SpacingScale::Tailwind),
        )
        .unwrap()
        .to_string();
        assert!(!scaled.contains("compile_error"), "{class}: {scaled}");
        // Negative classes compose with state variants.
        assert!(matches!(
            analyze_class(&format!("hover:{class}"), ClassMode::STRICT).outcome,
            ClassOutcome::Supported(_)
        ));
    }
}

#[test]
fn negative_preset_class_uses_the_gpui_negated_method() {
    let code = expand_str(
        "<div class=\"-m-px -m-0p5\"/>",
        ClassMode::STRICT.with_scale(SpacingScale::Px),
    )
    .unwrap()
    .to_string();
    let compact = code.split_whitespace().collect::<String>();
    assert!(compact.contains(".m_neg_px()"), "{code}");
    assert!(compact.contains(".m_neg_0p5()"), "{code}");
}

#[test]
fn negative_values_reject_the_same_inputs_as_positive_values() {
    for class in [
        "-m-[10%]", // percentages stay spacing-hostile
        "-p-1/2",   // fractions stay sizing-only
        "-w-[abc]",
        "-w-[1e999px]",
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
fn unsupported_negative_classes_are_not_suggested_with_a_sign_flip() {
    for class in ["-mx-auto", "-rounded-4", "-leading-4", "-m-foo"] {
        let ClassOutcome::Ignored { message } = analyze_class(class, ClassMode::PERMISSIVE).outcome
        else {
            panic!("{class}");
        };
        assert!(
            message.contains("GPUI does not support"),
            "{class}: {message}"
        );
        assert!(matches!(
            analyze_class(class, ClassMode::STRICT).outcome,
            ClassOutcome::Error { .. }
        ));
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
