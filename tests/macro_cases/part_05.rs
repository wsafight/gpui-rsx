use super::*;

// ===========================================================================
// 33. 扩充颜色系统 — 新色阶和任意 hex
// ===========================================================================

#[test]
fn test_class_color_new_shades() {
    // 新色阶测试
    let _a = rsx! { <div class="bg-red-100" /> };
    let _b = rsx! { <div class="bg-red-300" /> };
    let _c = rsx! { <div class="bg-red-700" /> };
    let _d = rsx! { <div class="bg-red-900" /> };
    let _e = rsx! { <div class="bg-red-950" /> };
}

#[test]
fn test_class_color_new_colors() {
    // 新颜色测试
    let _a = rsx! { <div class="bg-slate-500" /> };
    let _b = rsx! { <div class="bg-emerald-500" /> };
    let _c = rsx! { <div class="bg-teal-500" /> };
    let _d = rsx! { <div class="bg-cyan-500" /> };
    let _e = rsx! { <div class="bg-sky-500" /> };
    let _f = rsx! { <div class="bg-violet-500" /> };
    let _g = rsx! { <div class="bg-fuchsia-500" /> };
    let _h = rsx! { <div class="bg-rose-500" /> };
}

#[test]
fn test_class_color_text_new_colors() {
    let _a = rsx! { <div class="text-orange-500" /> };
    let _b = rsx! { <div class="text-amber-500" /> };
    let _c = rsx! { <div class="text-lime-500" /> };
}

#[test]
fn test_class_arbitrary_hex_bg() {
    // bg-[#ff0000] → .bg(rgb(0xff0000))
    let _el = rsx! { <div class="bg-[#ff0000]" /> };
}

#[test]
fn test_class_arbitrary_hex_text() {
    // text-[#333333] → .text_color(rgb(0x333333))
    let _el = rsx! { <div class="text-[#333333]" /> };
}

#[test]
fn test_class_arbitrary_hex_short() {
    // text-[#abc] → .text_color(rgb(0xaabbcc))
    let _el = rsx! { <div class="text-[#abc]" /> };
}

#[test]
fn test_class_arbitrary_hex_border() {
    // border-[#ff0000] → .border_color(rgb(0xff0000))
    let _el = rsx! { <div class="border-[#ff0000]" /> };
}

#[test]
fn test_class_arbitrary_rgba_hex_colors() {
    take_rgba_calls();

    let _a = rsx! { <div class="bg-[#11223344]" /> };
    let _b = rsx! { <div class="text-[#abcdef80]" /> };
    let _c = rsx! { <div class="border-[#0102037f]" /> };

    assert_eq!(take_rgba_calls(), vec![0x11223344, 0xabcdef80, 0x0102037f]);
}

#[test]
fn test_class_arbitrary_rgb_function_colors() {
    take_rgb_calls();
    take_rgba_calls();

    let _a = rsx! { <div class="bg-[rgb(15,23,42)]" /> };
    let _b = rsx! { <div class="text-[rgb(255,255,255)]" /> };
    let _c = rsx! { <div class="border-[rgba(15,23,42,0.8)]" /> };

    assert_eq!(take_rgb_calls(), vec![0x0f172a, 0xffffff]);
    assert_eq!(take_rgba_calls(), vec![0x0f172acc]);
}

#[test]
fn test_class_debug_outline() {
    let _el = rsx! { <div class="debug-outline" /> };
}

// ===========================================================================
// 34. 新属性映射测试
// ===========================================================================

#[test]
fn test_gap_x_attribute() {
    let _el = rsx! { <div gapX={px(8.0)} /> };
}

#[test]
fn test_gap_y_attribute() {
    let _el = rsx! { <div gapY={px(8.0)} /> };
}

#[test]
fn test_flex_basis_attribute() {
    let _el = rsx! { <div flexBasis={px(100.0)} /> };
}

#[test]
fn test_flex_grow_flag_attribute() {
    let _el = rsx! { <div flexGrow /> };
}

#[test]
fn test_flex_shrink_flag_attribute() {
    let _el = rsx! { <div flexShrink /> };
}

