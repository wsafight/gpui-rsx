use super::*;

fn expand(class: &str, mode: ClassMode) -> String {
    parse_class_string_with_mode(class, mode, Span::call_site())
        .into_iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn malformed_arbitrary_length_reports_compile_error() {
    let tokens = expand("w-[280px", ClassMode::PERMISSIVE);

    assert!(tokens.contains("Invalid length class `w-[280px`"));
    assert!(tokens.contains("compile_error"));
}

#[test]
fn fraction_parser_reports_each_malformed_component() {
    let cases = [
        (
            "12",
            "Invalid fraction `w-12`: expected numerator/denominator.",
        ),
        (
            "many/2",
            "Invalid fraction `w-many/2`: numerator must be a number.",
        ),
        (
            "1/many",
            "Invalid fraction `w-1/many`: denominator must be a number.",
        ),
    ];

    for (value, expected) in cases {
        let class = format!("w-{value}");
        assert_eq!(
            LengthKind::parse_fraction(&class, value).err().as_deref(),
            Some(expected)
        );
    }
}

#[test]
fn fraction_spacing_reports_unsupported_family() {
    let tokens = expand("p-1/2", ClassMode::PERMISSIVE);

    assert!(tokens.contains("fractions are only supported for sizing classes"));
    assert!(tokens.contains("compile_error"));
}

#[test]
fn length_number_rejects_empty_and_non_finite_values() {
    assert_eq!(parse_length_number(""), None);
    assert_eq!(parse_length_number("NaN"), None);
    assert_eq!(parse_length_number("inf"), None);
    assert_eq!(parse_length_number("-inf"), None);
}

#[test]
fn opacity_accepts_finite_decimal_percentages() {
    for (suffix, expected) in [("0", 0.0), ("33.5", 0.335), ("100", 1.0), ("1e2", 1.0)] {
        let class = format!("opacity-{suffix}");
        let operation = resolve_style(&class, ClassMode::STRICT).unwrap().unwrap();

        assert_eq!(operation.method(), "opacity");
        assert_eq!(
            operation.value(),
            Some(&crate::StyleValue::Number(expected)),
            "{class}"
        );
    }
}

#[test]
fn opacity_rejects_out_of_range_and_non_finite_percentages() {
    for suffix in ["", "-0.1", "100.1", "NaN", "inf", "-inf"] {
        let class = format!("opacity-{suffix}");
        let error = resolve_style(&class, ClassMode::STRICT)
            .unwrap()
            .unwrap_err();

        assert!(error.contains("Invalid opacity class"), "{class}: {error}");
    }
}

#[test]
fn variants_group_into_style_refinement_closures() {
    let tokens = expand(
        "bg-blue-500 hover:bg-blue-600 hover:text-white active:opacity-75 group-hover/menu:underline",
        ClassMode::PERMISSIVE,
    );
    assert!(tokens.contains(
        ". hover (| __style | __style . bg (rgb (2450411u32)) . text_color (rgb (16777215u32)))"
    ));
    assert!(tokens.contains(". active (| __style | __style . opacity (0.75f32))"));
    assert!(tokens.contains(". group_hover (\"menu\" , | __style | __style . underline ())"));
}

#[test]
fn unsupported_variants_warn_in_permissive_and_fail_in_strict() {
    let permissive = expand("dark:bg-black md:flex", ClassMode::PERMISSIVE);
    assert!(permissive.contains("deprecated"));
    assert!(permissive.contains("color-scheme"));
    assert!(permissive.contains("breakpoints"));
    assert!(!permissive.contains("compile_error"));

    let strict = expand("dark:bg-black", ClassMode::STRICT);
    assert!(strict.contains("compile_error"));
}

#[test]
fn variants_reject_element_level_classes() {
    let tokens = expand("hover:overflow-scroll", ClassMode::PERMISSIVE);
    assert!(tokens.contains("cannot be used inside the `hover:` variant"));
}

#[test]
fn tailwind_scale_multiplies_numeric_lengths_only() {
    let mode = ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind);
    assert_eq!(expand("gap-4", mode.clone()), ". gap (px (16f32))");
    assert_eq!(expand("p-2.5", mode.clone()), ". p (px (10f32))");
    assert_eq!(expand("w-64", mode.clone()), ". w (px (256f32))");
    assert_eq!(expand("gap-[14px]", mode), ". gap (px (14f32))");
    assert_eq!(expand("gap-4", ClassMode::PERMISSIVE), ". gap (px (4f32))");
}

