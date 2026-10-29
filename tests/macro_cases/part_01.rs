use super::*;

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
    use gpui::StatefulInteractiveElement;

    // Use a rendering element here: MockElement's inherent setters model a builder.
    mod ui {
        #[allow(non_snake_case)]
        pub fn TaskCard() -> gpui::Div {
            gpui::div()
        }
    }

    let _el = rsx! {
        <ui::TaskCard onClick={|_, _, _| {}} />
    };

    let id = gpui::Element::id(&_el).expect("path component with stateful attr should get auto id");
    assert!(format!("{id:?}").contains("__rsx_ui::TaskCard_"));
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
