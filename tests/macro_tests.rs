//! rsx! 宏展开测试
//!
//! 通过 mock GPUI 类型，实际展开 rsx! 宏验证生成代码的正确性。
//! 每个测试都会真实调用 rsx! 宏，确认展开后的方法链可以编译。

mod common;

use common::*;
use gpui_rsx::rsx;

mod ui {
    use super::MockElement;

    #[allow(non_snake_case)]
    pub fn TaskCard() -> MockElement {
        MockElement
    }

    pub struct TaskCardBuilder;

    impl TaskCardBuilder {
        pub fn new(_: u64) -> Self {
            Self
        }

        pub fn title(self, _: String) -> Self {
            self
        }

        pub fn compact(self) -> Self {
            self
        }
    }
}

// ===========================================================================
// 1. 基础元素
// ===========================================================================

#[test]
fn test_simple_div_with_text() {
    let _el = rsx! { <div>{"Hello"}</div> };
}

#[test]
fn test_self_closing_tag() {
    let _el = rsx! { <div /> };
}

#[test]
fn test_empty_pair_tag() {
    let _el = rsx! { <div></div> };
}

#[test]
fn test_self_closing_with_flag_attrs() {
    let _el = rsx! { <div flex flex_col /> };
}

#[test]
fn test_self_closing_with_value_attrs() {
    let _el = rsx! { <div bg={rgb(0xffffff)} text_color={rgb(0x000000)} /> };
}

#[test]
fn test_mixed_flag_and_value_attrs() {
    let _el = rsx! { <div flex flex_col bg={rgb(0xffffff)} gap={px(16.0)} /> };
}

// ===========================================================================
// 2. HTML 标签映射（全部映射到 div()）
// ===========================================================================

#[test]
fn test_container_tags() {
    let _a = rsx! { <span>{"span"}</span> };
    let _b = rsx! { <section>{"section"}</section> };
    let _c = rsx! { <article>{"article"}</article> };
    let _d = rsx! { <header>{"header"}</header> };
    let _e = rsx! { <footer>{"footer"}</footer> };
    let _f = rsx! { <main>{"main"}</main> };
    let _g = rsx! { <nav>{"nav"}</nav> };
    let _h = rsx! { <aside>{"aside"}</aside> };
}

#[test]
fn test_heading_tags() {
    let _a = rsx! { <h1>{"h1"}</h1> };
    let _b = rsx! { <h2>{"h2"}</h2> };
    let _c = rsx! { <h3>{"h3"}</h3> };
    let _d = rsx! { <h4>{"h4"}</h4> };
    let _e = rsx! { <h5>{"h5"}</h5> };
    let _f = rsx! { <h6>{"h6"}</h6> };
}

#[test]
fn test_text_tags() {
    let _a = rsx! { <p>{"paragraph"}</p> };
    let _b = rsx! { <label>{"label"}</label> };
    let _c = rsx! { <a>{"link"}</a> };
}

#[test]
fn test_form_tags() {
    let _a = rsx! { <button>{"btn"}</button> };
    let _b = rsx! { <input /> };
    let _c = rsx! { <textarea>{"text"}</textarea> };
    let _d = rsx! { <select>{"select"}</select> };
    let _e = rsx! { <form>{"form"}</form> };
}

#[test]
fn test_list_tags() {
    let _el = rsx! {
        <ul>
            <li>{"item 1"}</li>
            <li>{"item 2"}</li>
        </ul>
    };
}

// ===========================================================================
// 3. 自定义组件
// ===========================================================================

#[test]
fn test_custom_component_self_closing() {
    let _el = rsx! { <MyComponent /> };
}

#[test]
fn test_custom_component_with_children() {
    let _el = rsx! { <CustomWidget>{"content"}</CustomWidget> };
}

#[test]
fn test_custom_component_with_attrs() {
    let _el = rsx! { <MyComponent flex bg={rgb(0xff0000)} /> };
}

#[test]
fn test_custom_component_base_attribute_uses_custom_builder() {
    struct Button;

    impl Button {
        fn new(_: &str) -> Self {
            Self
        }

        fn label(self, _: &str) -> Self {
            self
        }

        fn small(self) -> Self {
            self
        }

        fn primary(self) -> Self {
            self
        }
    }

    let _el = rsx! {
        <Button
            base={Button::new("save")}
            label={"保存"}
            small
            primary
        />
    };
}

#[test]
fn test_path_component_self_closing() {
    let _el = rsx! { <ui::TaskCard flex /> };
}

