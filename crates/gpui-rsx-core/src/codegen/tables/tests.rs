use super::*;

// --- lookup_color ---

#[test]
fn color_table_contains_full_tailwind_palette() {
    // 验证完整 Tailwind 色板：22 色系 × 11 色阶（复用模块级常量，单一数据源）
    for family in COLOR_FAMILIES {
        for shade in COLOR_SHADES {
            let key = format!("{family}_{shade}");
            assert!(
                lookup_color(&key).is_some(),
                "缺少颜色: {key}（请在 lookup_color 中补充）"
            );
        }
    }
}

#[test]
fn color_table_contains_black_and_white() {
    assert_eq!(lookup_color("black"), Some(0x000000));
    assert_eq!(lookup_color("white"), Some(0xffffff));
}

#[test]
fn color_table_spot_check_values() {
    // 抽检几个关键颜色值，防止复制粘贴错误
    assert_eq!(lookup_color("red_500"), Some(0xef4444));
    assert_eq!(lookup_color("blue_500"), Some(0x3b82f6));
    assert_eq!(lookup_color("green_500"), Some(0x22c55e));
    assert_eq!(lookup_color("gray_500"), Some(0x6b7280));
    assert_eq!(lookup_color("amber_500"), Some(0xf59e0b));
    assert_eq!(lookup_color("purple_500"), Some(0xa855f7));
}

#[test]
fn color_table_unknown_returns_none() {
    assert_eq!(lookup_color("not_a_color"), None);
    assert_eq!(lookup_color("red_999"), None);
    assert_eq!(lookup_color(""), None);
}

// --- lookup_attr_method ---

#[test]
fn attr_method_camel_case_events() {
    let cases = [
        ("onClick", "on_click"),
        ("onMouseDown", "on_mouse_down"),
        ("onKeyDown", "on_key_down"),
        ("onDragMove", "on_drag_move"),
        ("onBoxedAction", "on_boxed_action"),
        ("onAuxClick", "on_aux_click"),
        ("onA11yAction", "on_a11y_action"),
        ("onMouseExit", "on_mouse_exit"),
        ("onMousePressure", "on_mouse_pressure"),
        ("captureMousePressure", "capture_mouse_pressure"),
        ("onPinch", "on_pinch"),
        ("capturePinch", "capture_pinch"),
    ];

    for (attr, method) in cases {
        assert_eq!(lookup_attr_method(attr), Some(method), "{attr}");
    }
}

#[test]
fn attr_method_snake_case_events() {
    assert_eq!(lookup_attr_method("on_click"), Some("on_click"));
    assert_eq!(lookup_attr_method("on_mouse_down"), Some("on_mouse_down"));
    assert_eq!(lookup_attr_method("on_key_down"), Some("on_key_down"));
}

#[test]
fn attr_method_dimension_aliases() {
    assert_eq!(lookup_attr_method("width"), Some("w"));
    assert_eq!(lookup_attr_method("height"), Some("h"));
    assert_eq!(lookup_attr_method("minWidth"), Some("min_w"));
    assert_eq!(lookup_attr_method("maxHeight"), Some("max_h"));
    assert_eq!(lookup_attr_method("trackFocus"), Some("track_focus"));
    assert_eq!(lookup_attr_method("focusVisible"), Some("focus_visible"));
    assert_eq!(
        lookup_attr_method("tooltipShowDelay"),
        Some("tooltip_show_delay")
    );
    assert_eq!(lookup_attr_method("groupHover"), Some("group_hover"));
    assert_eq!(lookup_attr_method("groupActive"), Some("group_active"));
    assert_eq!(lookup_attr_method("ariaLabel"), Some("aria_label"));
    assert_eq!(lookup_attr_method("ariaSelected"), Some("aria_selected"));
    assert_eq!(lookup_attr_method("fontSize"), Some("text_size"));
    assert_eq!(lookup_attr_method("fontFamily"), Some("font_family"));
    assert_eq!(lookup_attr_method("textColor"), Some("text_color"));
    assert_eq!(lookup_attr_method("backgroundColor"), Some("bg"));
    assert_eq!(lookup_attr_method("borderColor"), Some("border_color"));
    // `flexGrow={2.0}` now maps to GPUI's `flex_grow(f32)`; the flag form stays `flex_grow_1`.
    assert_eq!(lookup_attr_method("flexGrow"), Some("flex_grow"));
    assert_eq!(lookup_attr_method("flexShrink"), Some("flex_shrink"));
    assert_eq!(lookup_attr_method("zIndex"), None);
}

