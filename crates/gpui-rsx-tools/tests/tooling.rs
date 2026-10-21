use gpui_rsx_core::{ClassMode, SpacingScale};
use gpui_rsx_tools::{analyze_body, analyze_rust, convert, expand, format, macro_calls, preview};

#[test]
fn scans_qualified_macros_without_matching_comments_or_strings() {
    let source = "// rsx!{ignored}\nfn f(){ let s=\"rsx!{}\"; let _=gpui_rsx::rsx!{<div class=\"dark:flex\"/>}; }";
    assert_eq!(macro_calls(source).unwrap().len(), 1);
    let ds = analyze_rust(source, ClassMode::PERMISSIVE);
    assert_eq!(ds.len(), 1);
    assert_eq!(&source[ds[0].start..ds[0].end], "dark:flex");
}

#[test]
fn unicode_and_raw_literal_positions_are_exact() {
    let source = "fn f(){let _=rsx!{<div>{\"中文😀\"}<div class=r#\"flex flex-colum\"#/></div>};}";
    let ds = analyze_rust(source, ClassMode::PERMISSIVE);
    assert_eq!(ds.len(), 1);
    assert_eq!(
        &source[ds[0].start..ds[0].end],
        "flex-colum",
        "{ds:?}; calls={:?}",
        macro_calls(source)
    );
    assert_eq!(ds[0].suggestion.as_deref(), Some("flex-col"));
}

#[test]
fn diagnoses_static_branches_and_reports_dynamic_limits() {
    let source = r#"<div><div class={if active {"dark:flex"} else {"flex-colum"}}/><div class={classes}/></div>"#;
    let ds = analyze_body(source, ClassMode::PERMISSIVE);
    assert_eq!(ds.len(), 3);
    assert!(ds.iter().any(|d| d.severity == "information"));
}

#[test]
fn semantic_errors_and_strict_macro_modes_are_reported() {
    let ds = analyze_rust(
        r#"fn f(){rsx_strict!{<div class="nonsense"/>};}"#,
        ClassMode::PERMISSIVE,
    );
    assert!(ds.iter().any(|d| d.severity == "error"));
    let ds = analyze_body(
        r#"<ul>{for x in xs {<li onClick={handler}/>} }</ul>"#,
        ClassMode::PERMISSIVE,
    );
    assert!(ds.iter().any(|d| d.message.contains("key")));
}

#[test]
fn formatting_is_idempotent_and_preserves_comments_and_string_contents() {
    let source = "// keep\nfn f(){ let a=rsx!{<div class=\"flex\"><span>{\"https://example.com\"}</span></div>};let b=rsx!{ /*keep*/ <div/>}; }";
    let formatted = format::rust(source).unwrap();
    assert!(formatted.contains("/*keep*/"));
    assert!(formatted.starts_with("// keep\nfn f(){"));
    assert_eq!(format::rust(&formatted).unwrap(), formatted);
    let loops = r#"<><ul>{for item in items {<li key={item.id}>{item.name}</li>}}</ul></>"#;
    assert!(format::body(loops).is_ok());
}

#[test]
fn migration_preserves_expanded_spacing_and_skips_tailwind_macros() {
    let source = r#"fn f(){rsx!{<div class="gap-16 hover:p-8 w-[280px] h-1/2"/>};rsx_tw!{<div class="gap-16"/>};}"#;
    let migrated = format::migrate_rust(source).unwrap();
    assert!(migrated.contains("gap-4 hover:p-2 w-[280px] h-1/2"));
    assert!(migrated.contains("rsx_tw!{<div class=\"gap-16\"/>}"));
    let old = expand(r#"<div class="gap-16 p-8"/>"#, ClassMode::PERMISSIVE).unwrap();
    let new = expand(
        r#"<div class="gap-4 p-2"/>"#,
        ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind),
    )
    .unwrap();
    assert_eq!(old, new);
}

#[test]
fn jsx_conversion_uses_tsx_parser_and_marks_unsupported_behavior() {
    let result=convert::convert(r#"export function Card({title}: {title:string}) {return <div className="flex gap-4 md:hidden" onClick={() => alert('x')}>Hi <span>{title}</span></div>}"#).unwrap();
    assert!(result.rsx.contains("gap-4"));
    assert!(!result.rsx.contains("md:hidden"));
    assert!(result.rsx.contains("TODO: JavaScript expression"));
    assert_eq!(result.diagnostics.len(), 3);
    assert!(
        analyze_body(&result.rsx, ClassMode::STRICT)
            .iter()
            .all(|d| d.severity != "error")
    );
    assert!(convert::convert("<div>").is_err());
}

#[test]
fn preview_escapes_untrusted_text_and_never_executes_expressions() {
    let html = preview::render(
        r#"<div class="gap-4">{"<script>alert(1)</script>"}{dangerous()}<MyWidget/></div>"#,
        ClassMode::PERMISSIVE,
    )
    .unwrap();
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("dangerous()"));
    assert!(html.contains("component: MyWidget"));
}

#[test]
fn class_tools_do_not_rewrite_business_logic_or_runtime_format_strings() {
    let source = r#"fn f(){rsx!{<div class={if name == "gap-16" {"p-8"} else {"gap-4"}}/>};rsx!{<div class={format!("gap-{}",n)}/>};}"#;
    let migrated = format::migrate_rust(source).unwrap();
    assert!(migrated.contains(r#"name == "gap-16""#));
    assert!(migrated.contains(r#"{"p-2"} else {"gap-1"}"#));
    assert!(migrated.contains(r#"format!("gap-{}",n)"#));
    let ds = analyze_rust(source, ClassMode::PERMISSIVE);
    assert_eq!(ds.len(), 1);
    assert_eq!(ds[0].severity, "information");
}
