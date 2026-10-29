//! Assert actual GPUI refinements, rather than only inspecting expanded Rust.
use gpui::{
    BoxShadow, Div, Element, FontWeight, SharedString, Styled, div, hsla, img, prelude::*, px,
    relative, rems, rgb, rgba, svg,
};
use gpui_rsx::{rsx, rsx_expand, rsx_permissive, rsx_strict, rsx_tw};
use std::{borrow::Cow, cell::Cell};

fn same_style(mut actual: impl Styled, mut expected: impl Styled) {
    assert_eq!(
        format!("{:?}", actual.style()),
        format!("{:?}", expected.style())
    );
}

#[test]
fn paired_elements_flags_and_value_aliases_match_gpui() {
    let actual: Div = rsx! {
        <section flex flexCol itemsCenter gap={px(7.)} backgroundColor={rgb(0x123456)}
            textColor={rgb(0xffffff)} fontSize={px(14.)} minWidth={px(20.)} maxHeight={px(80.)}></section>
    };
    let expected = div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(7.))
        .bg(rgb(0x123456))
        .text_color(rgb(0xffffff))
        .text_size(px(14.))
        .min_w(px(20.))
        .max_h(px(80.));
    same_style(actual, expected);
    same_style(rsx! { <div /> }, rsx! { <div></div> });
}

#[test]
fn styled_defaults_apply_before_explicit_overrides() {
    same_style(
        rsx! { <h1 styled class="text-xl font-normal" /> },
        div().text_xl().font_weight(FontWeight::NORMAL),
    );
    same_style(
        rsx! { <input styled class="px-7" /> },
        div().px(px(7.)).py(px(1.)),
    );
    same_style(
        rsx! { <form styled /> },
        div().flex().flex_col().gap(px(4.)),
    );
    same_style(rsx! { <li styled /> }, div().flex().items_center());
    same_style(rsx! { <button styled /> }, div().cursor_pointer());
}

#[test]
fn numeric_arbitrary_fraction_and_grid_values_match_gpui() {
    let actual = rsx_strict! {
        <div class="p-2.5 mx-3 w-1/3 h-[75%] min-w-[2rem] max-h-[37px] rounded-[4px] text-[14px] leading-20 opacity-35 grid-cols-3 col-span-2 row-start-2" />
    };
    let expected = div()
        .p(px(2.5))
        .mx(px(3.))
        .w(relative(1. / 3.))
        .h(relative(0.75))
        .min_w(rems(2.))
        .max_h(px(37.))
        .rounded(px(4.))
        .text_size(px(14.))
        .line_height(px(20.))
        .opacity(0.35)
        .grid_cols(3)
        .col_span(2)
        .row_start(2);
    same_style(actual, expected);
}

#[test]
fn colors_accept_palette_hex_rgb_rgba_and_opacity() {
    same_style(
        rsx! { <div class="bg-red-500/50 text-[#0f0] border-[rgba(1,2,3,0.5)]" /> },
        div()
            .bg(rgba(0xef444480))
            .text_color(rgb(0x00ff00))
            .border_color(rgba(0x01020380)),
    );
    same_style(
        rsx! { <div class="bg-[#1234]/25 text-[rgb(1,2,3)]" /> },
        div().bg(rgba(0x11223340)).text_color(rgb(0x010203)),
    );
}

#[test]
fn conditional_classes_cover_all_branches_and_match_guards() {
    for enabled in [false, true] {
        same_style(
            rsx! { <div class={if enabled { "p-8" } else { "p-2" }} /> },
            div().p(px(if enabled { 8. } else { 2. })),
        );
        same_style(
            rsx! { <div class={if let Some(n) = enabled.then_some(4) { if n > 2 { "h-8" } else { "h-4" } } else { "h-2" }} /> },
            div().h(px(if enabled { 8. } else { 2. })),
        );
    }
    for (value, width) in [(None, 10.), (Some(1), 20.), (Some(4), 40.)] {
        same_style(
            rsx! { <div class={match value { Some(n) if n > 2 => "w-40", Some(_) => "w-20", None => "w-10" }} /> },
            div().w(px(width)),
        );
    }
    same_style(rsx! { <div class={("p-3")} /> }, div().p(px(3.)));
    same_style(rsx! { <div class={{ "p-3" }} /> }, div().p(px(3.)));
}

