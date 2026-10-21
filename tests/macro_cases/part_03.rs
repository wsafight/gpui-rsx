use super::*;

// ===========================================================================
// 21. whenSome 条件渲染
// ===========================================================================

#[test]
fn test_when_some_with_value() {
    let width: Option<f32> = Some(200.0);
    let _el = rsx! {
        <div
            flex
            whenSome={(width, |this, w| this.w(px(w)))}
        >
            {"Content"}
        </div>
    };
}

#[test]
fn test_when_some_with_none() {
    let height: Option<f32> = None;
    let _el = rsx! {
        <div
            flex
            whenSome={(height, |this, h| this.h(px(h)))}
        >
            {"Content"}
        </div>
    };
}

#[test]
fn test_when_some_with_color() {
    let bg_color: Option<u32> = Some(0x3b82f6);
    let _el = rsx! {
        <div
            flex
            whenSome={(bg_color, |this, color| this.bg(rgb(color)))}
        >
            {"Colored content"}
        </div>
    };
}

#[test]
fn test_when_some_with_string() {
    let placeholder: Option<&str> = Some("Enter text...");
    let _el = rsx! {
        <input
            flex
            whenSome={(placeholder, |this, text| this.placeholder(text))}
        />
    };
}

#[test]
fn test_when_and_when_some_together() {
    let is_active = true;
    let custom_width: Option<f32> = Some(300.0);
    let _el = rsx! {
        <div
            flex
            when={(is_active, |this| this.bg(rgb(0x3b82f6)))}
            whenSome={(custom_width, |this, w| this.w(px(w)))}
        >
            {"Content"}
        </div>
    };
}

#[test]
fn test_when_some_as_ref() {
    let error_message: Option<String> = Some(String::from("An error occurred"));
    let _el = rsx! {
        <div
            flex
            whenSome={(error_message.as_ref(), |this, msg| {
                this.text_color(rgb(0xef4444))
                    .child(msg.clone())
            })}
        />
    };
}

#[test]
fn test_complex_when_usage() {
    let is_selected = true;
    let is_disabled = false;
    let custom_bg: Option<u32> = Some(0xf3f4f6);

    let _el = rsx! {
        <button
            class="flex items-center gap-2 px-4 py-2 rounded-md"
            when={(is_selected, |this| {
                this.bg(rgb(0x3b82f6))
                    .text_color(rgb(0xffffff))
            })}
            when={(is_disabled, |this| {
                this.bg(rgb(0xe5e7eb))
                    .text_color(rgb(0x9ca3af))
            })}
            whenSome={(custom_bg, |this, color| this.bg(rgb(color)))}
        >
            {"Button"}
        </button>
    };
}

// ===========================================================================
// 22. Spread 语法
// ===========================================================================

#[test]
fn test_spread_children() {
    let items = ["a", "b", "c"];
    let _el = rsx! {
        <ul>
            {...items}
        </ul>
    };
}

#[test]
fn test_spread_with_map() {
    let items = ["apple", "banana"];
    let children: Vec<MockElement> = items.iter().map(|_| div()).collect();
    let _el = rsx! {
        <div>
            {...children}
        </div>
    };
}

#[test]
fn test_spread_mixed_with_child() {
    let items = vec!["a", "b"];
    let _el = rsx! {
        <div>
            <span>{"header"}</span>
            {...items}
        </div>
    };
}

// ===========================================================================
// 23. 裸字符串字面量子节点
// ===========================================================================

#[test]
fn test_bare_string_child() {
    let _el = rsx! { <div>"Hello"</div> };
}

#[test]
fn test_bare_string_mixed_with_elements() {
    let _el = rsx! {
        <div>
            "Hello"
            <span>{"world"}</span>
        </div>
    };
}

#[test]
fn test_bare_string_multiple() {
    let _el = rsx! {
        <div>
            "First"
            "Second"
            "Third"
        </div>
    };
}

// ===========================================================================
// 24. svg/img/canvas 标签映射
// ===========================================================================

#[test]
fn test_svg_tag() {
    let _el = rsx! { <svg src={"icons/logo.svg"} /> };
}

#[test]
fn test_img_tag() {
    let _el = rsx! { <img src={"images/logo.png"} /> };
}

#[test]
fn test_img_source_alias_and_image_attrs() {
    let _el = rsx! {
        <img
            source={"images/avatar.webp"}
            grayscale
            objectFit={()}
            withFallback={|| AnyElement}
            withLoading={|| AnyElement}
            imageCache={()}
        />
    };
}

#[test]
fn test_canvas_tag() {
    let _el = rsx! {
        <canvas
            prepaint={|_, _, _| ()}
            paint={|_, _, _, _| {}}
        />
    };
}