flag_attr_compile_tests! {
    test_border_top_attribute => border_t,
    test_border_bottom_attribute => border_b,
    test_border_left_attribute => border_l,
    test_border_right_attribute => border_r,
}

value_attr_compile_tests! {
    test_border_top_value_attribute => border_t = px(1.0),
    test_border_bottom_value_attribute => border_b = px(1.0),
    test_border_left_value_attribute => border_l = px(1.0),
    test_border_right_value_attribute => border_r = px(1.0),
}

class_compile_tests! {
    test_class_border_t => "border-t",
    test_class_border_b => "border-b",
    test_class_border_l => "border-l",
    test_class_border_r => "border-r",
}

#[test]
fn test_rounded_top_left_attribute() {
    let _el = rsx! { <div roundedTopLeft={px(8.0)} /> };
}

#[test]
fn test_rounded_bottom_right_attribute() {
    let _el = rsx! { <div roundedBottomRight={px(8.0)} /> };
}

#[test]
fn test_inset_attribute() {
    let _el = rsx! { <div inset={px(0.0)} /> };
}

// ===========================================================================
// 35. 新事件处理器
// ===========================================================================

#[test]
fn test_on_mouse_down_out_camel() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onMouseDownOut={h} /> };
}

#[test]
fn test_on_mouse_down_out_snake() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div on_mouse_down_out={h} /> };
}

#[test]
fn test_on_mouse_up_out_camel() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onMouseUpOut={(MouseButton::Left, h)} /> };
}

#[test]
fn test_on_mouse_up_out_snake() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div on_mouse_up_out={(MouseButton::Left, h)} /> };
}

// ===========================================================================
// 36. Children 聚合优化
// ===========================================================================

#[test]
fn test_children_aggregation_3_or_more() {
    // 3 个连续 Expr → .children(vec![...]) 而非 3 个 .child()
    let _el = rsx! {
        <div>
            {"first"}
            {"second"}
            {"third"}
        </div>
    };
}

#[test]
fn test_children_aggregation_mixed() {
    // 混合节点不影响聚合逻辑
    let _el = rsx! {
        <div>
            {"a"}
            {"b"}
            {"c"}
            <span>{"element"}</span>
            {"d"}
            {"e"}
        </div>
    };
}

#[test]
fn test_children_aggregation_under_3() {
    // 2 个 Expr 不触发聚合，仍用 .child()
    let _el = rsx! {
        <div>
            {"first"}
            {"second"}
        </div>
    };
}

// ===========================================================================
// 37. For 循环语法糖
// ===========================================================================

#[test]
fn test_for_loop_basic() {
    let items = ["apple", "banana", "cherry"];
    let _el = rsx! {
        <ul>
            {for item in items {
                <li>{item}</li>
            }}
        </ul>
    };
}

#[test]
fn test_for_loop_with_range() {
    let _el = rsx! {
        <div>
            {for i in 0..3 {
                <span>{i}</span>
            }}
        </div>
    };
}

#[test]
fn test_for_loop_with_method() {
    let items = ["a", "b", "c"];
    let _el = rsx! {
        <div>
            {for item in items.iter() {
                <span>{item}</span>
            }}
        </div>
    };
}

// ===========================================================================
// 38. Auto-ID 确定性（编译通过即验证）
// ===========================================================================

#[test]
fn test_auto_id_hash_format() {
    let h = |_: (), _: ()| {};
    // 编译通过即验证 auto-ID 使用 hash 格式
    let _el = rsx! {
        <div>
            <button onClick={h}>{"A"}</button>
        </div>
    };
}

#[test]
fn test_auto_id_multiple_unique() {
    let h1 = |_: (), _: ()| {};
    let h2 = |_: (), _: ()| {};
    // 两个按钮各自有唯一 auto-ID
    let _el = rsx! {
        <div>
            <button onClick={h1}>{"Btn 1"}</button>
            <button onClick={h2}>{"Btn 2"}</button>
        </div>
    };
}

// ===========================================================================
// 39. size-N 类名（bug 修复验证：size-4 → .size(px(4.0))）
// ===========================================================================

