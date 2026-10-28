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
