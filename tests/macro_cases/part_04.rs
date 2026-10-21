use super::*;

// ===========================================================================
// 26. 新事件处理器
// ===========================================================================

#[test]
fn test_on_hover_event() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onHover={h} /> };
}

#[test]
fn test_on_scroll_wheel_event() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onScrollWheel={h} /> };
}

#[test]
fn test_on_drag_event() {
    let h = |_: (), _: ()| {};
    let drag = ();
    let _el = rsx! { <div onDrag={(drag, h)} /> };
}

#[test]
fn test_on_drop_event() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onDrop={h} /> };
}

#[test]
fn test_on_action_event() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onAction={h} /> };
}

#[test]
fn test_on_hover_snake_case() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div on_hover={h} /> };
}

// 验证 onHover/onDrag/onDrop 事件处理器会自动注入 ID（v0.2.0 修复）
#[test]
fn test_on_hover_event_auto_id() {
    let h = |_: (), _: ()| {};
    // onHover 事件应该自动注入 .id()，因为它是 StatefulInteractiveElement 方法
    let _el = rsx! { <div onHover={h}>{"Hover me"}</div> };
}

#[test]
fn test_on_hover_event_snake_auto_id() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div on_hover={h}>{"Hover me"}</div> };
}

#[test]
fn test_on_drag_event_auto_id() {
    let h = |_: (), _: ()| {};
    // onDrag 事件应该自动注入 .id()
    let drag = ();
    let _el = rsx! { <div onDrag={(drag, h)}>{"Drag me"}</div> };
}

#[test]
fn test_on_drag_event_snake_auto_id() {
    let h = |_: (), _: ()| {};
    let drag = ();
    let _el = rsx! { <div on_drag={(drag, h)}>{"Drag me"}</div> };
}

#[test]
fn test_on_drop_event_auto_id() {
    let h = |_: (), _: ()| {};
    // onDrop 事件应该自动注入 .id()
    let _el = rsx! { <div onDrop={h}>{"Drop here"}</div> };
}

#[test]
fn test_on_drop_event_snake_auto_id() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div on_drop={h}>{"Drop here"}</div> };
}

// ===========================================================================
// 27. 颜色映射测试
// ===========================================================================

class_compile_tests! {
    test_class_color_red_500 => "bg-red-500",
    test_class_color_green_500 => "bg-green-500",
    test_class_color_gray_500 => "bg-gray-500",
    test_class_color_yellow_500 => "bg-yellow-500",
    test_class_color_purple_500 => "bg-purple-500",
    test_class_color_pink_500 => "bg-pink-500",
    test_class_color_indigo_500 => "bg-indigo-500",
    test_class_color_white => "bg-white",
    test_class_color_black => "text-black",
}

// ===========================================================================
// 28. 文本大小白名单测试
// ===========================================================================

class_compile_tests! {
    test_class_text_xs => "text-xs",
    test_class_text_base => "text-base",
    test_class_text_lg => "text-lg",
}

// ===========================================================================
// 29. 新属性名称映射
// ===========================================================================

#[test]
fn test_font_size_attribute() {
    let _el = rsx! { <div fontSize={px(16.0)} /> };
}

#[test]
fn test_line_height_attribute() {
    let _el = rsx! { <div lineHeight={px(24.0)} /> };
}

#[test]
fn test_font_weight_attribute() {
    let _el = rsx! { <div fontWeight={700} /> };
}

#[test]
fn test_common_jsx_style_attribute_aliases() {
    let _el = rsx! {
        <div
            fontFamily={"Inter"}
            textColor={rgb(0x111827)}
            backgroundColor={rgb(0xffffff)}
            borderColor={rgb(0xe5e7eb)}
        />
    };
}

#[test]
fn test_box_shadow_attribute() {
    let _el = rsx! { <div boxShadow={"shadow-lg"} /> };
}

#[test]
fn test_overflow_attribute() {
    // overflow 不再映射为 overflow_hidden，直接透传
    let _el = rsx! { <div overflow={true} /> };
}

// ===========================================================================
// 30. styled 标志属性 — 默认样式注入
// ===========================================================================

styled_tag_compile_tests! {
    test_styled_button => button,
    test_styled_h1 => h1,
    test_styled_h2 => h2,
    test_styled_h3 => h3,
    test_styled_h4 => h4,
    test_styled_h5 => h5,
    test_styled_h6 => h6,
    test_styled_a => a,
}

#[test]
fn test_styled_input() {
    // input + styled → px(2.0) py(1.0)
    let _el = rsx! { <input styled /> };
}

#[test]
fn test_styled_ul() {
    // ul + styled → flex flex_col
    let _el = rsx! {
        <ul styled>
            <li>{"item"}</li>
        </ul>
    };
}