#[test]
fn test_path_component_with_children() {
    let _el = rsx! {
        <ui::TaskCard>
            {"content"}
        </ui::TaskCard>
    };
}

#[test]
fn test_path_component_with_base_attribute() {
    let task_id = 42_u64;
    let title = String::from("Task");

    let _el = rsx! {
        <ui::TaskCard
            base={ui::TaskCardBuilder::new(task_id)}
            title={title.clone()}
            compact
        />
    };
}

#[test]
fn test_path_component_auto_id_with_stateful_attr() {
    take_last_auto_id();

    let _el = rsx! {
        <ui::TaskCard onClick={|_: &mut MockElement| {}} />
    };

    let id = take_last_auto_id().expect("path component with stateful attr should get auto id");
    assert!(id.contains("__rsx_ui::TaskCard_"));
}

// ===========================================================================
// 4. 嵌套
// ===========================================================================

#[test]
fn test_single_child_element() {
    let _el = rsx! {
        <div>
            <span>{"child"}</span>
        </div>
    };
}

#[test]
fn test_multiple_child_elements() {
    let _el = rsx! {
        <div>
            <span>{"child 1"}</span>
            <span>{"child 2"}</span>
            <span>{"child 3"}</span>
        </div>
    };
}

#[test]
fn test_deeply_nested_4_levels() {
    let _el = rsx! {
        <div>
            <section>
                <article>
                    <p>{"deep content"}</p>
                </article>
            </section>
        </div>
    };
}

#[test]
fn test_mixed_children_elements_and_exprs() {
    let _el = rsx! {
        <div>
            <span>{"text child"}</span>
            {format!("expr child {}", 42)}
            <p>{"another element"}</p>
        </div>
    };
}

// ===========================================================================
// 5. 表达式子节点
// ===========================================================================

#[test]
fn test_string_literal_child() {
    let _el = rsx! { <div>{"Hello world"}</div> };
}

#[test]
fn test_format_macro_child() {
    let count = 42;
    let _el = rsx! { <div>{format!("Count: {count}")}</div> };
}

#[test]
fn test_variable_child() {
    let message = String::from("hello");
    let _el = rsx! { <div>{message}</div> };
}

#[test]
fn test_method_call_child() {
    let items = ["a", "b", "c"];
    let _el = rsx! { <div>{items.len()}</div> };
}

#[test]
fn test_arithmetic_expr_child() {
    let x = 10;
    let _el = rsx! { <div>{x + 1}</div> };
}

#[test]
fn test_multiple_expr_children() {
    let _el = rsx! {
        <div>
            {"first"}
            {"second"}
            {"third"}
        </div>
    };
}

#[test]
fn test_multiple_expr_children_different_types() {
    let _el = rsx! {
        <div>
            {String::from("hello")}
            {"world"}
            {42_i32}
        </div>
    };
}

// ===========================================================================
// 6. 条件渲染
// ===========================================================================

#[test]
fn test_if_else() {
    let show = true;
    let _el = rsx! {
        <div>
            {if show {
                rsx! { <span>{"Visible"}</span> }
            } else {
                rsx! { <span>{"Hidden"}</span> }
            }}
        </div>
    };
}

#[test]
fn test_if_else_if_else() {
    let value = 1;
    let _el = rsx! {
        <div>
            {if value > 0 {
                rsx! { <span>{"positive"}</span> }
            } else if value < 0 {
                rsx! { <span>{"negative"}</span> }
            } else {
                rsx! { <span>{"zero"}</span> }
            }}
        </div>
    };
}

#[test]
fn test_conditional_attr_value() {
    let active = true;
    let _el = rsx! {
        <div bg={if active { rgb(0x3b82f6) } else { rgb(0xe5e7eb) }} />
    };
}

// ===========================================================================
// 7. class 属性 — 基本方法
// ===========================================================================

#[test]
fn test_class_flex_layout() {
    let _el = rsx! { <div class="flex flex-col" /> };
}

#[test]
fn test_class_flex_1() {
    let _el = rsx! { <div class="flex-1" /> };
}

#[test]
fn test_class_alignment() {
    let _el = rsx! { <div class="items-center justify-between" /> };
}

#[test]
fn test_class_font() {
    let _el = rsx! { <div class="font-bold" /> };
}

// ===========================================================================
// 8. class 属性 — 间距/尺寸数值
// ===========================================================================

#[test]
fn test_class_gap_numeric() {
    // gap-4 → .gap(px(4.0))
    let _el = rsx! { <div class="gap-4" /> };
}

#[test]
fn test_class_padding_numeric() {
    // p-4 → .p(px(4.0))
    let _el = rsx! { <div class="p-4" /> };
}

