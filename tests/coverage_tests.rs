//! 额外的代码覆盖率测试
//!
//! 这个文件包含针对边界情况和未覆盖代码路径的测试用例

mod common;

use common::*;
use gpui_rsx::rsx;

// ===========================================================================
// Class 解析边界情况
// ===========================================================================

#[test]
fn test_class_empty_string() {
    // 空 class 字符串应该不产生任何方法调用
    let _el = rsx! { <div class="">{"Empty class"}</div> };
}

#[test]
fn test_class_whitespace_only() {
    // 仅空格的 class 应该不产生任何方法调用
    let _el = rsx! { <div class="   ">{"Whitespace only"}</div> };
}

#[test]
fn test_class_leading_trailing_whitespace() {
    // 首尾空白应被 split_ascii_whitespace 丢弃，仍能正确解析中间的 class
    // "  flex gap-4  " → .flex().gap(px(4.0))
    let _el = rsx! { <div class="  flex gap-4  ">{"leading trailing"}</div> };
}

#[test]
fn test_class_consecutive_spaces() {
    // class 名之间的多个连续空格应视为单一分隔符
    // "flex    gap-4" → .flex().gap(px(4.0))
    let _el = rsx! { <div class="flex    gap-4">{"consecutive spaces"}</div> };
}

#[test]
fn test_class_with_non_ident_chars_ignored() {
    // Tailwind 变体语法（如 "hover:bg-blue-500"）含 ":" 不是合法 Rust 标识符，
    // 应静默跳过而非 panic，其余有效 class 正常应用。
    let _el = rsx! { <div class="flex hover:bg-blue-500">{"content"}</div> };
}

#[test]
fn test_class_hex_uppercase() {
    // Hex 颜色大写
    let _el = rsx! { <div class="bg-[#FF00FF]">{"Uppercase hex"}</div> };
}

#[test]
fn test_class_hex_mixed_case() {
    // Hex 颜色混合大小写
    let _el = rsx! { <div class="bg-[#FfAa00]">{"Mixed case hex"}</div> };
}

#[test]
fn test_class_spacing_zero() {
    // 间距类值为 0：suffix.parse::<f32>() 对 "0" 应返回 Ok(0.0)
    // p-0 → .p(px(0.0))，m-0 → .m(px(0.0))，gap-0 → .gap(px(0.0))
    let _el = rsx! { <div class="p-0 m-0 gap-0">{"zero spacing"}</div> };
}

#[test]
fn test_class_all_spacing_variants() {
    // 测试所有间距变体
    let _el = rsx! {
        <div class="gap-2 gap-4 gap-6">
            {"Spacing variants"}
        </div>
    };
}

#[test]
fn test_class_all_padding_variants() {
    // 测试所有 padding 变体
    let _el = rsx! {
        <div class="p-2 p-4">
            {"Padding variants"}
        </div>
    };
}

#[test]
fn test_class_multiple_colors_same_element() {
    // 多个颜色 class 在同一元素（后面的会覆盖）
    let _el = rsx! {
        <div class="bg-red-500 bg-blue-500 text-white text-black">
            {"Multiple colors"}
        </div>
    };
}

// ===========================================================================
// 动态 Class 边界情况
// ===========================================================================

// 注意：动态 class 的详细测试已在 macro_tests.rs 中覆盖
// 这里仅测试编译期静态 class 的边界情况

// ===========================================================================
// 自动 ID 确定性
// ===========================================================================

#[test]
fn test_auto_id_deterministic_same_signature() {
    let h = |_: (), _: ()| {};
    // 相同签名的元素应该生成不同的 ID（因为有计数器）
    let _el1 = rsx! { <button on_click={h}>{"Button 1"}</button> };
    let _el2 = rsx! { <button on_click={h}>{"Button 2"}</button> };
}

