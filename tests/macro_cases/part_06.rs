use super::*;

#[test]
fn test_shared_dynamic_helper_preserves_evaluation_and_per_element_handlers() {
    let evaluations = std::cell::Cell::new(0);
    let first = std::cell::RefCell::new(Vec::new());
    let second = std::cell::RefCell::new(Vec::new());
    let _ = rsx! {
        <div class={{ evaluations.set(evaluations.get() + 1); "gap-7 unknown-one" }}
            onUnknownClass={|class: &str| first.borrow_mut().push(class.to_owned())}>
            <div class={{ evaluations.set(evaluations.get() + 1); "p-5 unknown-two" }}
                onUnknownClass={|class: &str| second.borrow_mut().push(class.to_owned())}/>
        </div>
    };
    assert_eq!(evaluations.get(), 2);
    assert_eq!(*first.borrow(), ["unknown-one"]);
    assert_eq!(*second.borrow(), ["unknown-two"]);
}

// ===========================================================================
// 44. 动态 class 数值前缀回退（任意数值支持）
// ===========================================================================

#[test]
fn test_dynamic_class_arbitrary_gap() {
    take_length_calls();

    // gap-7 / gap-9 / gap-16 不在静态枚举中，走数值前缀回退路径
    let cls = "gap-7";
    let _a = rsx! { <div class={cls} /> };
    let cls = "gap-9";
    let _b = rsx! { <div class={cls} /> };
    let cls = "gap-16";
    let _c = rsx! { <div class={cls} /> };

    assert_eq!(
        take_length_calls(),
        vec![
            ("px", scaled_px(7.0)),
            ("px", scaled_px(9.0)),
            ("px", scaled_px(16.0))
        ]
    );
}

#[test]
fn test_dynamic_class_arbitrary_padding() {
    take_length_calls();

    // p-5 / px-5 / py-5 不在静态枚举中，走数值前缀回退路径
    let cls = "p-5";
    let _a = rsx! { <div class={cls} /> };
    let cls = "px-5";
    let _b = rsx! { <div class={cls} /> };
    let cls = "py-5";
    let _c = rsx! { <div class={cls} /> };
    let cls = "pt-3 pb-3";
    let _d = rsx! { <div class={cls} /> };

    assert_eq!(
        take_length_calls(),
        vec![
            ("px", scaled_px(5.0)),
            ("px", scaled_px(5.0)),
            ("px", scaled_px(5.0)),
            ("px", scaled_px(3.0)),
            ("px", scaled_px(3.0))
        ]
    );
}

#[test]
fn test_dynamic_class_arbitrary_margin() {
    take_length_calls();

    // m-3 / ml-3 / mr-3 走数值前缀回退路径
    let cls = "m-3";
    let _a = rsx! { <div class={cls} /> };
    let cls = "ml-3";
    let _b = rsx! { <div class={cls} /> };
    let cls = "mr-3";
    let _c = rsx! { <div class={cls} /> };

    assert_eq!(
        take_length_calls(),
        vec![
            ("px", scaled_px(3.0)),
            ("px", scaled_px(3.0)),
            ("px", scaled_px(3.0))
        ]
    );
}

#[test]
fn test_dynamic_class_opacity() {
    // opacity-50 走静态快速路径；opacity-33 走数值前缀回退路径
    let cls = "opacity-50";
    let _a = rsx! { <div class={cls} /> };
    let cls = "opacity-33";
    let _b = rsx! { <div class={cls} /> };
    let cls = "opacity-0";
    let _c = rsx! { <div class={cls} /> };
}

#[test]
fn test_dynamic_class_sizing_arbitrary() {
    take_length_calls();

    // w-48 / h-16 / size-8 走数值前缀回退路径
    let cls = "w-48";
    let _a = rsx! { <div class={cls} /> };
    let cls = "h-16";
    let _b = rsx! { <div class={cls} /> };
    let cls = "size-8";
    let _c = rsx! { <div class={cls} /> };

    assert_eq!(
        take_length_calls(),
        vec![
            ("px", scaled_px(48.0)),
            ("px", scaled_px(16.0)),
            ("px", scaled_px(8.0))
        ]
    );
}