#[test]
fn test_class_padding_directional() {
    // px-2 → .px(px(2.0)), py-1 → .py(px(1.0))
    let _el = rsx! { <div class="px-2 py-1" /> };
}

#[test]
fn test_class_margin_numeric() {
    // m-2 mx-4 → .m(px(2.0)).mx(px(4.0))
    let _el = rsx! { <div class="m-2 mx-4" /> };
}

#[test]
fn test_class_sizing_numeric() {
    // w-8 h-8 → .w(px(8.0)).h(px(8.0))
    let _el = rsx! { <div class="w-8 h-8" /> };
}

#[test]
fn test_class_all_padding_directions() {
    let _el = rsx! { <div class="pt-1 pb-2 pl-3 pr-4" /> };
}

#[test]
fn test_class_all_margin_directions() {
    let _el = rsx! { <div class="mt-1 mb-2 ml-3 mr-4" /> };
}

// ===========================================================================
// 9. class 属性 — 文本大小
// ===========================================================================

#[test]
fn test_class_text_sizes() {
    let _a = rsx! { <div class="text-sm" /> };
    let _b = rsx! { <div class="text-xl" /> };
    let _c = rsx! { <div class="text-2xl" /> };
    let _d = rsx! { <div class="text-3xl" /> };
}

// ===========================================================================
// 10. class 属性 — 颜色
// ===========================================================================

#[test]
fn test_class_text_colors() {
    let _a = rsx! { <div class="text-red-600" /> };
    let _b = rsx! { <div class="text-green-600" /> };
    let _c = rsx! { <div class="text-blue-600" /> };
    let _d = rsx! { <div class="text-gray-600" /> };
}

#[test]
fn test_class_bg_colors() {
    let _a = rsx! { <div class="bg-blue-500" /> };
    let _b = rsx! { <div class="bg-red-500" /> };
    let _c = rsx! { <div class="bg-red-600" /> };
}

// ===========================================================================
// 11. class 属性 — 综合
// ===========================================================================

#[test]
fn test_class_combined_many() {
    let _el = rsx! {
        <div class="flex flex-col gap-4 p-4 text-2xl font-bold items-center" />
    };
}

#[test]
fn test_class_with_other_attrs() {
    let _el = rsx! {
        <div class="flex flex-col gap-4" bg={rgb(0xffffff)} cursor_pointer />
    };
}

// ===========================================================================
// 12. 事件处理器 — camelCase
// ===========================================================================

#[test]
fn test_on_click_camel() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <button onClick={h}>{"Click"}</button> };
}

#[test]
fn test_on_mouse_down_camel() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onMouseDown={(MouseButton::Left, h)} /> };
}

#[test]
fn test_on_mouse_up_camel() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onMouseUp={(MouseButton::Left, h)} /> };
}

#[test]
fn test_on_mouse_move_camel() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onMouseMove={h} /> };
}

#[test]
fn test_on_key_down_camel() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onKeyDown={h} /> };
}

#[test]
fn test_on_key_up_camel() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onKeyUp={h} /> };
}

// ===========================================================================
// 13. 事件处理器 — snake_case
// ===========================================================================

#[test]
fn test_on_click_snake() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <button on_click={h}>{"Click"}</button> };
}

#[test]
fn test_on_mouse_down_snake() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div on_mouse_down={(MouseButton::Right, h)} /> };
}

#[test]
fn test_on_key_down_snake() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div on_key_down={h} /> };
}

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

#[test]
fn test_class_color_red_500() {
    let _el = rsx! { <div class="bg-red-500" /> };
}

#[test]
fn test_class_color_green_500() {
    let _el = rsx! { <div class="bg-green-500" /> };
}

#[test]
fn test_class_color_gray_500() {
    let _el = rsx! { <div class="bg-gray-500" /> };
}

#[test]
fn test_class_color_yellow_500() {
    let _el = rsx! { <div class="bg-yellow-500" /> };
}

#[test]
fn test_class_color_purple_500() {
    let _el = rsx! { <div class="bg-purple-500" /> };
}

#[test]
fn test_class_color_pink_500() {
    let _el = rsx! { <div class="bg-pink-500" /> };
}

#[test]
fn test_class_color_indigo_500() {
    let _el = rsx! { <div class="bg-indigo-500" /> };
}

#[test]
fn test_class_color_white() {
    let _el = rsx! { <div class="bg-white" /> };
}

#[test]
fn test_class_color_black() {
    let _el = rsx! { <div class="text-black" /> };
}