#[test]
fn tailwind_scale_multiplies_negative_lengths_too() {
    let mode = ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind);
    assert_eq!(expand("-gap-4", mode.clone()), ". gap (px (- 16f32))");
    assert_eq!(expand("-p-2.5", mode.clone()), ". p (px (- 10f32))");
    assert_eq!(expand("-gap-[14px]", mode), ". gap (px (- 14f32))");
    assert_eq!(
        expand("-gap-4", ClassMode::PERMISSIVE),
        ". gap (px (- 4f32))"
    );
}

#[test]
fn negative_spacing_classes_expand_inside_variants() {
    let tokens = expand("hover:-mt-2 active:-ml-[6px]", ClassMode::STRICT);
    assert!(
        tokens.contains(". hover (| __style | __style . mt (px (- 2f32)))"),
        "{tokens}"
    );
    assert!(
        tokens.contains(". active (| __style | __style . ml (px (- 6f32)))"),
        "{tokens}"
    );
}

#[test]
fn unsupported_negative_classes_explain_the_limitation() {
    let tokens = expand("-mx-auto -rounded-4 -m-foo", ClassMode::PERMISSIVE);
    assert!(
        tokens.contains("GPUI does not support `-mx-auto`"),
        "{tokens}"
    );
    assert!(
        tokens.contains("`auto` and `full` cannot be negative"),
        "{tokens}"
    );
    assert!(
        tokens.contains("GPUI does not support `-rounded-4`"),
        "{tokens}"
    );
    assert!(tokens.contains("negative values"), "{tokens}");
    assert!(!tokens.contains("compile_error"), "{tokens}");

    let strict = expand("-mx-auto", ClassMode::STRICT);
    assert!(strict.contains("compile_error"), "{strict}");
}

#[test]
fn negative_length_classes_never_pass_through_as_method_idents() {
    // A leading `-` cannot become a method name: negated length families that cannot
    // expand are reported, never passed through as `_mx_auto`-style calls.
    for class in ["-mx-auto", "-m-foo", "-rounded-4", "-w-full", "-m-"] {
        let tokens = expand(class, ClassMode::PERMISSIVE);
        assert!(
            tokens.contains("GPUI does not support"),
            "{class}: {tokens}"
        );
        assert!(!tokens.contains("passed through"), "{class}: {tokens}");
    }
}

#[test]
fn suggestions_cover_typos_prefixes_and_variants() {
    let mode = ClassMode::PERMISSIVE;
    assert_eq!(
        suggest_class("flex-colum", mode.clone()).as_deref(),
        Some("flex-col")
    );
    assert_eq!(
        suggest_class("bg-blu-500", mode.clone()).as_deref(),
        Some("bg-blue-500")
    );
    assert_eq!(
        suggest_class("gpa-4", mode.clone()).as_deref(),
        Some("gap-4")
    );
    assert_eq!(
        suggest_class("hovr:flex", mode.clone()).as_deref(),
        Some("hover:flex")
    );
    assert_eq!(suggest_class("zzzzzzzz", mode.clone()), None);
}

#[test]
fn unknown_identifier_classes_pass_through_with_typo_warning() {
    let tokens = expand("flex-colum", ClassMode::PERMISSIVE);
    assert!(tokens.contains(". flex_colum ()"));
    assert!(tokens.contains("Did you mean `flex-col`?"));

    // Extensions still work, but unknown vocabulary is never silently accepted.
    let tokens = expand("h-flex", ClassMode::PERMISSIVE);
    assert!(tokens.contains(". h_flex ()"));
    assert!(tokens.contains("explicit flag attribute `h_flex`"));
}

#[test]
fn digit_leading_classes_never_become_identifiers() {
    let tokens = expand("2xl 3d:flex", ClassMode::PERMISSIVE);
    assert!(tokens.contains("ignored unsupported class `2xl`"));
}

#[test]
fn mode_builders_only_change_the_selected_field() {
    let mode = ClassMode::PERMISSIVE
        .with_scale(SpacingScale::Tailwind)
        .with_dynamic_report(true);
    assert_eq!(mode.scale(), SpacingScale::Tailwind);
    assert!(mode.reports_dynamic_at_exit());
    assert!(!mode.is_strict());
    assert!(mode.theme().is_none());

    let strict = ClassMode::STRICT.with_theme(None);
    assert!(strict.is_strict());
    assert_eq!(strict.scale(), SpacingScale::Px);

    // `ClassMode` is cheaply cloneable, so helpers can take it by value.
    fn takes_by_value(mode: ClassMode) -> bool {
        mode.is_strict()
    }
    assert!(takes_by_value(ClassMode::STRICT));
}

