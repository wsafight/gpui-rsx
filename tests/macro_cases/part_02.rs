use super::*;

// ===========================================================================
// 14. 自动 ID（onClick 需要 StatefulInteractiveElement）
// ===========================================================================

#[test]
fn test_on_click_auto_generates_id() {
    let h = |_: (), _: ()| {};
    // onClick → 自动插入 .id("__rsx_N")
    let _el = rsx! { <button onClick={h}>{"Click"}</button> };
}

#[test]
fn test_user_provided_id_with_on_click() {
    let h = |_: (), _: ()| {};
    // 用户显式提供 id 时不自动生成
    let _el = rsx! { <button id="my-btn" onClick={h}>{"Click"}</button> };
}

#[test]
fn test_no_auto_id_without_stateful_events() {
    // 没有 onClick 时不生成 .id()
    let _el = rsx! { <div flex>{"No events"}</div> };
}

#[test]
fn test_multiple_on_click_unique_ids() {
    let h1 = |_: (), _: ()| {};
    let h2 = |_: (), _: ()| {};
    let h3 = |_: (), _: ()| {};
    let _el = rsx! {
        <div>
            <button onClick={h1}>{"Button 1"}</button>
            <button onClick={h2}>{"Button 2"}</button>
            <button onClick={h3}>{"Button 3"}</button>
        </div>
    };
}

#[test]
fn test_on_mouse_down_no_auto_id() {
    let h = |_: (), _: ()| {};
    // onMouseDown 不需要 StatefulInteractiveElement，不生成 .id()
    let _el = rsx! { <div onMouseDown={(MouseButton::Left, h)} /> };
}

#[test]
fn test_track_focus_camel_case_no_auto_id() {
    take_last_auto_id();
    let _el = rsx! { <div trackFocus={|_: (), _: ()| {}} /> };
    let captured = take_last_auto_id();
    assert!(
        captured.is_none(),
        "trackFocus 映射到 InteractiveElement::track_focus，不应生成 auto-ID"
    );
}

// ===========================================================================
// 15. id 属性
// ===========================================================================

#[test]
fn test_id_with_string_literal() {
    let _el = rsx! { <div id="main-container">{"content"}</div> };
}

#[test]
fn test_id_with_expression() {
    let my_id = "dynamic-id";
    let _el = rsx! { <div id={my_id}>{"content"}</div> };
}

#[test]
fn test_id_without_events() {
    // 可以单独使用 id，不一定配合事件
    let _el = rsx! { <div id="solo-id" flex>{"content"}</div> };
}

// ===========================================================================
// 16. 迭代器子节点
// ===========================================================================

#[test]
fn test_iterator_map_children() {
    let items = ["apple", "banana", "cherry"];
    let _el = rsx! {
        <ul>
            {items.iter().map(|item| {
                rsx! { <li>{*item}</li> }
            }).collect::<Vec<_>>()}
        </ul>
    };
}

#[test]
fn test_range_iterator_children() {
    let _el = rsx! {
        <div>
            {(0..3).map(|i| {
                rsx! { <span>{i}</span> }
            }).collect::<Vec<_>>()}
        </div>
    };
}

// ===========================================================================
// 17. 综合场景
// ===========================================================================

#[test]
fn test_counter_ui() {
    let count = 5;
    let handler = |_: (), _: ()| {};
    let _el = rsx! {
        <div class="flex flex-col gap-4 p-4" bg={rgb(0xf3f4f6)}>
            <div class="text-2xl font-bold">
                {format!("Count: {count}")}
            </div>
            <div class="flex gap-2">
                <button
                    bg={rgb(0x3b82f6)}
                    text_color={rgb(0xffffff)}
                    px_4
                    py_2
                    rounded_md
                    onClick={handler}
                >
                    {"Increment"}
                </button>
            </div>
            {if count > 0 {
                rsx! { <div class="text-green-600">{"Positive!"}</div> }
            } else {
                rsx! { <div class="text-red-600">{"Negative or zero"}</div> }
            }}
        </div>
    };
}