#[test]
fn test_auto_id_with_multiple_stateful_attrs() {
    let h = |_: (), _: ()| {};
    // on_click 是真正的 stateful 属性（StatefulInteractiveElement），触发 auto ID 注入。
    // hover / active 是 Styled trait 的样式方法，不触发 auto ID。
    let _el = rsx! {
        <div
            on_click={h}
            hover={|el| el.bg(rgb(0x3b82f6))}
            active={|el| el.bg(rgb(0x2563eb))}
        >
            {"Multiple stateful"}
        </div>
    };
}

#[test]
fn test_auto_id_all_stateful_attributes() {
    let h = |_: (), _: ()| {};
    // 真正需要 auto ID 的属性：on_click/on_hover/on_drag、tooltip、focusable 等。
    // hover / focus / group 是 Styled trait 方法，不需要 ID，
    // 但可以和 stateful 元素共存（元素因 on_click 已获得 ID）。
    let _el = rsx! {
        <div
            on_click={h}
            hover={|el| el}
            active={|el| el}
            focus={|el| el}
            tooltip={"tip"}
            group={"group1"}
            focusable
            overflow_scroll
        >
            {"All stateful attrs"}
        </div>
    };
}

#[test]
fn test_no_auto_id_without_stateful() {
    // 没有 stateful 属性时不应该生成 ID
    let _el = rsx! {
        <div flex gap={px(4.0)} bg={rgb(0xffffff)}>
            {"No stateful"}
        </div>
    };
}

#[test]
fn test_user_id_prevents_auto_id() {
    let h = |_: (), _: ()| {};
    // 用户提供 ID 时不应该生成自动 ID
    let _el = rsx! {
        <div id="my-custom-id" on_click={h}>
            {"User ID"}
        </div>
    };
}

// ===========================================================================
// 子节点聚合边界
// ===========================================================================

#[test]
fn test_children_single_expr() {
    // 单个表达式子节点（最小边界）
    let _el = rsx! {
        <div>
            {"single"}
        </div>
    };
}

#[test]
fn test_children_exactly_two_exprs() {
    // 恰好 2 个表达式（聚合阈值下界）
    let _el = rsx! {
        <div>
            {"first"}
            {"second"}
        </div>
    };
}

#[test]
fn test_children_exactly_three_exprs() {
    // 恰好 3 个表达式（聚合阈值上界，应触发 .children()）
    let _el = rsx! {
        <div>
            {"first"}
            {"second"}
            {"third"}
        </div>
    };
}

#[test]
fn test_children_four_exprs() {
    // 4 个表达式（应触发 .children()）
    let _el = rsx! {
        <div>
            {"first"}
            {"second"}
            {"third"}
            {"fourth"}
        </div>
    };
}

#[test]
fn test_children_mixed_types_complex() {
    // 复杂的混合类型子节点
    let _el = rsx! {
        <div>
            {"text1"}
            <span>{"element1"}</span>
            {"text2"}
            {"text3"}
            {"text4"}
            <div>{"element2"}</div>
            {"text5"}
        </div>
    };
}

#[test]
fn test_children_element_between_exprs() {
    // 元素节点打断连续的表达式
    let _el = rsx! {
        <div>
            {"expr1"}
            {"expr2"}
            <span>{"element"}</span>
            {"expr3"}
            {"expr4"}
            {"expr5"}
        </div>
    };
}

// ===========================================================================
// Styled 标志高级场景
// ===========================================================================

#[test]
fn test_styled_with_override() {
    // styled 默认样式被用户属性覆盖
    let _el = rsx! {
        <h1 styled text_color={rgb(0xff0000)} font_bold>
            {"Overridden"}
        </h1>
    };
}

#[test]
fn test_styled_multiple_per_element() {
    // styled 标志与其他标志组合
    let _el = rsx! { <button styled flex>{"content"}</button> };
}

// ===========================================================================
// When/WhenSome 边界情况
// ===========================================================================