#[test]
fn attr_method_keeps_directional_border_value_methods() {
    assert_eq!(lookup_attr_method("border_t"), Some("border_t"));
    assert_eq!(lookup_attr_method("border_b"), Some("border_b"));
    assert_eq!(lookup_attr_method("border_l"), Some("border_l"));
    assert_eq!(lookup_attr_method("border_r"), Some("border_r"));
}

#[test]
fn attr_flag_method_maps_directional_border_width_flags() {
    assert_eq!(lookup_attr_flag_method("border_t"), Some("border_t_1"));
    assert_eq!(lookup_attr_flag_method("border_b"), Some("border_b_1"));
    assert_eq!(lookup_attr_flag_method("border_l"), Some("border_l_1"));
    assert_eq!(lookup_attr_flag_method("border_r"), Some("border_r_1"));
}

#[test]
fn attr_flag_method_maps_gpui_flex_flags() {
    assert_eq!(lookup_attr_flag_method("flexGrow"), Some("flex_grow_1"));
    assert_eq!(lookup_attr_flag_method("flexShrink"), Some("flex_shrink_1"));
}

#[test]
fn attr_method_unknown_returns_none() {
    assert_eq!(lookup_attr_method("onFocus"), None);
    assert_eq!(lookup_attr_method("onDoubleClick"), None);
    assert_eq!(lookup_attr_method("unknown_attr"), None);
    assert_eq!(lookup_attr_method(""), None);
}

#[test]
fn latest_stateful_aliases_keep_method_properties() {
    let cases = [
        ("accessibilityId", "accessibility_id", false),
        ("ariaDescription", "aria_description", false),
        ("ariaKeyShortcuts", "aria_keyshortcuts", false),
        ("ariaActiveDescendant", "aria_active_descendant", false),
        ("a11ySyntheticChildren", "a11y_synthetic_children", false),
        ("ariaNumericValueStep", "aria_numeric_value_step", false),
        ("ariaValue", "aria_value", false),
        ("ariaPlaceholder", "aria_placeholder", false),
        ("restrictScrollToAxis", "restrict_scroll_to_axis", false),
        ("externalDragPayload", "external_drag_payload", false),
    ];

    for (attr, method, multi_arg) in cases {
        let info = lookup_attr_method_info(attr).unwrap_or_else(|| panic!("missing {attr}"));
        assert_eq!(info.method, method, "{attr}");
        assert!(info.needs_id, "{attr}");
        assert_eq!(info.multi_arg, multi_arg, "{attr}");
        assert!(is_stateful_attr(method), "{method}");
    }
}

#[test]
fn latest_interactive_events_do_not_need_id() {
    for attr in [
        "onMouseExit",
        "on_mouse_exit",
        "onMousePressure",
        "on_mouse_pressure",
        "captureMousePressure",
        "capture_mouse_pressure",
        "onPinch",
        "on_pinch",
        "capturePinch",
        "capture_pinch",
    ] {
        let info = lookup_attr_method_info(attr).unwrap_or_else(|| panic!("missing {attr}"));
        assert!(!info.needs_id, "{attr}");
        assert!(!info.multi_arg, "{attr}");
    }
}

#[test]
fn latest_styled_aliases_map_without_id() {
    let cases = [
        ("scrollbarWidth", "scrollbar_width"),
        ("gridColsMinContent", "grid_cols_min_content"),
        ("gridColsMaxContent", "grid_cols_max_content"),
        ("gridRowsMinContent", "grid_rows_min_content"),
        ("gridRowsMaxContent", "grid_rows_max_content"),
    ];

    for (attr, method) in cases {
        let info = lookup_attr_method_info(attr).unwrap_or_else(|| panic!("missing {attr}"));
        assert_eq!(info.method, method, "{attr}");
        assert!(!info.needs_id, "{attr}");
        assert!(!info.multi_arg, "{attr}");
        assert!(!is_stateful_attr(method), "{method}");
    }
}

#[test]
fn generated_stateful_methods_are_classified() {
    let stateful: Vec<_> = GPUI_METHODS
        .iter()
        .filter(|m| m.traits & T_STATEFUL != 0)
        .collect();
    assert!(
        stateful.len() >= 40,
        "StatefulInteractiveElement facts missing"
    );
    for method in stateful {
        assert!(
            is_stateful_method(method.name),
            "unclassified: {}",
            method.name
        );
    }
}

#[test]
fn generated_method_table_is_sorted_for_binary_search() {
    assert!(GPUI_METHODS.windows(2).all(|w| w[0].name < w[1].name));
    assert_eq!(lookup_gpui_method("flex_col").map(|m| m.arity), Some(0));
    assert_eq!(
        lookup_gpui_method("on_mouse_down").map(|m| m.arity),
        Some(2)
    );
    assert!(lookup_gpui_method("not_a_method").is_none());
}