#[test]
fn test_dynamic_class_arbitrary_lengths() {
    take_length_calls();

    let cls = "w-[280px] h-[50%] min-w-6/24 max-w-[32rem] gap-[14px] gap-x-[0.75rem] m-[18px] mx-[1.25rem]";
    let _el = rsx! { <div class={cls} /> };

    assert_eq!(
        take_length_calls(),
        vec![
            ("px", 280.0),
            ("relative", 0.5),
            ("relative", 0.25),
            ("rems", 32.0),
            ("px", 14.0),
            ("rems", 0.75),
            ("px", 18.0),
            ("rems", 1.25)
        ]
    );
}

#[test]
fn test_dynamic_class_ignores_invalid_arbitrary_lengths() {
    take_length_calls();

    let cls = "gap-[10%] p-[10%] w-[bad] w-1/0 m-1/2";
    let _el = rsx! { <div class={cls} /> };

    assert_eq!(take_length_calls(), Vec::<(&'static str, f32)>::new());
}

#[test]
fn test_dynamic_class_keeps_valid_lengths_when_neighbors_are_invalid() {
    take_length_calls();

    let cls = "gap-[10%] gap-[14px] p-[10%] p-[4px] w-[bad] h-[25%]";
    let _el = rsx! { <div class={cls} /> };

    assert_eq!(
        take_length_calls(),
        vec![("px", 14.0), ("px", 4.0), ("relative", 0.25)]
    );
}

#[test]
fn test_dynamic_class_spacing_rejects_fraction_while_sizing_accepts_it() {
    take_length_calls();

    let cls = "m-1/2 w-1/2 size-3/4 h-1/0 max-w-2/4";
    let _el = rsx! { <div class={cls} /> };

    assert_eq!(
        take_length_calls(),
        vec![("relative", 0.5), ("relative", 0.75), ("relative", 0.5)]
    );
}

#[test]
fn test_dynamic_class_font_extralight_mapping() {
    take_font_weight_calls();

    let cls = "font-extralight";
    let _el = rsx! { <div class={cls} /> };

    assert_eq!(take_font_weight_calls(), vec![200.0]);
}

#[test]
fn test_dynamic_class_rejects_non_finite_numeric_values() {
    take_length_calls();

    let cls = "w-[NaNpx] h-[inf%] gap-NaN p-[infrem] w-NaN/1 h-1/-2";
    let _el = rsx! { <div class={cls} /> };

    assert_eq!(take_length_calls(), Vec::<(&'static str, f32)>::new());
}

#[test]
fn test_dynamic_class_gap_xy_arbitrary() {
    take_length_calls();

    // gap-x-4 / gap-y-6 走数值前缀回退路径
    let cls = "gap-x-4";
    let _a = rsx! { <div class={cls} /> };
    let cls = "gap-y-6";
    let _b = rsx! { <div class={cls} /> };

    assert_eq!(
        take_length_calls(),
        vec![("px", scaled_px(4.0)), ("px", scaled_px(6.0))]
    );
}

#[test]
fn test_dynamic_class_gpui_styled_helpers() {
    take_integer_calls();

    let cls = "flex-grow flex-shrink content-normal self-center whitespace-nowrap line-clamp-3 col-span-full row-end-auto shadow-xl cursor-grab overflow-x-hidden no-underline debug-outline";
    let _el = rsx! { <div class={cls} /> };

    assert_eq!(take_integer_calls(), vec![("line_clamp", 3)]);
}

#[test]
fn test_dynamic_class_integer_fallbacks() {
    take_integer_calls();

    let cls = "line-clamp-4 grid-cols-12 grid-rows-3 col-span-2 row-span-5 col-start--1 col-end-4 row-start--2 row-end-6";
    let _el = rsx! { <div class={cls} /> };

    assert_eq!(
        take_integer_calls(),
        vec![
            ("line_clamp", 4),
            ("grid_cols", 12),
            ("grid_rows", 3),
            ("col_span", 2),
            ("row_span", 5),
            ("col_start", -1),
            ("col_end", 4),
            ("row_start", -2),
            ("row_end", 6)
        ]
    );
}

#[test]
fn test_dynamic_class_integer_fallbacks_ignore_invalid_values() {
    take_integer_calls();

    let cls = "line-clamp--1 grid-cols-70000 col-span--2 row-span-abc col-start-abc row-end-40000 line-clamp-3";
    let _el = rsx! { <div class={cls} /> };

    assert_eq!(take_integer_calls(), vec![("line_clamp", 3)]);
}

#[test]
fn test_strict_dynamic_class_supports_shared_fast_path_entries() {
    take_font_weight_calls();
    take_length_calls();

    let cls = "flex font-bold gap-4";
    let _el = gpui_rsx::rsx_strict! { <div class={cls} /> };

    assert_eq!(take_font_weight_calls(), vec![700.0]);
    assert_eq!(take_length_calls(), vec![("px", scaled_px(4.0))]);
}

#[test]
#[should_panic(expected = "unsupported dynamic class")]
fn test_strict_dynamic_class_rejects_strict_only_overflow_scroll() {
    let cls = "overflow-scroll";
    let _el = gpui_rsx::rsx_strict! { <div class={cls} /> };
}

#[test]
fn test_dynamic_class_directional_border() {
    // Bug: static path maps `border-t` → .border_t_1(), dynamic path drops it.
    // After fix, both paths must accept `border-t` / `border-b` / `border-l` /
    // `border-r` / `border-x` / `border-y` in dynamic class expressions.
    common::take_border_calls();

    let cls = "border-t border-b border-l border-r border-x border-y";
    let _el = rsx! { <div class={cls} /> };

    let calls = common::take_border_calls();
    assert!(
        calls.contains(&"border_t_1"),
        "dynamic class `border-t` should map to .border_t_1(), got {calls:?}"
    );
    assert!(
        calls.contains(&"border_b_1"),
        "dynamic class `border-b` should map to .border_b_1(), got {calls:?}"
    );
    assert!(
        calls.contains(&"border_l_1"),
        "dynamic class `border-l` should map to .border_l_1(), got {calls:?}"
    );
    assert!(
        calls.contains(&"border_r_1"),
        "dynamic class `border-r` should map to .border_r_1(), got {calls:?}"
    );
    assert!(
        calls.contains(&"border_x_1"),
        "dynamic class `border-x` should map to .border_x_1(), got {calls:?}"
    );
    assert!(
        calls.contains(&"border_y_1"),
        "dynamic class `border-y` should map to .border_y_1(), got {calls:?}"
    );
}

#[test]
fn test_conditional_literal_class_static_path_evaluates_condition_once() {
    take_length_calls();

    let mut condition_calls = 0;
    let _el = rsx! {
        <div class={if { condition_calls += 1; true } { "gap-7" } else { "gap-9" }} />
    };

    assert_eq!(condition_calls, 1);
    assert_eq!(take_length_calls(), vec![("px", scaled_px(7.0))]);
}

#[test]
fn test_match_literal_class_static_path_evaluates_expr_once() {
    take_length_calls();

    let mut match_expr_calls = 0;
    let _el = rsx! {
        <div class={match { match_expr_calls += 1; "primary" } {
            "primary" => "gap-7",
            _ => "gap-9",
        }} />
    };

    assert_eq!(match_expr_calls, 1);
    assert_eq!(take_length_calls(), vec![("px", scaled_px(7.0))]);
}

#[test]
fn test_match_literal_class_stateful_generates_auto_id() {
    take_last_auto_id();

    let active = true;
    let _el = rsx! {
        <div class={match active {
            true => "overflow-scroll",
            false => "",
        }} />
    };

    assert!(take_last_auto_id().is_some());
}

#[test]
fn test_rsx_expand_preview_contains_generated_code() {
    let expanded = gpui_rsx::rsx_expand! {
        <div class="flex w-[280px] bg-[rgba(15,23,42,0.8)]" />
    };

    assert!(expanded.contains("flex"));
    assert!(expanded.contains("w"));
    assert!(expanded.contains("280"));
    assert!(expanded.contains("rgba"));
}

#[test]
fn test_rsx_expand_preview_staticizes_conditional_literal_class() {
    let expanded = gpui_rsx::rsx_expand! {
        <div class={if active { "flex gap-7" } else { "block" }} />
    };
    let compact = expanded.split_whitespace().collect::<String>();

    assert!(!expanded.contains("__rsx_apply_class"));
    assert!(compact.contains("ifactive"));
    assert!(compact.contains(&format!(".flex().gap(px({}", scaled_px(7.0))));
    assert!(compact.contains(".block()"));
}

#[test]
fn test_rsx_expand_preview_staticizes_match_literal_class() {
    let expanded = gpui_rsx::rsx_expand! {
        <div class={match state { 0 => "flex gap-7", _ => "block" }} />
    };
    let compact = expanded.split_whitespace().collect::<String>();

    assert!(!expanded.contains("__rsx_apply_class"));
    assert!(compact.contains("matchstate"));
    assert!(compact.contains(&format!("0=>__el.flex().gap(px({}", scaled_px(7.0))));
    assert!(compact.contains("_=>__el.block()"));
}

#[test]
fn test_rsx_expand_preview_uses_base_attribute() {
    let expanded = gpui_rsx::rsx_expand! {
        <Button base={Button::new("save")} label={"保存"} small />
    };
    let compact = expanded.replace(' ', "");

    assert!(compact.contains("Button::new(\"save\").label(\"保存\").small()"));
    assert!(!compact.contains(".base("));
}

#[test]
fn test_rsx_expand_preview_uses_path_tag_constructor() {
    let expanded = gpui_rsx::rsx_expand! {
        <ui::TaskCard flex />
    };
    let compact = expanded.replace(' ', "");

    assert!(compact.contains("ui::TaskCard().flex()"));
}

#[test]
fn test_rsx_expand_preview_uses_when_class() {
    let expanded = gpui_rsx::rsx_expand! {
        <div whenClass={(active, "bg-neutral-900 text-white")} />
    };
    let compact = expanded.split_whitespace().collect::<String>();

    assert!(
        compact
            .contains(".when(active,|__el|__el.bg(rgb(1513239u32)).text_color(rgb(16777215u32)))")
    );
}

#[test]
fn test_rsx_expand_preview_uses_state_class_attributes() {
    let expanded = gpui_rsx::rsx_expand! {
        <button
            hoverClass="bg-blue-600"
            focusClass="border-blue-500"
            activeClass="opacity-75"
        />
    };
    let compact = expanded.split_whitespace().collect::<String>();

    assert!(compact.contains(".id("));
    assert!(compact.contains(".hover(|__style|__style.bg(rgb("));
    assert!(compact.contains(".focus(|__style|__style.border_color(rgb("));
    assert!(compact.contains(".active(|__style|__style.opacity("));
    assert!(!compact.contains("__rsx_apply_class"));
}

#[test]
fn test_rsx_expand_preview_hover_focus_class_do_not_inject_id() {
    let expanded = gpui_rsx::rsx_expand! {
        <button
            hoverClass="bg-blue-600"
            focusClass="border-blue-500"
        />
    };
    let compact = expanded.split_whitespace().collect::<String>();

    assert!(!compact.contains(".id("));
    assert!(compact.contains(".hover(|__style|__style.bg(rgb("));
    assert!(compact.contains(".focus(|__style|__style.border_color(rgb("));
}

#[test]
fn test_rsx_expand_preview_expands_multi_arg_interactive_attrs() {
    let expanded = gpui_rsx::rsx_expand! {
        <button
            onA11yAction={(AccessibleAction::Click, handler)}
            onAuxClick={aux}
        />
    };
    let compact = expanded.split_whitespace().collect::<String>();

    assert!(compact.contains(".id("));
    assert!(compact.contains(".on_a11y_action(AccessibleAction::Click,handler)"));
    assert!(compact.contains(".on_aux_click(aux)"));
}