#[test]
fn stable_class_mode_builder_exposes_the_supported_configuration() {
    let mode = ClassMode::builder()
        .strict(true)
        .scale(SpacingScale::Tailwind)
        .report_dynamic_at_exit(true)
        .build();

    assert!(mode.is_strict());
    assert_eq!(mode.scale(), SpacingScale::Tailwind);
    assert!(mode.reports_dynamic_at_exit());
    assert!(mode.theme().is_none());
}

#[test]
fn diagnostics_report_offsets() {
    let diagnostics = class_diagnostics("flex  dark:bg-black flex-colum", ClassMode::PERMISSIVE);
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].class, "dark:bg-black");
    assert_eq!(diagnostics[0].offset, 6);
    assert_eq!(diagnostics[1].suggestion.as_deref(), Some("flex-col"));
}

#[test]
fn color_rgba_matches_what_the_class_expands_to() {
    // Palette names, short/long hex and color functions all resolve to `0xRRGGBBAA`.
    let cases = [
        ("red-500", Some(0xef44_44ff)),
        ("black", Some(0x0000_00ff)),
        ("white", Some(0xffff_ffff)),
        ("[#f00]", Some(0xff00_00ff)),
        ("[#abc]", Some(0xaabb_ccff)),
        ("[#112233]", Some(0x1122_33ff)),
        ("[#11223344]", Some(0x1122_3344)),
        ("[#1234]", Some(0x1122_3344)),
        ("[rgb(1,2,3)]", Some(0x0102_03ff)),
        ("[rgba(1,2,3,0.5)]", Some(0x0102_0380)),
        ("[rgba(1,2,3,1)]", Some(0x0102_03ff)),
    ];
    for (color, expected) in cases {
        assert_eq!(color_rgba(color), expected, "{color}");
    }

    // Anything the macro rejects must not get a swatch either.
    for color in [
        "[#+123]",
        "[#12345]",
        "[#12345g]",
        "[rgba(1,2,3,1.5)]",
        "[rgba(1,2,3)]",
        "not-a-color",
        "red-1000",
        "",
    ] {
        assert_eq!(color_rgba(color), None, "{color}");
    }
}

#[test]
fn overflowing_lengths_are_never_emitted_as_infinite_literals() {
    // `1e38 * 4` overflows f32; the class must fall back to pass-through instead of
    // generating `px(inf)` (which `quote` cannot represent).
    let mode = ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind);
    let tokens = expand("w-1e38 gap-1e38", mode);
    assert!(tokens.contains(". w_1e38 ()"), "{tokens}");
    assert!(tokens.contains(". gap_1e38 ()"), "{tokens}");
    assert!(!tokens.contains("px ("), "{tokens}");

    // The same value stays valid without scaling.
    assert!(expand("w-1e38", ClassMode::PERMISSIVE).contains("px ("));
}

#[test]
fn fraction_ratio_out_of_range_reports_compile_error() {
    let tokens = expand("w-1e38/1e-38", ClassMode::PERMISSIVE);
    assert!(tokens.contains("value is out of range"), "{tokens}");
    assert!(tokens.contains("compile_error"), "{tokens}");
}

#[test]
fn suggestions_cover_theme_tokens_for_every_length_prefix() {
    static THEME: std::sync::OnceLock<std::sync::Arc<Theme>> = std::sync::OnceLock::new();
    let theme = THEME.get_or_init(|| {
        std::sync::Arc::new(
            Theme::parse("[colors]\naccent = \"#123456\"\n[spacing]\ngutter = \"12px\"\n").unwrap(),
        )
    });
    let mode = ClassMode::PERMISSIVE.with_theme(Some(std::sync::Arc::clone(theme)));

    assert_eq!(
        suggest_class("bg-acent", mode.clone()).as_deref(),
        Some("bg-accent")
    );
    assert_eq!(
        suggest_class("hover:bg-acent", mode.clone()).as_deref(),
        Some("hover:bg-accent")
    );
    for typo in ["p-guttr", "pt-guttr", "min-w-guttr", "gap-x-guttr"] {
        let expected = typo.replace("guttr", "gutter");
        assert_eq!(
            suggest_class(typo, mode.clone()).as_deref(),
            Some(expected.as_str())
        );
    }
    // Correct tokens need no suggestion for the color family.
    assert_eq!(suggest_class("bg-accent", mode.clone()), None);
    // Length prefixes are one edit apart from each other, so a valid `pt-gutter` may still
    // suggest `px-gutter`; the typo itself must always resolve to the right prefix.
    assert_eq!(
        suggest_class("pt-guttr", mode.clone()).as_deref(),
        Some("pt-gutter")
    );
}