#[test]
fn test_styled_override() {
    // 用户属性 text_color 在默认样式 text_3xl + font_bold 之后，可覆盖
    let _el = rsx! { <h1 styled text_color={rgb(0xff0000)}>{"Title"}</h1> };
}

#[test]
fn test_styled_with_class() {
    // styled + class 属性共存
    let _el = rsx! { <button styled class="flex items-center">{"Click"}</button> };
}

#[test]
fn test_unstyled_no_defaults() {
    // 不带 styled 时无默认样式（现有行为不变）
    let _el = rsx! { <button>{"Click"}</button> };
}

#[test]
fn test_styled_unknown_tag() {
    // 无默认样式表的标签加 styled 不报错，只是无额外样式
    let _el = rsx! { <div styled>{"content"}</div> };
}

// ===========================================================================
// 31. Fragment 支持 — 多根节点
// ===========================================================================

#[test]
fn test_fragment_basic() {
    let _els: Vec<MockElement> = rsx! {
        <>
            <div>{"first"}</div>
            <div>{"second"}</div>
        </>
    };
}

#[test]
fn test_fragment_multiple_elements() {
    let _els: Vec<MockElement> = rsx! {
        <>
            <div>{"a"}</div>
            <span>{"b"}</span>
            <p>{"c"}</p>
        </>
    };
}

#[test]
fn test_fragment_with_attrs() {
    let _els: Vec<MockElement> = rsx! {
        <>
            <div flex>{"item 1"}</div>
            <div flex_col>{"item 2"}</div>
        </>
    };
}

#[test]
fn test_fragment_mixed_types_with_explicit_any_elements() {
    let _els: Vec<AnyElement> = rsx! {
        <>
            {div().into_any_element()}
            {ui::TaskCard().into_any_element()}
        </>
    };
}

// ===========================================================================
// 32. 扩充 class 解析 — border、border-color
// ===========================================================================

#[test]
fn test_class_border() {
    // "border" → .border_1()
    let _el = rsx! { <div class="border" /> };
}

#[test]
fn test_class_border_2() {
    // "border-2" → .border_2()
    let _el = rsx! { <div class="border-2" /> };
}

#[test]
fn test_class_border_color() {
    // "border-red-500" → .border_color(rgb(0xef4444))
    let _el = rsx! { <div class="border border-red-500" /> };
}

#[test]
fn test_class_border_blue() {
    let _el = rsx! { <div class="border-blue-600" /> };
}

#[test]
fn test_class_overflow_hidden() {
    let _el = rsx! { <div class="overflow-hidden" /> };
}

#[test]
fn test_class_overflow_scroll() {
    let _el = rsx! { <div class="overflow-scroll" /> };
}

#[test]
fn test_strict_class_overflow_scroll_is_supported_static_stateful_class() {
    let _el = gpui_rsx::rsx_strict! { <div class="overflow-scroll" /> };
}

#[test]
fn test_class_rounded_variants() {
    let _a = rsx! { <div class="rounded-sm" /> };
    let _b = rsx! { <div class="rounded-md" /> };
    let _c = rsx! { <div class="rounded-lg" /> };
    let _d = rsx! { <div class="rounded-xl" /> };
    let _e = rsx! { <div class="rounded-full" /> };
    let _f = rsx! { <div class="rounded-none" /> };
}

#[test]
fn test_class_shadow_variants() {
    let _none = rsx! { <div class="shadow-none" /> };
    let _2xs = rsx! { <div class="shadow-2xs" /> };
    let _xs = rsx! { <div class="shadow-xs" /> };
    let _a = rsx! { <div class="shadow-sm" /> };
    let _b = rsx! { <div class="shadow-md" /> };
    let _c = rsx! { <div class="shadow-lg" /> };
    let _xl = rsx! { <div class="shadow-xl" /> };
    let _2xl = rsx! { <div class="shadow-2xl" /> };
}

#[test]
fn test_class_cursor_variants() {
    let _a = rsx! { <div class="cursor-pointer" /> };
    let _b = rsx! { <div class="cursor-default" /> };
    let _c = rsx! { <div class="cursor-text" /> };
}

#[test]
fn test_class_cursor_extra_variants() {
    let _a = rsx! { <div class="cursor-move" /> };
    let _b = rsx! { <div class="cursor-not-allowed" /> };
    let _c = rsx! { <div class="cursor-context-menu" /> };
    let _d = rsx! { <div class="cursor-crosshair" /> };
    let _e = rsx! { <div class="cursor-vertical-text" /> };
    let _f = rsx! { <div class="cursor-alias" /> };
    let _g = rsx! { <div class="cursor-copy" /> };
    let _h = rsx! { <div class="cursor-no-drop" /> };
    let _i = rsx! { <div class="cursor-grab" /> };
    let _j = rsx! { <div class="cursor-grabbing" /> };
    let _k = rsx! { <div class="cursor-ew-resize" /> };
    let _l = rsx! { <div class="cursor-ns-resize" /> };
    let _m = rsx! { <div class="cursor-nesw-resize" /> };
    let _n = rsx! { <div class="cursor-nwse-resize" /> };
    let _o = rsx! { <div class="cursor-col-resize" /> };
    let _p = rsx! { <div class="cursor-row-resize" /> };
    let _q = rsx! { <div class="cursor-n-resize" /> };
    let _r = rsx! { <div class="cursor-e-resize" /> };
    let _s = rsx! { <div class="cursor-s-resize" /> };
    let _t = rsx! { <div class="cursor-w-resize" /> };
}