#[test]
fn conditions_and_visibility_evaluate_once_and_preserve_order() {
    for enabled in [false, true] {
        let calls = Cell::new(0);
        let mut actual = rsx! {
            <div class="w-10" when={(enabled, |el| el.w(px(20.)))}
                whenSome={(enabled.then_some(30.), |el, n| el.w(px(n)))}
                whenClass={(enabled, "w-40")}
                visible={{ calls.set(calls.get() + 1); enabled }} />
        };
        let mut expected = div().w(px(if enabled { 40. } else { 10. }));
        expected = if enabled {
            expected.visible()
        } else {
            expected.invisible()
        };
        assert_eq!(calls.get(), 1);
        assert_eq!(actual.style().visibility, expected.style().visibility);
        same_style(actual, expected);
    }
}

#[test]
fn dynamic_strings_accept_str_string_cow_and_shared_string() {
    let class = "p-7\tbg-blue-500\nrounded-[3px]";
    let expected = || div().p(px(7.)).bg(rgb(0x3b82f6)).rounded(px(3.));
    same_style(rsx! { <div class={class} /> }, expected());
    same_style(rsx! { <div class={String::from(class)} /> }, expected());
    same_style(rsx! { <div class={Cow::Borrowed(class)} /> }, expected());
    same_style(
        rsx! { <div class={SharedString::from(class)} /> },
        expected(),
    );
}

#[test]
fn dynamic_and_static_values_apply_the_same_refinements() {
    let class = "p-2.5 w-1/3 h-[75%] min-w-[2rem] rounded-[4px] text-[14px] leading-20 bg-red-500/50 opacity-35 grid-cols-3";
    same_style(
        rsx_strict! { <div class={class} /> },
        rsx_strict! {
            <div class="p-2.5 w-1/3 h-[75%] min-w-[2rem] rounded-[4px] text-[14px] leading-20 bg-red-500/50 opacity-35 grid-cols-3" />
        },
    );
}

#[test]
fn macro_modes_preserve_their_spacing_contract() {
    same_style(rsx! { <div class="p-4" /> }, div().p(px(4.)));
    same_style(rsx_permissive! { <div class="p-4" /> }, div().p(px(4.)));
    same_style(rsx_strict! { <div class="p-4" /> }, div().p(px(4.)));
    same_style(
        rsx_tw! { <div class="p-4 w-[8px]" /> },
        div().p(px(16.)).w(px(8.)),
    );
    let class = "p-4 w-[8px]";
    same_style(
        rsx_tw! { <div class={class} /> },
        div().p(px(16.)).w(px(8.)),
    );
}

#[test]
fn theme_tokens_resolve_through_the_demo_configuration() {
    same_style(
        rsx! { <div class="text-size-body leading-body opacity-muted shadow-card" /> },
        div()
            .text_size(px(14.))
            .line_height(px(20.))
            .opacity(0.6)
            .shadow(vec![gpui::BoxShadow::new(
                px(0.),
                px(1.),
                gpui::hsla(0., 0., 0., 0.2),
            )]),
    );
}

#[test]
fn literal_and_conditional_state_classes_receive_ids() {
    assert!(Element::id(&rsx! { <div class="active:opacity-75" /> }).is_some());
    for selected in [false, true] {
        assert!(
            Element::id(
                &rsx! { <div class={if selected { "active:opacity-75" } else { "flex" }} /> }
            )
            .is_some()
        );
    }
    let suffix = "p-2";
    assert!(Element::id(&rsx! { <div class={format!("active:opacity-75 {suffix}")} /> }).is_some());
    let _: gpui::Stateful<Div> = rsx! { <div activeClass="opacity-75" /> };
}

#[test]
fn dynamic_unknown_handlers_receive_only_unsupported_tokens_in_order() {
    let class = "flex unsupported dark:bg-black hover:underline bg-red-500/50/50";
    let mut unknown = Vec::new();
    same_style(
        rsx_strict! { <div class={class} onUnknownClass={|c: &str| unknown.push(c.to_owned())} /> },
        div().flex(),
    );
    assert_eq!(
        unknown,
        [
            "unsupported",
            "dark:bg-black",
            "hover:underline",
            "bg-red-500/50/50"
        ]
    );
    for empty in ["", " \t\n"] {
        same_style(
            rsx! { <div class="border-2" class={empty} /> },
            div().border_2(),
        );
    }
}

#[test]
#[should_panic(expected = "unsupported dynamic class")]
fn strict_dynamic_classes_panic_without_an_override() {
    let class = "definitely-unknown";
    let _ = rsx_strict! { <div class={class} /> };
}