#[test]
fn test_when_condition_false() {
    // when 条件为 false 时不应用样式
    let is_active = false;
    let _el = rsx! {
        <div when={(is_active, |el| el.bg(rgb(0x3b82f6)))}>
            {"Inactive"}
        </div>
    };
}

#[test]
fn test_when_condition_true() {
    // when 条件为 true 时应用样式
    let is_active = true;
    let _el = rsx! {
        <div when={(is_active, |el| el.bg(rgb(0x3b82f6)))}>
            {"Active"}
        </div>
    };
}

#[test]
fn test_when_some_none_value() {
    // whenSome 值为 None 时不应用样式
    let value: Option<f32> = None;
    let _el = rsx! {
        <div whenSome={(value, |el, v| el.w(px(v)))}>
            {"No value"}
        </div>
    };
}

#[test]
fn test_when_some_some_value() {
    // whenSome 值为 Some 时应用样式
    let value: Option<f32> = Some(200.0);
    let _el = rsx! {
        <div whenSome={(value, |el, v| el.w(px(v)))}>
            {"Has value"}
        </div>
    };
}

#[test]
fn test_multiple_when_same_element() {
    // 多个 when 在同一元素上
    let cond1 = true;
    let cond2 = false;
    let cond3 = true;
    let _el = rsx! {
        <div
            when={(cond1, |el| el.flex())}
            when={(cond2, |el| el.gap(px(4.0)))}
            when={(cond3, |el| el.p(px(4.0)))}
        >
            {"Multiple when"}
        </div>
    };
}

#[test]
fn test_when_and_when_some_mixed() {
    // when 和 whenSome 混合使用
    let cond = true;
    let opt_value: Option<u32> = Some(0xff0000);
    let _el = rsx! {
        <div
            when={(cond, |el| el.flex())}
            whenSome={(opt_value, |el, color| el.bg(rgb(color)))}
        >
            {"Mixed when/whenSome"}
        </div>
    };
}

// ===========================================================================
// For 循环边界情况
// ===========================================================================

#[test]
fn test_for_loop_empty_iterator() {
    // 空迭代器
    let items: Vec<&str> = vec![];
    let _el = rsx! {
        <ul>
            {for item in &items {
                <li>{*item}</li>
            }}
        </ul>
    };
}

#[test]
fn test_for_loop_single_item() {
    // 单个元素
    let items = vec!["only"];
    let _el = rsx! {
        <ul>
            {for item in &items {
                <li>{*item}</li>
            }}
        </ul>
    };
}

#[test]
fn test_for_loop_nested() {
    // 嵌套 for 循环
    let outer = vec![vec![1, 2], vec![3, 4]];
    let _el = rsx! {
        <div>
            {for row in &outer {
                <div>
                    {for cell in row {
                        <span>{*cell}</span>
                    }}
                </div>
            }}
        </div>
    };
}

#[test]
fn test_for_loop_with_filter() {
    // for 循环与迭代器链
    let items = [1, 2, 3, 4, 5];
    let _el = rsx! {
        <div>
            {for item in items.iter().filter(|&&x| x > 2) {
                <span>{*item}</span>
            }}
        </div>
    };
}

#[test]
fn test_for_loop_tuple_destructuring() {
    // for 循环元组解构
    let items = vec![(1, "one"), (2, "two"), (3, "three")];
    let _el = rsx! {
        <div>
            {for (num, name) in &items {
                <div>{format!("{}: {}", num, name)}</div>
            }}
        </div>
    };
}

#[test]
fn test_for_loop_multiple_body_nodes() {
    // for 循环体内有 2 个子节点时触发 flat_map 路径（generate_for_loop 中 body_exprs.len() > 1）
    // 生成：(items).into_iter().flat_map(|item| vec![child1, child2])
    let items = ["a", "b", "c"];
    let _el = rsx! {
        <ul>
            {for item in items {
                <li>{"label"}</li>
                <li>{item}</li>
            }}
        </ul>
    };
}