#[test]
fn length_specs_point_at_real_gpui_setters() {
    for spec in LENGTH_CLASS_SPECS {
        let method = lookup_gpui_method(spec.method)
            .unwrap_or_else(|| panic!("{} is not a GPUI method", spec.method));
        assert_eq!(method.arity, 1, "{}", spec.method);
    }
}

#[test]
fn semantic_aliases_point_at_real_gpui_methods() {
    for name in [
        "width",
        "fontSize",
        "backgroundColor",
        "borderTop",
        "boxShadow",
        "ariaKeyShortcuts",
        "gridCols",
    ] {
        let method = lookup_semantic_attr_alias(name).unwrap();
        assert!(lookup_gpui_method(method).is_some(), "{name} → {method}");
    }
}

#[test]
fn attr_method_info_keeps_behavior_flags_together() {
    let a11y = lookup_attr_method_info("onA11yAction").unwrap();
    assert_eq!(a11y.method, "on_a11y_action");
    assert!(a11y.needs_id);
    assert!(a11y.multi_arg);

    let mouse_down = lookup_attr_method_info("onMouseDown").unwrap();
    assert_eq!(mouse_down.method, "on_mouse_down");
    assert!(!mouse_down.needs_id);
    assert!(mouse_down.multi_arg);

    let focus_visible = lookup_attr_method_info("focusVisible").unwrap();
    assert_eq!(focus_visible.method, "focus_visible");
    assert!(!focus_visible.needs_id);
    assert!(!focus_visible.multi_arg);

    let aria_label = lookup_attr_method_info("ariaLabel").unwrap();
    assert_eq!(aria_label.method, "aria_label");
    assert!(aria_label.needs_id);
    assert!(!aria_label.multi_arg);

    for attr in ["groupActive", "onA11yAction", "onDrag"] {
        let info = lookup_attr_method_info(attr).unwrap();
        assert!(info.needs_id, "{attr}");
        assert!(info.multi_arg, "{attr}");
    }

    for attr in [
        "groupHover",
        "onMouseDown",
        "onMouseUp",
        "onMouseUpOut",
        "onBoxedAction",
    ] {
        let info = lookup_attr_method_info(attr).unwrap();
        assert!(!info.needs_id, "{attr}");
        assert!(info.multi_arg, "{attr}");
    }
}

// --- is_stateful_attr ---

#[test]
fn stateful_attr_detects_event_handlers() {
    assert!(is_stateful_attr("onClick"));
    assert!(is_stateful_attr("on_click"));
    assert!(is_stateful_attr("onDrag"));
    assert!(is_stateful_attr("on_hover"));
}

#[test]
fn stateful_attr_detects_interactive_attrs() {
    // StatefulInteractiveElement 方法需要 ID。
    assert!(is_stateful_attr("active"));
    assert!(is_stateful_attr("focusable"));
    assert!(is_stateful_attr("overflow_scroll"));
    assert!(is_stateful_attr("tooltip"));
    assert!(is_stateful_attr("track_scroll"));
    assert!(is_stateful_attr("onAuxClick"));
    assert!(is_stateful_attr("ariaLabel"));
    assert!(is_stateful_attr("role"));
    assert!(is_stateful_attr("tooltipShowDelay"));
    assert!(is_stateful_attr("groupActive"));
}

#[test]
fn stateful_attr_ignores_styled_trait_methods() {
    // hover / focus / group 是 Styled trait 的样式方法，不需要 ID
    assert!(!is_stateful_attr("hover"));
    assert!(!is_stateful_attr("focus"));
    assert!(!is_stateful_attr("group"));
    assert!(!is_stateful_attr("groupHover"));
}

#[test]
fn stateful_attr_ignores_non_stateful() {
    assert!(!is_stateful_attr("onMouseDown"));
    assert!(!is_stateful_attr("captureKeyDown"));
    assert!(!is_stateful_attr("class"));
    assert!(!is_stateful_attr("id"));
    assert!(!is_stateful_attr("flex"));
    assert!(!is_stateful_attr("gap"));
}

// --- lookup_spacing_method ---