#[test]
fn expand_macro_returns_rust_without_evaluating_expressions() {
    let expanded = rsx_expand! { <div flex gap={unresolved_function()}><span>"child"</span></div> };
    assert!(
        expanded
            .split_whitespace()
            .collect::<String>()
            .contains("div().flex().gap(unresolved_function()).child(div().child(\"child\"))")
    );
}

#[test]
fn every_html_alias_constructs_a_plain_div() {
    macro_rules! aliases {
        ($($tag:ident),* $(,)?) => {$(
            let element: Div = rsx! { <$tag /> };
            assert!(Element::id(&element).is_none(), stringify!($tag));
        )*};
    }
    aliases!(
        div, span, section, article, header, footer, main, nav, aside, h1, h2, h3, h4, h5, h6, p,
        label, a, button, input, textarea, select, form, ul, ol, li
    );
}

#[test]
fn dynamic_numeric_overflow_is_reported_without_modifying_styles() {
    let class = "w-1e38 h-1/1e-40";
    let mut unknown = Vec::new();
    same_style(
        rsx_tw! { <div class={class} onUnknownClass={|c: &str| unknown.push(c.to_owned())} /> },
        div(),
    );
    assert_eq!(unknown, ["w-1e38", "h-1/1e-40"]);
}

#[test]
fn shadowed_format_macros_keep_their_actual_output() {
    macro_rules! format {
        ($template:literal, $value:expr) => {
            String::from("p-3")
        };
    }
    same_style(
        rsx! { <div class={format!("p-9 {}", 1)} /> },
        div().p(px(3.)),
    );
    // Equal byte lengths must not apply a static prefix from an unrelated macro.
    macro_rules! format {
        ($template:literal, $value:expr) => {
            String::from("p-3 p-4")
        };
    }
    same_style(
        rsx! { <div class={format!("p-9 {}", 1)} /> },
        div().p(px(4.)),
    );
}

#[test]
fn grid_integer_classes_match_dynamic_values_including_negative_lines() {
    let class = "grid-cols-12 grid-rows-3 col-span-2 row-span-5 col-start--1 col-end-4 row-start--2 row-end-6 line-clamp-4";
    same_style(
        rsx_strict! { <div class={class} /> },
        rsx_strict! {
            <div class="grid-cols-12 grid-rows-3 col-span-2 row-span-5 col-start--1 col-end-4 row-start--2 row-end-6 line-clamp-4" />
        },
    );
    let class = "grid-cols-70000 col-span--1 row-start-40000";
    let mut unknown = Vec::new();
    same_style(
        rsx! { <div class={class} onUnknownClass={|c: &str| unknown.push(c.to_owned())} /> },
        div(),
    );
    assert_eq!(
        unknown,
        ["grid-cols-70000", "col-span--1", "row-start-40000"]
    );
}

#[test]
fn nested_iterators_can_borrow_outer_patterns_and_mutable_caller_state() {
    let groups = [vec![1, 2], vec![3]];
    let mut visited = Vec::new();
    let children = rsx! {
        <>{for (index, row) in groups.iter().enumerate() {
            {for cell in row {
                <div id={{ visited.push((index, *cell)); format!("{index}-{cell}") }} />
            }}
        }}</>
    };
    assert_eq!(visited, [(0, 1), (0, 2), (1, 3)]);
    assert_eq!(children.len(), 3);

    visited.clear();
    let children = rsx! {
        <>{for index in 0..2 {
            {...(0..2).map(|cell| {
                visited.push((index, cell));
                rsx! { <div id={format!("{index}-{cell}")} /> }.into_any_element()
            })}
        }}</>
    };
    assert_eq!(visited, [(0, 0), (0, 1), (1, 0), (1, 1)]);
    assert_eq!(children.len(), 4);
}

#[test]
fn special_image_tags_use_real_constructors_and_sources() {
    let source = gpui_rsx_demo::syntax::sample_image();
    same_style(
        rsx! { <img src={source.clone()} class="w-24 h-32" grayscale /> },
        img(source.clone()).w(px(24.)).h(px(32.)),
    );
    same_style(
        rsx! { <img source={source.clone()} class="size-24" /> },
        img(source).size(px(24.)),
    );
    same_style(
        rsx! { <svg src="icons/check.svg" class="size-24 text-blue-500" /> },
        svg()
            .path("icons/check.svg")
            .size(px(24.))
            .text_color(rgb(0x3b82f6)),
    );
    assert!(
        rsx_expand! { <img src={some_image} grayscale /> }
            .split_whitespace()
            .collect::<String>()
            .contains(".grayscale(true)")
    );
}