#[test]
fn passthrough_messages_inside_variants_name_valid_rust() {
    let diagnostics = class_diagnostics("focus:ring-2 ring-2", ClassMode::PERMISSIVE);
    assert_eq!(diagnostics.len(), 2);
    let variant = &diagnostics[0].message;
    assert!(
        variant.contains("`.focus(|style| style.ring_2())`"),
        "{variant}"
    );
    assert!(!variant.contains(".focus:"), "{variant}");
    assert!(diagnostics[1].message.contains("`.ring_2()`"));
}

#[test]
fn negative_sizes_warn_that_layout_clamps_them_but_still_expand() {
    for class in [
        "-w-4",
        "-h-4",
        "-size-2",
        "-max-w-4",
        "-max-h-4",
        "-w-1/2",
        "-w-[10px]",
        "-w-px",
        "hover:-w-4",
        "w-[-4px]",
        "h--4",
        "size-[-2rem]",
        "max-w-[-50%]",
        "w-neg-px",
        "focus:w-[-4px]",
    ] {
        for mode in [ClassMode::PERMISSIVE, ClassMode::STRICT] {
            let tokens = expand(class, mode);
            assert!(
                tokens.contains("unsupported_rsx_class"),
                "{class}: {tokens}"
            );
            assert!(
                tokens.contains("clamps a negative size to 0"),
                "{class}: {tokens}"
            );
            // Still expanded, and never promoted to an error.
            assert!(!tokens.contains("compile_error"), "{class}: {tokens}");
        }
    }
    assert!(expand("-w-4", ClassMode::STRICT).contains(". w (px (- 4f32))"));
}

#[test]
fn negative_minimum_sizes_warn_that_layout_ignores_them() {
    for class in [
        "-min-w-4",
        "-min-h-4",
        "focus:-min-w-4",
        "min-w-[-4px]",
        "min-h--4",
        "focus:min-h-[-2rem]",
    ] {
        let tokens = expand(class, ClassMode::STRICT);
        assert!(
            tokens.contains("ignores a negative minimum size"),
            "{class}: {tokens}"
        );
        assert!(!tokens.contains("compile_error"), "{class}: {tokens}");
    }
}

#[test]
fn negative_margin_padding_gap_and_positive_sizes_do_not_warn() {
    for class in [
        "-m-2",
        "-mt-px",
        "-mx-[4px]",
        "-p-2",
        "-pl-1",
        "-gap-4",
        "-gap-x-2",
        "w-4",
        "min-w-4",
        "hover:-mt-2",
        "-w-[-4px]",
        "-size-[-2rem]",
        "-max-w-[-50%]",
        "-min-w-[-4px]",
        "-w--4",
        "-w-0",
        "w-[-0px]",
        "-min-w-0",
        "w-neg-0",
        "hover:-w-[-4px]",
    ] {
        let tokens = expand(class, ClassMode::STRICT);
        assert!(
            !tokens.contains("unsupported_rsx_class"),
            "{class}: {tokens}"
        );
    }
}

#[test]
fn negative_size_warnings_reach_lint_and_lsp_diagnostics() {
    let diagnostics = class_diagnostics("flex -w-4 -mt-2 hover:-min-h-1", ClassMode::STRICT);
    let found: Vec<_> = diagnostics
        .iter()
        .map(|d| (d.class.as_str(), d.offset, d.severity))
        .collect();
    assert_eq!(
        found,
        [
            ("-w-4", 5, Severity::Warning),
            ("hover:-min-h-1", 16, Severity::Warning)
        ]
    );
    assert!(diagnostics[0].message.contains("collapses"));
    assert!(diagnostics[0].suggestion.is_none());
}

#[test]
fn size_diagnostics_follow_resolved_signs_in_both_scales() {
    for scale in [SpacingScale::Px, SpacingScale::Tailwind] {
        for mode in [ClassMode::PERMISSIVE, ClassMode::STRICT] {
            let mode = mode.with_scale(scale);
            for (class, warns) in [
                ("w-[-4px]", true),
                ("hover:max-h-[-25%]", true),
                ("min-w-[-1rem]", true),
                ("-w-[-4px]", false),
                ("hover:-max-h-[-25%]", false),
                ("-min-w-[-1rem]", false),
                ("-w-0", false),
                ("w-[-0px]", false),
                ("-min-w-0", false),
            ] {
                let diagnostics = class_diagnostics(class, mode.clone());
                assert_eq!(
                    diagnostics.len(),
                    usize::from(warns),
                    "{class}: {diagnostics:?}"
                );
                if warns {
                    assert_eq!(diagnostics[0].severity, Severity::Warning);
                }
            }
        }
    }
}