#[test]
fn test_svg_with_children() {
    let _el = rsx! {
        <svg>
            {"svg content"}
        </svg>
    };
}

// ===========================================================================
// 25. hover/active/focus 自动 ID
// ===========================================================================

#[test]
fn test_hover_auto_id() {
    let _el = rsx! {
        <div hover={|this| this.bg(rgb(0x3b82f6))}>
            {"Hover me"}
        </div>
    };
}

#[test]
fn test_active_auto_id() {
    let _el = rsx! {
        <div active={|this| this.bg(rgb(0xdc2626))}>
            {"Press me"}
        </div>
    };
}

#[test]
fn test_focus_auto_id() {
    let _el = rsx! {
        <div focus={|this| this.bg(rgb(0x2563eb))}>
            {"Focus me"}
        </div>
    };
}

#[test]
fn test_tooltip_auto_id() {
    let _el = rsx! {
        <div tooltip={"Tooltip text"}>
            {"Hover for tooltip"}
        </div>
    };
}

#[test]
fn test_focusable_auto_id() {
    let _el = rsx! {
        <div focusable>
            {"Tracked focus"}
        </div>
    };
}

#[test]
fn test_hover_focus_active_class_attributes() {
    let _el = rsx! {
        <button
            hoverClass="bg-blue-600 text-white"
            focusClass="border-blue-500"
            activeClass="opacity-75"
        >
            {"State classes"}
        </button>
    };
}

#[test]
fn test_active_class_generates_auto_id() {
    let _el = rsx! {
        <button activeClass="bg-blue-700">
            {"Active"}
        </button>
    };
    let id = take_last_auto_id().expect("activeClass should trigger auto-ID generation");
    assert!(id.contains("__rsx_button_"));
}

#[test]
fn test_group_hover_does_not_generate_auto_id() {
    let _ = take_last_auto_id();
    let _el = rsx! {
        <button groupHover={("menu", |style| style.bg(rgb(0x334155)))}>
            {"Hover"}
        </button>
    };

    assert!(take_last_auto_id().is_none());
}

#[test]
fn test_group_active_generates_auto_id() {
    let _el = rsx! {
        <button groupActive={("menu", |style| style.opacity(0.85))}>
            {"Active"}
        </button>
    };

    let id = take_last_auto_id().expect("groupActive should trigger auto-ID generation");
    assert!(id.contains("__rsx_button_"));
}

#[test]
fn test_gpui_interactive_and_a11y_mappings() {
    let aux = |_: ()| {};
    let a11y = |_: (), _: (), _: ()| {};
    let _el = rsx! {
        <button
            role={"button"}
            ariaLabel={"Open command palette"}
            ariaSelected={true}
            ariaExpanded={false}
            ariaLevel={2usize}
            focusVisible={|style| style.border_color(rgb(0x3b82f6))}
            tooltipShowDelay={0usize}
            onAuxClick={aux}
            onA11yAction={("press", a11y)}
        >
            {"Accessible action"}
        </button>
    };
}

fn assert_auto_id_before_method(expanded: &str, method: &str) {
    let compact = expanded.split_whitespace().collect::<String>();
    let id_position = compact.find(".id(").expect("missing auto ID");
    let method_call = format!(".{method}(");
    let method_position = compact
        .find(&method_call)
        .unwrap_or_else(|| panic!("missing {method_call} in {compact}"));
    assert!(id_position < method_position, "{compact}");
}

#[test]
fn test_latest_stateful_attributes_inject_id_independently() {
    let cases = [
        (
            "accessibility_id",
            gpui_rsx::rsx_expand! { <div accessibility_id={"platform-id"} /> },
            gpui_rsx::rsx_expand! { <div accessibilityId={"platform-id"} /> },
        ),
        (
            "aria_description",
            gpui_rsx::rsx_expand! { <div aria_description={"description"} /> },
            gpui_rsx::rsx_expand! { <div ariaDescription={"description"} /> },
        ),
        (
            "aria_keyshortcuts",
            gpui_rsx::rsx_expand! { <div aria_keyshortcuts={"Ctrl+K"} /> },
            gpui_rsx::rsx_expand! { <div ariaKeyShortcuts={"Ctrl+K"} /> },
        ),
        (
            "aria_active_descendant",
            gpui_rsx::rsx_expand! { <div aria_active_descendant /> },
            gpui_rsx::rsx_expand! { <div ariaActiveDescendant /> },
        ),
        (
            "a11y_synthetic_children",
            gpui_rsx::rsx_expand! { <div a11y_synthetic_children={|| ()} /> },
            gpui_rsx::rsx_expand! { <div a11ySyntheticChildren={|| ()} /> },
        ),
        (
            "aria_numeric_value_step",
            gpui_rsx::rsx_expand! { <div aria_numeric_value_step={1.0} /> },
            gpui_rsx::rsx_expand! { <div ariaNumericValueStep={1.0} /> },
        ),
        (
            "aria_value",
            gpui_rsx::rsx_expand! { <div aria_value={"value"} /> },
            gpui_rsx::rsx_expand! { <div ariaValue={"value"} /> },
        ),
        (
            "aria_placeholder",
            gpui_rsx::rsx_expand! { <div aria_placeholder={"placeholder"} /> },
            gpui_rsx::rsx_expand! { <div ariaPlaceholder={"placeholder"} /> },
        ),
        (
            "restrict_scroll_to_axis",
            gpui_rsx::rsx_expand! { <div restrict_scroll_to_axis /> },
            gpui_rsx::rsx_expand! { <div restrictScrollToAxis /> },
        ),
        (
            "external_drag_payload",
            gpui_rsx::rsx_expand! { <div external_drag_payload={|_| ()} /> },
            gpui_rsx::rsx_expand! { <div externalDragPayload={|_| ()} /> },
        ),
    ];

    for (method, snake_case, camel_case) in cases {
        assert_auto_id_before_method(snake_case, method);
        assert_auto_id_before_method(camel_case, method);
    }
}