#[test]
fn test_todo_item_ui() {
    let completed = true;
    let text = "Buy groceries";
    let toggle = |_: (), _: ()| {};
    let delete = |_: (), _: ()| {};

    let _el = rsx! {
        <div
            class="flex gap-3 items-center p-3 rounded-md"
            bg={if completed { rgb(0xf3f4f6) } else { rgb(0xffffff) }}
        >
            <div
                w={px(20.0)}
                h={px(20.0)}
                rounded={px(4.0)}
                bg={if completed { rgb(0x3b82f6) } else { rgb(0xe5e7eb) }}
                onClick={toggle}
            />
            <div
                class="flex-1"
                text_color={if completed { rgb(0x9ca3af) } else { rgb(0x1f2937) }}
            >
                {text}
            </div>
            <button
                bg={rgb(0xef4444)}
                text_color={rgb(0xffffff)}
                px_3
                py_1
                rounded_md
                onClick={delete}
            >
                {"Delete"}
            </button>
        </div>
    };
}

#[test]
fn test_app_layout() {
    let menu = |_: (), _: ()| {};
    let _el = rsx! {
        <div flex flex_col>
            <header class="flex items-center justify-between p-4">
                <h1 class="text-2xl font-bold">{"App Title"}</h1>
                <button onClick={menu} rounded_md px_4 py_2>
                    {"Menu"}
                </button>
            </header>
            <main class="flex-1 p-4">
                <section>
                    <p>{"Content goes here"}</p>
                </section>
            </main>
            <footer class="p-4">
                <p class="text-sm text-gray-600">{"Footer"}</p>
            </footer>
        </div>
    };
}

#[test]
fn test_many_attrs_on_one_element() {
    let h = |_: (), _: ()| {};
    let _el = rsx! {
        <button
            id="styled-btn"
            flex
            items_center
            justify_center
            bg={rgb(0x3b82f6)}
            text_color={rgb(0xffffff)}
            px_6
            py_2
            rounded_lg
            cursor_pointer
            border_1
            border_color={rgb(0x2563eb)}
            onClick={h}
        >
            {"Styled Button"}
        </button>
    };
}

#[test]
fn test_nested_conditional_elements() {
    let logged_in = true;
    let is_admin = false;
    let _el = rsx! {
        <div>
            {if logged_in {
                rsx! {
                    <div>
                        {if is_admin {
                            rsx! { <span>{"Admin Panel"}</span> }
                        } else {
                            rsx! { <span>{"User Dashboard"}</span> }
                        }}
                    </div>
                }
            } else {
                rsx! { <div>{"Please log in"}</div> }
            }}
        </div>
    };
}

// ===========================================================================
// 18. 常用属性简写
// ===========================================================================

#[test]
fn test_opacity_attribute() {
    let _el = rsx! {
        <div opacity={0.5}>{"半透明内容"}</div>
    };
}

#[test]
fn test_visible_attribute() {
    let is_shown = true;
    let _el = rsx! {
        <div visible={is_shown}>{"可见内容"}</div>
    };
}

#[test]
fn test_visible_false() {
    let _el = rsx! {
        <div visible={false}>{"隐藏内容"}</div>
    };
}

#[test]
fn test_visible_condition_evaluates_once() {
    let mut calls = 0;
    let _el = rsx! {
        <div visible={{
            calls += 1;
            true
        }}>
            {"可见内容"}
        </div>
    };
    assert_eq!(calls, 1);
}

#[test]
fn test_positioning_attributes() {
    let _el = rsx! {
        <div
            absolute
            top={px(10.0)}
            left={px(20.0)}
            right={px(30.0)}
            bottom={px(40.0)}
        >
            {"定位元素"}
        </div>
    };
}

#[test]
fn test_size_attributes() {
    let _el = rsx! {
        <div
            width={px(200.0)}
            height={px(100.0)}
            minWidth={px(100.0)}
            minHeight={px(50.0)}
            maxWidth={px(400.0)}
            maxHeight={px(200.0)}
        >
            {"尺寸控制"}
        </div>
    };
}