#[test]
fn test_class_size_numeric() {
    let _a = rsx! { <div class="size-4" /> };
    let _b = rsx! { <div class="size-8" /> };
    let _c = rsx! { <div class="size-16" /> };
}

#[test]
fn test_class_size_combined() {
    // size-N 与其他间距类同时使用
    let _el = rsx! { <div class="size-4 gap-4" /> };
}

// ===========================================================================
// 40. opacity-* 类名
// ===========================================================================

#[test]
fn test_class_opacity_values() {
    // opacity-N → .opacity(N / 100.0)
    let _a = rsx! { <div class="opacity-0" /> }; // → .opacity(0.0)
    let _b = rsx! { <div class="opacity-50" /> }; // → .opacity(0.5)
    let _c = rsx! { <div class="opacity-100" /> }; // → .opacity(1.0)
}

#[test]
fn test_class_opacity_combined() {
    let _el = rsx! { <div class="opacity-75 flex" /> };
}

// ===========================================================================
// 41. styled 新标签默认样式（li、p、label、form）
// ===========================================================================

#[test]
fn test_styled_li() {
    // li styled → "flex items-center" → .flex().items_center()
    let _el = rsx! { <li styled>{"list item"}</li> };
}

#[test]
fn test_styled_p() {
    // p styled → "text-base" → .text_base()
    let _el = rsx! { <p styled>{"paragraph"}</p> };
}

#[test]
fn test_styled_label() {
    // label styled → "text-sm" → .text_sm()
    let _el = rsx! { <label styled>{"label text"}</label> };
}

#[test]
fn test_styled_form() {
    // form styled → "flex flex-col gap-4" → .flex().flex_col().gap(px(4.0))
    let _el = rsx! { <form styled>{"form content"}</form> };
}

#[test]
fn test_styled_new_tags_with_class_override() {
    // styled 默认样式可被 class 属性追加覆盖
    let _el = rsx! { <li styled class="p-4">{"item"}</li> };
}

// ===========================================================================
// 42. 动态 class 新增条目（gap 更多值、方向性 padding/margin）
// ===========================================================================

#[test]
fn test_dynamic_class_new_gap_values() {
    // gap-5 / gap-10 / gap-12 是新增条目
    let cls = "gap-5";
    let _a = rsx! { <div class={cls} /> };
    let cls = "gap-10";
    let _b = rsx! { <div class={cls} /> };
    let cls = "gap-12";
    let _c = rsx! { <div class={cls} /> };
}

#[test]
fn test_dynamic_class_direction_padding() {
    // pt-N, pb-N, pl-N, pr-N 是新增条目
    let cls = "pt-4 pb-2";
    let _a = rsx! { <div class={cls} /> };
    let cls = "pl-4 pr-2";
    let _b = rsx! { <div class={cls} /> };
}

#[test]
fn test_dynamic_class_direction_margin() {
    // mt-N, mb-N, mx-N, my-N 是新增条目
    let cls = "mt-4 mb-2";
    let _a = rsx! { <div class={cls} /> };
    let cls = "mx-4 my-2";
    let _b = rsx! { <div class={cls} /> };
}

#[test]
fn test_dynamic_class_unknown_ignored_no_panic() {
    // 不在预定义列表的 class 被静默忽略，不 panic
    let cls = "not-a-class-xyz";
    let _el = rsx! { <div class={cls} /> };
}

#[test]
fn test_dynamic_class_arbitrary_hex_colors() {
    // 动态 class 也支持 arbitrary hex 颜色，和静态 class 路径保持一致。
    take_rgb_calls();
    take_rgba_calls();

    let cls = "bg-[#ff0000]";
    let _a = rsx! { <div class={cls} /> };
    let cls = "text-[#abc]";
    let _b = rsx! { <div class={cls} /> };
    let cls = "border-[#333333]";
    let _c = rsx! { <div class={cls} /> };
    let cls = "bg-[#11223344]";
    let _d = rsx! { <div class={cls} /> };
    let cls = "text-[rgba(15,23,42,0.8)]";
    let _e = rsx! { <div class={cls} /> };
    let cls = "border-[rgb(1,2,3)]";
    let _f = rsx! { <div class={cls} /> };

    assert_eq!(
        take_rgb_calls(),
        vec![0xff0000, 0xaabbcc, 0x333333, 0x010203]
    );
    assert_eq!(take_rgba_calls(), vec![0x11223344, 0x0f172acc]);
}