#[test]
fn test_class_full_size() {
    let _a = rsx! { <div class="w-full" /> };
    let _b = rsx! { <div class="h-full" /> };
    let _c = rsx! { <div class="size-full" /> };
}

#[test]
fn test_class_fractional_and_alias_sizes() {
    let _a = rsx! { <div class="w-1/2 h-1/3" /> };
    let _b = rsx! { <div class="w-px h-px size-px" /> };
    let _c = rsx! { <div class="w-auto h-auto" /> };
    let _d = rsx! { <div class="size-1/2" /> };
}

#[test]
fn test_class_arbitrary_length_px_rem_percent() {
    let _el = rsx! {
        <div class="w-[280px] min-w-[280px] max-w-[32rem] h-[50%] p-[18px] gap-[0.75rem]" />
    };
}

#[test]
fn test_class_fractional_size_arbitrary_denominator() {
    let _el = rsx! { <div class="w-6/24 h-3/12 size-1/24 min-w-6/24 max-w-18/24" /> };
}

#[test]
fn test_class_numeric_sizes_keep_px_semantics() {
    // P0 must not reinterpret existing numeric classes as Tailwind rem scale.
    let _el = rsx! { <div class="w-64 h-12 p-4 gap-3" /> };
}

#[test]
fn test_class_font_weight_mapping_gpui() {
    take_font_weight_calls();

    let _el = rsx! {
        <div class="font-thin font-extralight font-light font-normal font-medium font-semibold font-bold font-extrabold font-black" />
    };

    assert_eq!(
        take_font_weight_calls(),
        vec![
            100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0
        ]
    );
}

#[test]
fn test_class_flex_variants() {
    let _a = rsx! { <div class="flex-none" /> };
    let _b = rsx! { <div class="flex-auto" /> };
    let _c = rsx! { <div class="flex-1" /> };
    let _d = rsx! { <div class="flex-grow-0" /> };
    let _e = rsx! { <div class="flex-grow" /> };
    let _f = rsx! { <div class="flex-grow-1" /> };
    let _g = rsx! { <div class="flex-shrink" /> };
    let _h = rsx! { <div class="flex-shrink-1" /> };
}

#[test]
fn test_class_gpui_helpers() {
    let _b = rsx! { <div class="aspect-square" /> };
    let _c = rsx! { <div class="content-normal self-center whitespace-nowrap line-clamp-2" /> };
    let _d = rsx! { <div class="col-span-full col-start-auto col-end-auto" /> };
    let _e = rsx! { <div class="row-span-full row-start-auto row-end-auto" /> };
}

#[test]
fn test_class_directional_rounded_variants() {
    let _a = rsx! { <div class="rounded-t-lg" /> };
    let _b = rsx! { <div class="rounded-b-lg" /> };
    let _c = rsx! { <div class="rounded-r-lg" /> };
    let _d = rsx! { <div class="rounded-l-lg" /> };
}

#[test]
fn test_class_self_alignment_variants() {
    let _a = rsx! { <div class="self-start" /> };
    let _b = rsx! { <div class="self-end" /> };
    let _c = rsx! { <div class="self-flex-start" /> };
    let _d = rsx! { <div class="self-flex-end" /> };
    let _e = rsx! { <div class="self-baseline" /> };
    let _f = rsx! { <div class="self-stretch" /> };
}

#[test]
fn test_class_whitespace_variants() {
    let _a = rsx! { <div class="whitespace-normal" /> };
    let _b = rsx! { <div class="whitespace-nowrap" /> };
}

#[test]
fn test_class_text_decoration_extra_variants() {
    let _a = rsx! { <div class="no-underline" /> };
    let _b = rsx! { <div class="text-decoration-solid" /> };
    let _c = rsx! { <div class="text-decoration-wavy" /> };
    let _d = rsx! { <div class="text-decoration-0" /> };
    let _e = rsx! { <div class="text-decoration-1" /> };
    let _f = rsx! { <div class="text-decoration-2" /> };
    let _g = rsx! { <div class="text-decoration-4" /> };
    let _h = rsx! { <div class="text-decoration-8" /> };
}

#[test]
fn test_class_overflow_axis_hidden() {
    let _a = rsx! { <div class="overflow-x-hidden" /> };
    let _b = rsx! { <div class="overflow-y-hidden" /> };
}