// ===========================================================================
// 28. 文本大小白名单测试
// ===========================================================================

#[test]
fn test_class_text_xs() {
    let _el = rsx! { <div class="text-xs" /> };
}

#[test]
fn test_class_text_base() {
    let _el = rsx! { <div class="text-base" /> };
}

#[test]
fn test_class_text_lg() {
    let _el = rsx! { <div class="text-lg" /> };
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

#[test]
fn test_styled_button() {
    // button + styled → cursor_pointer
    let _el = rsx! { <button styled>{"Click"}</button> };
}

#[test]
fn test_styled_h1() {
    let _el = rsx! { <h1 styled>{"Title"}</h1> };
}

#[test]
fn test_styled_h2() {
    let _el = rsx! { <h2 styled>{"Title"}</h2> };
}

#[test]
fn test_styled_h3() {
    let _el = rsx! { <h3 styled>{"Title"}</h3> };
}

#[test]
fn test_styled_h4() {
    let _el = rsx! { <h4 styled>{"Title"}</h4> };
}

#[test]
fn test_styled_h5() {
    let _el = rsx! { <h5 styled>{"Title"}</h5> };
}

#[test]
fn test_styled_h6() {
    let _el = rsx! { <h6 styled>{"Title"}</h6> };
}

#[test]
fn test_styled_a() {
    let _el = rsx! { <a styled>{"Link"}</a> };
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
fn test_class_font_weight_mapping_gpui_0_2() {
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
    let _f = rsx! { <div class="flex-shrink" /> };
}

#[test]
fn test_class_gpui_0_2_2_helpers() {
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

#[test]
fn test_border_top_attribute() {
    let _el = rsx! { <div border_t /> };
}

#[test]
fn test_border_bottom_attribute() {
    let _el = rsx! { <div border_b /> };
}

#[test]
fn test_border_left_attribute() {
    let _el = rsx! { <div border_l /> };
}

#[test]
fn test_border_right_attribute() {
    let _el = rsx! { <div border_r /> };
}

#[test]
fn test_border_top_value_attribute() {
    let _el = rsx! { <div border_t={px(1.0)} /> };
}

#[test]
fn test_border_bottom_value_attribute() {
    let _el = rsx! { <div border_b={px(1.0)} /> };
}

#[test]
fn test_border_left_value_attribute() {
    let _el = rsx! { <div border_l={px(1.0)} /> };
}

#[test]
fn test_border_right_value_attribute() {
    let _el = rsx! { <div border_r={px(1.0)} /> };
}

#[test]
fn test_class_border_t() {
    let _el = rsx! { <div class="border-t" /> };
}

#[test]
fn test_class_border_b() {
    let _el = rsx! { <div class="border-b" /> };
}

#[test]
fn test_class_border_l() {
    let _el = rsx! { <div class="border-l" /> };
}

#[test]
fn test_class_border_r() {
    let _el = rsx! { <div class="border-r" /> };
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
        vec![("px", 7.0), ("px", 9.0), ("px", 16.0)]
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
            ("px", 5.0),
            ("px", 5.0),
            ("px", 5.0),
            ("px", 3.0),
            ("px", 3.0)
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
        vec![("px", 3.0), ("px", 3.0), ("px", 3.0)]
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
        vec![("px", 48.0), ("px", 16.0), ("px", 8.0)]
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

    assert_eq!(take_length_calls(), vec![("px", 4.0), ("px", 6.0)]);
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
    assert_eq!(take_length_calls(), vec![("px", 4.0)]);
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
        "dynamic class `border-t` should map to .border_t_1(), got {:?}",
        calls
    );
    assert!(
        calls.contains(&"border_b_1"),
        "dynamic class `border-b` should map to .border_b_1(), got {:?}",
        calls
    );
    assert!(
        calls.contains(&"border_l_1"),
        "dynamic class `border-l` should map to .border_l_1(), got {:?}",
        calls
    );
    assert!(
        calls.contains(&"border_r_1"),
        "dynamic class `border-r` should map to .border_r_1(), got {:?}",
        calls
    );
    assert!(
        calls.contains(&"border_x_1"),
        "dynamic class `border-x` should map to .border_x_1(), got {:?}",
        calls
    );
    assert!(
        calls.contains(&"border_y_1"),
        "dynamic class `border-y` should map to .border_y_1(), got {:?}",
        calls
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
    assert_eq!(take_length_calls(), vec![("px", 7.0)]);
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
    assert!(compact.contains(".flex().gap(px(7"));
    assert!(compact.contains(".block()"));
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
