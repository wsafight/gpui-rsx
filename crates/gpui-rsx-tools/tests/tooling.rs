#![cfg(feature = "convert")]

use gpui_rsx_core::{ClassMode, SpacingScale};
use gpui_rsx_tools::{
    analyze_body, analyze_rust, analyze_rust_document, class_literal_ranges, convert, expand,
    format, macro_calls, mode_for, preview,
};

#[test]
fn scans_qualified_macros_without_matching_comments_or_strings() {
    let source = "// rsx!{ignored}\nfn f(){ let s=\"rsx!{}\"; let _=gpui_rsx::rsx!{<div class=\"dark:flex\"/>}; }";
    assert_eq!(macro_calls(source).unwrap().len(), 1);
    let ds = analyze_rust(source, ClassMode::PERMISSIVE);
    assert_eq!(ds.len(), 1);
    assert_eq!(&source[ds[0].start..ds[0].end], "dark:flex");
}

#[test]
fn document_analysis_matches_legacy_ranges_across_multiple_macro_modes() {
    let source = r#"fn f(){let _=rsx!{<div class="flex flex-colum" whenClass={(active, "items-center")}/>};let _=rsx_tw!{<div class="gap-4"/>};}"#;
    let analysis = analyze_rust_document(source, ClassMode::PERMISSIVE);
    let calls = macro_calls(source).unwrap();
    let mut legacy_ranges = Vec::new();
    for call in &calls {
        legacy_ranges.extend(
            class_literal_ranges(&source[call.start..call.end])
                .unwrap()
                .into_iter()
                .map(|(start, end)| (call.start + start, call.start + end)),
        );
    }

    assert_eq!(analysis.calls, calls);
    assert_eq!(analysis.literals, legacy_ranges);
    assert_eq!(
        analysis.diagnostics,
        analyze_rust(source, ClassMode::PERMISSIVE)
    );
    assert_eq!(analysis.calls.len(), 2);
    assert_eq!(analysis.literals.len(), 3);
    assert_eq!(analysis.diagnostics.len(), 1);
    let diagnostic = &analysis.diagnostics[0];
    assert_eq!(&source[diagnostic.start..diagnostic.end], "flex-colum");
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
fn jsx_conversion_drops_names_that_are_not_rust_identifiers() {
    let result = convert::convert(
        r#"const x = <my-el><button type="button" aria-label="Close" id="b">x</button></my-el>"#,
    )
    .unwrap();
    assert!(result.rsx.starts_with("<div>"));
    assert!(result.rsx.contains("id="));
    assert!(!result.rsx.contains("type"));
    assert!(!result.rsx.contains("aria-label"));
    assert_eq!(result.diagnostics.len(), 3);
    assert!(format::body(&result.rsx).is_ok());
}

#[test]
fn jsx_conversion_uses_tsx_parser_and_marks_unsupported_behavior() {
    let result=convert::convert(r#"export function Card({title}: {title:string}) {return <div className="flex gap-4 md:hidden" onClick={() => alert('x')}>Hi <span>{title}</span></div>}"#).unwrap();
    assert!(
        result.rsx.contains("gap-16"),
        "Tailwind units are rescaled to px"
    );
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

#[test]
fn formatter_keeps_expression_text_and_is_idempotent() {
    let source = "fn f(){\r\n    rsx!{<div class=\"flex\" on_click={cx.listener(|this, _, _, cx| {\r\n            this.n += 1;\r\n        })}>{items\r\n            .len()}</div>}\r\n}\r\n";
    let once = format::rust(source).unwrap();
    assert!(once.contains("cx.listener(|this, _, _, cx| {"));
    assert!(once.contains("class=\"flex\""));
    assert!(
        !once.replace("\r\n", "").contains('\n'),
        "CRLF must be preserved"
    );
    assert_eq!(format::rust(&once).unwrap(), once);
}

#[test]
fn formatter_skips_unparsable_and_multiline_literal_bodies() {
    let source = "fn f(){let _=rsx!{<div>};let _=rsx!{<p>{\"a\n b\"}</p>};let _=rsx!{<span class=\"p-1\"/>};}";
    let out = format::rust(source).unwrap();
    assert!(out.contains("rsx!{<div>}"));
    assert!(out.contains("rsx!{<p>{\"a\n b\"}</p>}"));
    assert!(out.contains("<span class=\"p-1\" />"));
}

#[test]
fn document_analysis_reports_unparsable_rust_and_bodies() {
    // A document that is not even a token stream becomes one error at the start.
    let analysis = analyze_rust_document("fn f( { rsx!{<div/>}; }", ClassMode::PERMISSIVE);
    assert!(analysis.calls.is_empty());
    assert!(analysis.literals.is_empty());
    assert_eq!(analysis.diagnostics.len(), 1);
    assert_eq!(analysis.diagnostics[0].severity, "error");
    assert_eq!(
        (analysis.diagnostics[0].start, analysis.diagnostics[0].end),
        (0, 1)
    );

    // A macro whose body does not parse is located at the macro and yields no literals.
    let source = "fn f(){rsx!{<div>};}";
    let analysis = analyze_rust_document(source, ClassMode::PERMISSIVE);
    assert_eq!(analysis.calls.len(), 1);
    assert!(analysis.literals.is_empty());
    assert_eq!(analysis.diagnostics.len(), 1);
    let diagnostic = &analysis.diagnostics[0];
    assert_eq!(diagnostic.severity, "error");
    assert!(
        diagnostic.message.contains("Unclosed tag"),
        "{diagnostic:?}"
    );
    assert_eq!(diagnostic.start, 12);
}

#[test]
fn class_literal_ranges_cover_when_class_and_state_class() {
    let source = r#"<div class="flex" whenClass={(a, "p-8")} hoverClass="m-4"/>"#;
    let ranges = class_literal_ranges(source).unwrap();
    let contents: Vec<_> = ranges.iter().map(|&(s, e)| &source[s..e]).collect();
    assert_eq!(contents, ["flex", "p-8", "m-4"]);
}

#[test]
fn escaped_literals_highlight_the_whole_literal_instead_of_bytes() {
    // `\n` has no one-to-one source mapping, so the diagnostic must cover the literal and
    // no literal range (which drives completions) may be published.
    let source = "fn f(){rsx!{<div class=\"a\\nb\"/>};}";
    let analysis = analyze_rust_document(source, ClassMode::PERMISSIVE);

    assert!(analysis.literals.is_empty());
    assert_eq!(analysis.diagnostics.len(), 2);
    for diagnostic in &analysis.diagnostics {
        assert_eq!(&source[diagnostic.start..diagnostic.end], "\"a\\nb\"");
        assert_eq!(diagnostic.suggestion, None);
    }
    assert_eq!(class_literal_ranges("<div class=\"a\\nb\"/>").unwrap(), []);
}

#[test]
fn formatter_emits_literal_attributes_without_braces() {
    assert_eq!(
        format::body(r#"<div class={"flex"} />"#).unwrap(),
        "<div class=\"flex\" />"
    );
    // Raw string literals keep their `r#"..."#` spelling.
    assert_eq!(
        format::body(r##"<div id={r#"a"#} />"##).unwrap(),
        "<div id=r#\"a\"# />"
    );
    // Non-literal expressions keep braces and are copied verbatim.
    assert_eq!(
        format::body("<div class={if a {\"flex\"} else {\"grid\"}} />").unwrap(),
        "<div class={if a {\"flex\"} else {\"grid\"}} />"
    );
    assert_eq!(
        format::body("<div class={c} />").unwrap(),
        "<div class={c} />"
    );
}

#[test]
fn formatter_refuses_comments_and_multiline_literals_without_touching_them() {
    let comment = format::body("<div /*x*/ />").unwrap_err();
    assert!(comment.contains("comments"), "{comment}");
    let multiline = format::body("<div>{\"a\nb\"}</div>").unwrap_err();
    assert!(
        multiline.contains("multi-line string literal"),
        "{multiline}"
    );

    // `rust` skips the offending macro but still formats the next one.
    let out = format::rust("fn f(){rsx!{<div /*x*/ />};rsx!{<p/>};}").unwrap();
    assert!(out.contains("rsx!{<div /*x*/ />}"));
    assert!(out.contains("rsx!{\n    <p />\n}"));
}

#[test]
fn migration_rewrites_every_class_attribute_but_skips_tailwind_macros() {
    let source = r#"fn f(){rsx!{<div class="gap-16" whenClass={(c, "p-8")} hoverClass="m-4"/>};rsx_tw!{<div class="gap-16"/>};}"#;
    let migrated = format::migrate_rust(source).unwrap();
    assert!(migrated.contains(r#"class="gap-4""#));
    assert!(migrated.contains(r#"whenClass={(c, "p-2")}"#));
    assert!(migrated.contains(r#"hoverClass="m-1""#));
    // `rsx_tw!` already uses the Tailwind scale, so it must not be migrated twice.
    assert!(migrated.contains(r#"rsx_tw!{<div class="gap-16"/>}"#));
    // Dynamic expressions are left for manual review and don't fail the migration.
    let dynamic = "fn f(){rsx!{<div class={c}/>};}";
    assert_eq!(format::migrate_rust(dynamic).unwrap(), dynamic);
    // A body that does not parse is an error, not a silent skip.
    assert!(format::migrate_rust("fn f(){rsx!{<div>};}").is_err());
}

#[test]
fn conversion_handles_fragments_multiple_roots_and_placeholders() {
    let fragment = convert::convert("const a = <><div/></>;").unwrap();
    assert_eq!(fragment.rsx, "<>\n    <div />\n</>");
    assert!(fragment.diagnostics.is_empty());

    let roots = convert::convert("const a = <div/>; const b = <span/>;").unwrap();
    assert_eq!(roots.rsx, "<>\n    <div />\n    <span />\n</>");

    let member = convert::convert("const a = <Foo.Bar/>;").unwrap();
    assert_eq!(member.rsx, "<div />");
    assert_eq!(member.diagnostics.len(), 1);
    assert!(
        member.diagnostics[0].message.contains("member component"),
        "{:?}",
        member.diagnostics
    );

    let spread = convert::convert("const a = <div>{...props}</div>;").unwrap();
    assert!(spread.rsx.contains("TODO: spread children"));
    assert_eq!(spread.diagnostics.len(), 1);
}

#[test]
fn preview_uses_the_macro_color_parser() {
    let html = preview::render(
        r#"<div class="bg-red-500 text-[#abc] border-[#11223344] bg-[rgba(1,2,3,0.5)]"/>"#,
        ClassMode::PERMISSIVE,
    )
    .unwrap();
    // Palette colors get an opaque alpha byte; hex and rgba keep the alpha they declare.
    assert!(html.contains("background-color:#ef4444ff"), "{html}");
    assert!(html.contains("color:#aabbccff"), "{html}");
    assert!(html.contains("border-color:#11223344"), "{html}");
    assert!(html.contains("background-color:#01020380"), "{html}");
}

#[test]
fn preview_marks_loops_spreads_and_dynamic_classes_as_partial() {
    let looped = preview::render(
        "<ul>{for x in xs {<li>{x}</li>}}</ul>",
        ClassMode::PERMISSIVE,
    )
    .unwrap();
    assert!(looped.contains("loop template"), "{looped}");
    assert!(looped.contains("data-rsx-tag=\"li\""), "{looped}");

    let spread = preview::render("<div>{...rows}</div>", ClassMode::PERMISSIVE).unwrap();
    assert!(spread.contains("{children}"), "{spread}");

    let dynamic = preview::render("<div class={c} id=\"x\"/>", ClassMode::PERMISSIVE).unwrap();
    assert!(dynamic.contains("dynamic class"), "{dynamic}");
    assert!(dynamic.contains("attribute/behavior"), "{dynamic}");

    // Text children are escaped, and parse errors are returned rather than rendered.
    let escaped = preview::render("<div>{\"a<b\"}</div>", ClassMode::PERMISSIVE).unwrap();
    assert!(escaped.contains("a&lt;b"), "{escaped}");
    assert!(preview::render("<div>", ClassMode::PERMISSIVE).is_err());
}

#[test]
fn preview_page_embeds_refresh_script_only_in_watch_mode() {
    let page = preview::page("<b>content</b>", false);
    assert!(page.starts_with("<!doctype html>"));
    assert!(page.contains("<main id=\"preview\"><b>content</b></main>"));
    assert!(!page.contains("setInterval"));

    let watching = preview::page("", true);
    assert!(watching.contains("setInterval"));
    assert!(watching.contains("/content"));
    assert_eq!(preview::escape("a&<>\"'"), "a&amp;&lt;&gt;&quot;&#39;");
}

#[test]
fn mode_for_derives_per_macro_options() {
    let base = ClassMode::PERMISSIVE;
    assert_eq!(
        mode_for("rsx_tw", base.clone()).scale,
        SpacingScale::Tailwind
    );
    assert_eq!(mode_for("rsx", base.clone()).scale, SpacingScale::Px);
    assert!(mode_for("rsx_strict", base.clone()).strict);
    // `rsx_permissive!` opts out of an enclosing strict mode.
    assert!(!mode_for("rsx_permissive", ClassMode::STRICT).strict);
    assert!(mode_for("rsx", ClassMode::STRICT).strict);
}

#[test]
fn releasing_span_memory_between_documents_keeps_offsets_stable() {
    let source = "fn f(){rsx!{<div class=\"flex-colum\"/>};}";
    let before = analyze_rust_document(source, ClassMode::PERMISSIVE);
    gpui_rsx_tools::release_span_memory();
    let after = analyze_rust_document(source, ClassMode::PERMISSIVE);

    assert_eq!(before.calls, after.calls);
    assert_eq!(before.literals, after.literals);
    assert_eq!(before.diagnostics, after.diagnostics);
    assert_eq!(
        &source[after.diagnostics[0].start..after.diagnostics[0].end],
        "flex-colum"
    );

    // Repeating the cycle must not drift the offsets, and other entry points stay usable
    // after the source map is dropped.
    for _ in 0..5 {
        let analysis = analyze_rust_document(source, ClassMode::PERMISSIVE);
        assert_eq!(analysis.diagnostics, before.diagnostics);
        gpui_rsx_tools::release_span_memory();
    }
    assert!(format::rust(source).unwrap().contains("rsx!{"));
    assert!(expand("<div/>", ClassMode::PERMISSIVE).is_ok());
}

#[test]
fn formatter_rebases_multiline_expressions_onto_the_template_indent() {
    let source = "impl Foo {\n    fn render(&self) -> impl IntoElement {\n        rsx! {\n            <div on_click={cx.listener(|this, _, _, cx| {\n                this.n += 1;\n            })}>{\n                items\n                    .len()\n            }</div>\n        }\n    }\n}\n";
    let once = format::rust(source).unwrap();
    assert!(
        once.contains("            <div on_click={cx.listener(|this, _, _, cx| {\n                this.n += 1;\n            })}>"),
        "{once}"
    );
    assert!(
        once.contains("                {items\n                    .len()}"),
        "{once}"
    );
    assert_eq!(
        format::rust(&once).unwrap(),
        once,
        "not idempotent:\n{once}"
    );

    // Tabs in the surrounding Rust keep working and the body still re-indents.
    let tabs = "fn f(){\n\trsx!{<div>\n\t\t<span/>\n\t</div>}\n}\n";
    let formatted = format::rust(tabs).unwrap();
    assert!(
        formatted.contains("\trsx!{\n\t    <div>\n\t        <span />"),
        "{formatted}"
    );
    assert_eq!(format::rust(&formatted).unwrap(), formatted);
}

#[test]
fn formatter_reformats_fragments_and_preserves_expression_spacing() {
    let fragment = "fn f(){\n    rsx! {<>\n<div/><span/></>}\n}\n";
    assert_eq!(
        format::rust(fragment).unwrap(),
        "fn f(){\n    rsx! {\n        <>\n            <div />\n            <span />\n        </>\n    }\n}\n"
    );

    // Expression text is copied from the source, not re-printed token by token.
    let spaced = format::rust("fn f(){rsx!{<div>{a   +   b}</div>}}\n").unwrap();
    assert!(spaced.contains("{a   +   b}"), "{spaced}");
}

#[test]
fn formatter_handles_every_attribute_kind() {
    let body = format::body(
        r#"<div id="a" onClick={h} when={(c, |el| el.flex())} whenSome={(o, |el, v| el.w(v))} whenClass={(c, "bg-red-500")} hoverClass="bg-blue-500" styled>{for x in xs {<li key={x.id}>{x.name}</li>}}</div>"#,
    )
    .unwrap();
    assert_eq!(
        body,
        "<div id=\"a\" onClick={h} when={(c, |el| el.flex())} whenSome={(o, |el, v| el.w(v))} whenClass={(c, \"bg-red-500\")} hoverClass=\"bg-blue-500\" styled>\n    {for x in xs {\n        <li key={x.id}>\n            {x.name}\n        </li>\n    }}\n</div>"
    );
    // Formatting a formatted body is a no-op.
    assert_eq!(format::body(&body).unwrap(), body);
}

#[test]
fn conversion_notes_every_unsupported_jsx_construct() {
    let cases = [
        (
            "const a = <div {...props}/>;",
            "JSX spread props require manual conversion",
        ),
        (
            "const a = <div xlink:href=\"x\"/>;",
            "Namespaced attribute requires manual conversion",
        ),
        (
            "const a = <input disabled />;",
            "Boolean prop `disabled` needs a GPUI method; omitted",
        ),
        (
            "const a = <div title={title}/>;",
            "Expression prop `title` needs manual Rust conversion; omitted",
        ),
        (
            "const a = <div>{name}</div>;",
            "JavaScript expression replaced with a text placeholder",
        ),
    ];
    for (source, expected) in cases {
        let result = convert::convert(source).unwrap();
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.starts_with(expected)),
            "{source}: {:?}",
            result.diagnostics
        );
        // Whatever is emitted must still parse as RSX.
        assert!(
            format::body(&result.rsx).is_ok(),
            "{source}: {}",
            result.rsx
        );
    }
}

#[test]
fn conversion_keeps_string_and_number_children_and_normalizes_classes() {
    let result = convert::convert("const a = <div>{\"str\"}{42}<span>hi</span></div>;").unwrap();
    assert_eq!(
        result.rsx,
        "<div>\n    {\"str\"}\n    {\"42\"}\n    <span>\n        {\"hi\"}\n    </span>\n</div>"
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);

    // Whitespace inside classes is normalized and Tailwind units rescaled.
    let classes = convert::convert("const a = <div class=\"flex  gap-4\"/>;").unwrap();
    assert_eq!(classes.rsx, "<div class=\"flex gap-16\" />");

    // Unsupported classes are dropped with a note instead of producing invalid RSX.
    let dropped = convert::convert("const a = <div class=\"flex -mt-4 md:hidden\"/>;").unwrap();
    assert_eq!(dropped.rsx, "<div class=\"flex\" />");
    assert_eq!(dropped.diagnostics.len(), 2);
}

#[test]
fn preview_renders_fragments_and_translates_every_length_prefix() {
    // A fragment renders its children without a wrapper element.
    let fragment = preview::render("<><div/><span/></>", ClassMode::PERMISSIVE).unwrap();
    assert_eq!(
        fragment,
        "<div data-rsx-tag=\"div\" style=\"\"></div><div data-rsx-tag=\"span\" style=\"\"></div>"
    );

    let html = preview::render(
        r#"<div class="min-w-4 max-h-8 size-2 gap-x-1 gap-y-2 mx-1 my-2 mt-1 mb-2 ml-3 mr-4 px-1 py-2 pt-1 pb-2 pl-3 pr-4 w-1/2 h-[14px] opacity-50"/>"#,
        ClassMode::PERMISSIVE,
    )
    .unwrap();
    for declaration in [
        "min-width:4px",
        "max-height:8px",
        "width:2px;height:2px",
        "column-gap:1px",
        "row-gap:2px",
        "margin-left:1px;margin-right:1px",
        "margin-top:1px",
        "margin-left:3px",
        "margin-right:4px",
        "padding-left:1px;padding-right:1px",
        "padding-top:1px",
        "padding-bottom:2px",
        "padding-left:3px",
        "padding-right:4px",
        "width:50%",
        "height:14px",
        "opacity:0.5",
    ] {
        assert!(html.contains(declaration), "missing {declaration}: {html}");
    }

    // The Tailwind scale applies to previews too.
    let scaled = preview::render(
        "<div class=\"p-4\"/>",
        ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind),
    )
    .unwrap();
    assert!(scaled.contains("padding:16px"), "{scaled}");
}

#[test]
fn preview_leaves_unknown_classes_out_of_the_style_attribute() {
    let html = preview::render(
        "<div class=\"flex unknown-class dark:flex\"/>",
        ClassMode::PERMISSIVE,
    )
    .unwrap();
    assert!(html.contains("display:flex"), "{html}");
    assert!(!html.contains("unknown-class;"), "{html}");
    assert!(html.contains("partial preview"), "{html}");
}

#[test]
fn conversion_collapses_jsx_text_whitespace() {
    let single = convert::convert("const a = <div>hi</div>;").unwrap();
    assert_eq!(single.rsx, "<div>\n    {\"hi\"}\n</div>");

    // Leading/trailing whitespace per line is trimmed and lines join with one space.
    let multiline = convert::convert("const a = <div>\n    a\n    b\n    c\n</div>;").unwrap();
    assert_eq!(multiline.rsx, "<div>\n    {\"a b c\"}\n</div>");

    // Blank lines and indentation-only text produce nothing.
    let blank = convert::convert("const a = <div>\n   \n</div>;").unwrap();
    assert_eq!(blank.rsx, "<div />");

    // A module with no JSX at all is an error, not empty output.
    let error = convert::convert("let x = 1;").unwrap_err();
    assert!(error.contains("No JSX element found"), "{error}");
}

#[test]
fn analysis_walks_fragments_and_nested_loops() {
    let source = "fn f(){rsx!{<><div class=\"flex-colum\"/>{for x in xs {<li class=\"flex-colun\"/>{...x}}}</>};}";
    let analysis = analyze_rust_document(source, ClassMode::PERMISSIVE);
    let classes: Vec<_> = analysis
        .literals
        .iter()
        .map(|&(start, end)| &source[start..end])
        .collect();
    assert_eq!(classes, ["flex-colum", "flex-colun"]);
    assert_eq!(analysis.diagnostics.len(), 2);
}

#[test]
fn literal_string_only_accepts_string_literals() {
    let literal = |source: &str| {
        let expr: syn::Expr = syn::parse_str(source).unwrap();
        gpui_rsx_tools::literal_string(&expr)
    };
    assert_eq!(literal("\"hello\"").as_deref(), Some("hello"));
    assert_eq!(literal("42"), None);
    assert_eq!(literal("name"), None);
}

#[test]
fn formatter_keeps_blank_lines_out_of_the_indented_body() {
    // A blank line inside a body must not gain trailing indentation.
    let source = "fn f(){rsx!{<div>\n\n<span/>\n</div>};}\n";
    let formatted = format::rust(source).unwrap();
    assert!(!formatted.contains("    \n"), "{formatted:?}");
    assert_eq!(format::rust(&formatted).unwrap(), formatted);
}

#[test]
fn formatter_renders_spread_children_verbatim() {
    assert_eq!(
        format::body("<ul>{...rows}</ul>").unwrap(),
        "<ul>\n    {...rows}\n</ul>"
    );
    // A spread inside a loop keeps its indentation relative to the loop body.
    assert_eq!(
        format::body("<ul>{for row in rows {<li/>{...row}}}</ul>").unwrap(),
        "<ul>\n    {for row in rows {\n        <li />\n        {...row}\n    }}\n</ul>"
    );
}