#[test]
fn test_combined_common_attributes() {
    let is_visible = true;
    let _el = rsx! {
        <div
            class="flex items-center"
            opacity={0.8}
            visible={is_visible}
            absolute
            top={px(0.0)}
            left={px(0.0)}
        >
            {"组合属性"}
        </div>
    };
}

#[test]
fn test_modal_overlay_example() {
    let is_open = true;
    let _el = rsx! {
        <div
            absolute
            top={px(0.0)}
            left={px(0.0)}
            width={px(100.0)}
            height={px(100.0)}
            bg={rgb(0x000000)}
            opacity={0.5}
            visible={is_open}
        >
            {"模态框遮罩"}
        </div>
    };
}

// ===========================================================================
// 19. map 方法 - 直接使用表达式
// ===========================================================================

#[test]
fn test_map_with_expression() {
    let is_active = true;
    let _el = rsx! {
        <div>
            {div()
                .flex()
                .map(|this| {
                    if is_active {
                        this.bg(rgb(0x3b82f6))
                    } else {
                        this
                    }
                })
                .child("content")
            }
        </div>
    };
}

#[test]
fn test_map_with_rsx_element() {
    let is_highlighted = true;
    let _el = rsx! {
        <div>
            {
                rsx! { <div flex px_4 py_2 /> }
                    .map(|this| {
                        if is_highlighted {
                            this.bg(rgb(0xfef3c7))
                        } else {
                            this
                        }
                    })
                    .child("Button")
            }
        </div>
    };
}

// ===========================================================================
// 20. when 条件渲染
// ===========================================================================

#[test]
fn test_when_basic() {
    let is_active = true;
    let _el = rsx! {
        <div
            flex
            when={(is_active, |this| this.bg(rgb(0x3b82f6)))}
        >
            {"Content"}
        </div>
    };
}

#[test]
fn test_when_with_multiple_methods() {
    let show_border = true;
    let _el = rsx! {
        <div
            flex
            when={(show_border, |this| {
                this.border_1()
                    .border_color(rgb(0xe5e7eb))
                    .rounded_md()
            })}
        >
            {"Content"}
        </div>
    };
}

#[test]
fn test_when_false_condition() {
    let is_highlighted = false;
    let _el = rsx! {
        <div
            flex
            when={(is_highlighted, |this| this.bg(rgb(0xfef3c7)))}
        >
            {"Not highlighted"}
        </div>
    };
}

#[test]
fn test_multiple_when_on_same_element() {
    let is_active = true;
    let is_large = false;
    let _el = rsx! {
        <div
            flex
            when={(is_active, |this| this.bg(rgb(0x3b82f6)))}
            when={(is_large, |this| this.text_2xl())}
        >
            {"Content"}
        </div>
    };
}

#[test]
fn test_when_class_static_literal() {
    let active = true;
    let _el = rsx! {
        <div whenClass={(active, "bg-neutral-900 text-white px-2")} />
    };
}

#[test]
fn test_multiple_when_class_attributes() {
    let active = true;
    let _el = rsx! {
        <div
            class="flex"
            whenClass={(active, "bg-neutral-900 text-white")}
            whenClass={(!active, "text-neutral-600")}
        />
    };
}

#[test]
fn test_when_with_class_and_other_attrs() {
    let has_shadow = true;
    let _el = rsx! {
        <div
            class="flex flex-col gap-4 p-4"
            bg={rgb(0xffffff)}
            when={(has_shadow, |this| this.rounded_lg())}
        >
            {"Card content"}
        </div>
    };
}

#[test]
fn test_when_with_expression_condition() {
    let count = 5;
    let _el = rsx! {
        <div
            flex
            when={(count > 0, |this| this.text_color(rgb(0x22c55e)))}
            when={(count > 10, |this| this.font_bold())}
        >
            {format!("Count: {count}")}
        </div>
    };
}