// ===========================================================================
// 43. Auto-ID span 位置稳定性
// ===========================================================================

#[test]
fn test_auto_id_span_format_exact() {
    // 验证 auto-ID 精确格式：{file}::__rsx_{tag}_L{line}C{col}
    //
    // 列号固定为 22："    let _el = rsx! { <" = 22 个字符（0-indexed）
    // 行号用 line!() + 1 在编译时精确定位，无需硬编码绝对行号。
    //
    // ⚠️ 若在本行与 rsx! 行之间插入新行，需将 line!() + 1 的偏移量同步更新。
    let h = |_: (), _: ()| {};
    let expected_line = line!() + 1; // 下一行就是 rsx! 调用
    let _el = rsx! { <div on_click={h} /> }; // div 在列 22
    let captured = take_last_auto_id().expect("on_click 应触发 auto-ID 生成");
    let expected = format!("{}::__rsx_div_L{expected_line}C22", file!());
    assert_eq!(
        captured, expected,
        "auto-ID 格式应为 {{file}}::__rsx_{{tag}}_L{{line}}C{{col}}\n期望: {expected}\n实际: {captured}"
    );
}

#[test]
fn test_auto_id_tag_name_in_id() {
    // 标签名应出现在 auto-ID 中
    let h = |_: (), _: ()| {};
    let _el = rsx! { <button on_click={h} /> };
    let id = take_last_auto_id().expect("应生成 auto-ID");
    assert!(
        id.contains("__rsx_button_L"),
        "button 的 ID 应包含 __rsx_button_L，实际: {id}"
    );
}

#[test]
fn test_auto_id_different_lines_get_different_ids() {
    // 不同行的 rsx! 调用应生成不同 auto-ID（行号不同）
    let h = |_: (), _: ()| {};
    let _a = rsx! { <div on_click={h} /> };
    let id_a = take_last_auto_id().unwrap();
    let _b = rsx! { <div on_click={h} /> };
    let id_b = take_last_auto_id().unwrap();
    // 两行行号不同 → ID 不同
    assert_ne!(id_a, id_b, "不同行的元素应有不同 auto-ID");
    // 两个 ID 都应包含行列信息
    assert!(
        id_a.contains("_L") && id_a.contains('C'),
        "ID 格式应含 _L 和 C: {id_a}"
    );
    assert!(
        id_b.contains("_L") && id_b.contains('C'),
        "ID 格式应含 _L 和 C: {id_b}"
    );
}

#[test]
fn test_auto_id_no_id_for_non_stateful() {
    // 非 stateful 元素不触发 auto-ID 生成，LAST_AUTO_ID 应为空
    take_last_auto_id(); // 清空上一次的残留
    let _el = rsx! { <div flex gap={px(4.0)} /> };
    let captured = take_last_auto_id();
    assert!(captured.is_none(), "无 stateful 属性时不应生成 auto-ID");
}

#[test]
fn test_keyed_auto_id_literal_uses_static_concat() {
    let expanded = gpui_rsx::rsx_expand! {
        <button key="primary" onClick={h} />
    };
    let compact = expanded.split_whitespace().collect::<String>();

    assert!(compact.contains("concat!(file!(),"));
    assert!(compact.contains("\"primary\""));
    assert!(!compact.contains("format!"));
}

#[test]
fn test_keyed_auto_id_dynamic_key_keeps_format_fallback() {
    let expanded = gpui_rsx::rsx_expand! {
        <button key={item.id} onClick={h} />
    };
    let compact = expanded.split_whitespace().collect::<String>();

    assert!(compact.contains("format!(concat!(file!(),"));
    assert!(compact.contains("item.id"));
}
