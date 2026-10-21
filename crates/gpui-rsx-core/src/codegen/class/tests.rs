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
    assert_eq!(expand("gap-4", mode), ". gap (px (16f32))");
    assert_eq!(expand("p-2.5", mode), ". p (px (10f32))");
    assert_eq!(expand("w-64", mode), ". w (px (256f32))");
    assert_eq!(expand("gap-[14px]", mode), ". gap (px (14f32))");
    assert_eq!(expand("gap-4", ClassMode::PERMISSIVE), ". gap (px (4f32))");
}

#[test]
fn suggestions_cover_typos_prefixes_and_variants() {
    let mode = ClassMode::PERMISSIVE;
    assert_eq!(
        suggest_class("flex-colum", mode).as_deref(),
        Some("flex-col")
    );
    assert_eq!(
        suggest_class("bg-blu-500", mode).as_deref(),
        Some("bg-blue-500")
    );
    assert_eq!(suggest_class("gpa-4", mode).as_deref(), Some("gap-4"));
    assert_eq!(
        suggest_class("hovr:flex", mode).as_deref(),
        Some("hover:flex")
    );
    assert_eq!(suggest_class("zzzzzzzz", mode), None);
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
fn diagnostics_report_offsets() {
    let diagnostics = class_diagnostics("flex  dark:bg-black flex-colum", ClassMode::PERMISSIVE);
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].class, "dark:bg-black");
    assert_eq!(diagnostics[0].offset, 6);
    assert_eq!(diagnostics[1].suggestion.as_deref(), Some("flex-col"));
}