#[test]
fn spacing_method_covers_all_directions() {
    assert_eq!(lookup_spacing_method("gap_"), Some("gap"));
    assert_eq!(lookup_spacing_method("gap_x_"), Some("gap_x"));
    assert_eq!(lookup_spacing_method("gap_y_"), Some("gap_y"));
    assert_eq!(lookup_spacing_method("p_"), Some("p"));
    assert_eq!(lookup_spacing_method("px_"), Some("px"));
    assert_eq!(lookup_spacing_method("py_"), Some("py"));
    assert_eq!(lookup_spacing_method("m_"), Some("m"));
    assert_eq!(lookup_spacing_method("mx_"), Some("mx"));
    assert_eq!(lookup_spacing_method("my_"), Some("my"));
    assert_eq!(lookup_spacing_method("w_"), Some("w"));
    assert_eq!(lookup_spacing_method("h_"), Some("h"));
    assert_eq!(lookup_spacing_method("min_w_"), Some("min_w"));
    assert_eq!(lookup_spacing_method("max_h_"), Some("max_h"));
}

#[test]
fn spacing_method_unknown_returns_none() {
    assert_eq!(lookup_spacing_method("text_"), None);
    assert_eq!(lookup_spacing_method(""), None);
}

// --- shared length class specs ---

#[test]
fn split_length_class_prefers_long_prefixes() {
    assert_eq!(
        split_length_class("min-w-[280px]"),
        Some(("min_w", "[280px]", LengthClassFamily::Sizing))
    );
    assert_eq!(
        split_length_class("max-h-1/2"),
        Some(("max_h", "1/2", LengthClassFamily::Sizing))
    );
    assert_eq!(
        split_length_class("gap-x-[1rem]"),
        Some(("gap_x", "[1rem]", LengthClassFamily::Spacing))
    );
    assert_eq!(
        split_length_class("gap-y-4"),
        Some(("gap_y", "4", LengthClassFamily::Spacing))
    );
}

#[test]
fn split_length_class_separates_sizing_and_spacing_rules() {
    assert_eq!(
        split_length_class("w-[50%]"),
        Some(("w", "[50%]", LengthClassFamily::Sizing))
    );
    assert_eq!(
        split_length_class("size-1/2"),
        Some(("size", "1/2", LengthClassFamily::Sizing))
    );
    assert_eq!(
        split_length_class("p-[8px]"),
        Some(("p", "[8px]", LengthClassFamily::Spacing))
    );
    assert_eq!(
        split_length_class("m-1/2"),
        Some(("m", "1/2", LengthClassFamily::Spacing))
    );
    assert_eq!(split_length_class("line-clamp-3"), None);
}

#[test]
fn length_class_family_capabilities_match_tailwind_subset() {
    assert!(LengthClassFamily::Sizing.allows_percent());
    assert!(LengthClassFamily::Sizing.allows_fraction());
    assert!(!LengthClassFamily::Spacing.allows_percent());
    assert!(!LengthClassFamily::Spacing.allows_fraction());
}

// --- common class support metadata ---

#[test]
fn common_class_metadata_distinguishes_strict_and_dynamic_support() {
    assert!(is_supported_common_no_arg_class("flex"));
    assert!(is_supported_common_no_arg_class("overflow-scroll"));
    assert!(is_supported_common_no_arg_class("gap-4")); // GPUI preset `gap_4`
    assert!(!is_supported_common_no_arg_class("font-bold"));
    assert!(!is_supported_common_no_arg_class("not-a-class"));

    let dynamic = dynamic_common_classes().collect::<Vec<_>>();
    assert!(dynamic.contains(&"flex"));
    assert!(dynamic.contains(&"gap-4"));
    assert!(dynamic.contains(&"font-bold"));
    assert!(!dynamic.contains(&"overflow-scroll"));
    assert!(!dynamic.contains(&"overflow-x-scroll"));
    assert!(!dynamic.contains(&"overflow-y-scroll"));
}

#[test]
fn dynamic_fast_path_has_no_duplicates_or_stateful_classes() {
    let classes: Vec<_> = dynamic_common_classes().collect();
    for (index, class) in classes.iter().enumerate() {
        assert!(
            !classes[..index].contains(class),
            "duplicate dynamic class {class}"
        );
        assert!(!is_stateful_class(class), "stateful dynamic class {class}");
    }
    assert!(classes.contains(&"rounded-md"));
    assert!(classes.contains(&"w-full"));
    assert!(classes.contains(&"mx-auto"));
    assert!(!classes.contains(&"p-0p5"));
    assert!(!classes.contains(&"debug"));
}

// --- is_valid_text_size ---

#[test]
fn text_size_validates_known_sizes() {
    for size in ["xs", "sm", "base", "lg", "xl", "2xl", "3xl"] {
        assert!(is_valid_text_size(size), "应接受文本大小: {size}");
    }
}

#[test]
fn text_size_rejects_unknown() {
    assert!(!is_valid_text_size("huge"));
    assert!(!is_valid_text_size("4xl"));
    assert!(!is_valid_text_size("5xl"));
    assert!(!is_valid_text_size("6xl"));
    assert!(!is_valid_text_size(""));
}