#[test]
fn test_latest_stateful_value_is_evaluated_once_and_explicit_id_is_not_duplicated() {
    let mut evaluations = 0;
    let _el = rsx! {
        <div
            id="described-control"
            ariaDescription={{
                evaluations += 1;
                "description"
            }}
        />
    };
    assert_eq!(evaluations, 1);

    let expanded = gpui_rsx::rsx_expand! {
        <div id="described-control" ariaDescription={"description"} />
    };
    let compact = expanded.split_whitespace().collect::<String>();
    assert_eq!(compact.matches(".id(").count(), 1, "{compact}");
}

#[test]
fn test_latest_interactive_event_aliases_do_not_inject_id() {
    let cases = [
        (
            "on_mouse_exit",
            gpui_rsx::rsx_expand! { <div onMouseExit={h} /> },
        ),
        (
            "on_mouse_pressure",
            gpui_rsx::rsx_expand! { <div onMousePressure={h} /> },
        ),
        (
            "capture_mouse_pressure",
            gpui_rsx::rsx_expand! { <div captureMousePressure={h} /> },
        ),
        ("on_pinch", gpui_rsx::rsx_expand! { <div onPinch={h} /> }),
        (
            "capture_pinch",
            gpui_rsx::rsx_expand! { <div capturePinch={h} /> },
        ),
    ];

    for (method, expanded) in cases {
        let compact = expanded.split_whitespace().collect::<String>();
        assert!(compact.contains(&format!(".{method}(")), "{compact}");
        assert!(!compact.contains(".id("), "{compact}");
    }
}

#[test]
fn test_scrollbar_width_does_not_inject_id() {
    take_last_auto_id();
    let _el = rsx! { <div scrollbarWidth={px(8.0)} /> };
    assert!(take_last_auto_id().is_none());

    let expanded = gpui_rsx::rsx_expand! { <div scrollbarWidth={px(8.0)} /> };
    let compact = expanded.split_whitespace().collect::<String>();
    assert!(compact.contains(".scrollbar_width("), "{compact}");
    assert!(!compact.contains(".id("), "{compact}");
}

#[test]
fn test_ellipsis_classes_cover_static_strict_and_dynamic_paths() {
    let _static = rsx! { <div class="text-ellipsis-start text-ellipsis-middle" /> };
    let _strict = gpui_rsx::rsx_strict! {
        <div class="text-ellipsis-start text-ellipsis-middle" />
    };
    let classes = "text-ellipsis-start text-ellipsis-middle";
    let _dynamic = rsx! { <div class={classes} /> };

    let expanded = gpui_rsx::rsx_expand! {
        <div class="text-ellipsis-start text-ellipsis-middle" />
    };
    let compact = expanded.split_whitespace().collect::<String>();
    assert!(compact.contains(".text_ellipsis_start()"), "{compact}");
    assert!(compact.contains(".text_ellipsis_middle()"), "{compact}");
}

#[test]
fn test_grid_min_max_content_aliases_keep_u16_values() {
    take_integer_calls();
    let _el = rsx! {
        <div
            gridColsMinContent={1u16}
            gridColsMaxContent={2u16}
            gridRowsMinContent={3u16}
            gridRowsMaxContent={4u16}
        />
    };
    assert_eq!(
        take_integer_calls(),
        vec![
            ("grid_cols_min_content", 1),
            ("grid_cols_max_content", 2),
            ("grid_rows_min_content", 3),
            ("grid_rows_max_content", 4),
        ]
    );
}
