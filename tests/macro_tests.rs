//! rsx! 宏展开测试
//!
//! 通过 mock GPUI 类型，实际展开 rsx! 宏验证生成代码的正确性。
//! 每个测试都会真实调用 rsx! 宏，确认展开后的方法链可以编译。

use gpui_rsx::rsx;

// ===========================================================================
// Mock GPUI 类型
// ===========================================================================

/// Mock Element，模拟 GPUI 的 Div / Stateful<Div>。
/// 所有 builder 方法返回 Self 以支持方法链。
#[allow(dead_code)]
#[derive(Debug)]
struct MockElement;

// 模拟 GPUI 构造函数
#[allow(dead_code)]
fn div() -> MockElement { MockElement }
fn rgb(_hex: u32) -> u32 { 0 }
fn px(_val: f32) -> f32 { 0.0 }

// 模拟自定义组件构造函数
#[allow(non_snake_case, dead_code)]
fn MyComponent() -> MockElement { MockElement }
#[allow(non_snake_case, dead_code)]
fn CustomWidget() -> MockElement { MockElement }

#[allow(dead_code)]
impl MockElement {
    // --- 身份 ---
    fn id<T>(self, _: T) -> Self { self }

    // --- 布局 ---
    fn flex(self) -> Self { self }
    fn flex_col(self) -> Self { self }
    fn flex_row(self) -> Self { self }
    fn flex_1(self) -> Self { self }
    fn flex_grow(self) -> Self { self }
    fn flex_shrink(self) -> Self { self }
    fn flex_wrap(self) -> Self { self }
    fn items_center(self) -> Self { self }
    fn items_start(self) -> Self { self }
    fn items_end(self) -> Self { self }
    fn justify_center(self) -> Self { self }
    fn justify_between(self) -> Self { self }

    // --- 间距（参数化 + 预设） ---
    fn gap<T>(self, _: T) -> Self { self }
    fn gap_2(self) -> Self { self }
    fn gap_3(self) -> Self { self }
    fn gap_4(self) -> Self { self }
    fn p<T>(self, _: T) -> Self { self }
    fn p_3(self) -> Self { self }
    fn p_4(self) -> Self { self }
    #[allow(clippy::wrong_self_convention)]
    fn px<T>(self, _: T) -> Self { self } // 注意：方法 px 与函数 px 不冲突
    fn py<T>(self, _: T) -> Self { self }
    fn pt<T>(self, _: T) -> Self { self }
    fn pb<T>(self, _: T) -> Self { self }
    fn pl<T>(self, _: T) -> Self { self }
    fn pr<T>(self, _: T) -> Self { self }
    fn px_2(self) -> Self { self }
    fn px_3(self) -> Self { self }
    fn px_4(self) -> Self { self }
    fn px_6(self) -> Self { self }
    fn py_1(self) -> Self { self }
    fn py_2(self) -> Self { self }
    fn m<T>(self, _: T) -> Self { self }
    fn mx<T>(self, _: T) -> Self { self }
    fn my<T>(self, _: T) -> Self { self }
    fn mt<T>(self, _: T) -> Self { self }
    fn mb<T>(self, _: T) -> Self { self }
    fn ml<T>(self, _: T) -> Self { self }
    fn mr<T>(self, _: T) -> Self { self }

    // --- 尺寸 ---
    fn w<T>(self, _: T) -> Self { self }
    fn h<T>(self, _: T) -> Self { self }
    fn w_full(self) -> Self { self }
    fn h_full(self) -> Self { self }
    fn size_full(self) -> Self { self }

    // --- 颜色 ---
    fn bg<T>(self, _: T) -> Self { self }
    fn text_color<T>(self, _: T) -> Self { self }
    fn border_color<T>(self, _: T) -> Self { self }

    // --- 文本 ---
    fn text_xs(self) -> Self { self }
    fn text_sm(self) -> Self { self }
    fn text_xl(self) -> Self { self }
    fn text_2xl(self) -> Self { self }
    fn text_3xl(self) -> Self { self }
    fn font_bold(self) -> Self { self }

    // --- 边框 ---
    fn rounded<T>(self, _: T) -> Self { self }
    fn rounded_md(self) -> Self { self }
    fn rounded_lg(self) -> Self { self }
    fn rounded_full(self) -> Self { self }
    fn border_1(self) -> Self { self }

    // --- 定位 ---
    fn absolute(self) -> Self { self }
    fn relative(self) -> Self { self }
    fn overflow_hidden(self) -> Self { self }

    // --- 光标 ---
    fn cursor_pointer(self) -> Self { self }

    // --- 事件 ---
    fn on_click<T>(self, _: T) -> Self { self }
    fn on_mouse_down<T>(self, _: T) -> Self { self }
    fn on_mouse_up<T>(self, _: T) -> Self { self }
    fn on_mouse_move<T>(self, _: T) -> Self { self }
    fn on_key_down<T>(self, _: T) -> Self { self }
    fn on_key_up<T>(self, _: T) -> Self { self }
    fn on_focus<T>(self, _: T) -> Self { self }
    fn on_blur<T>(self, _: T) -> Self { self }

    // --- 子节点 ---
    fn child<T>(self, _: T) -> Self { self }
    fn children<I: IntoIterator>(self, _: I) -> Self { self }

    // --- 杂项（示例中用到的属性） ---
    fn placeholder<T>(self, _: T) -> Self { self }
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
    let items = vec!["a", "b", "c"];
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
    let _el = rsx! { <div onMouseDown={h} /> };
}

#[test]
fn test_on_mouse_up_camel() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onMouseUp={h} /> };
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

#[test]
fn test_on_focus_camel() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onFocus={h} /> };
}

#[test]
fn test_on_blur_camel() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div onBlur={h} /> };
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
    let _el = rsx! { <div on_mouse_down={h} /> };
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
    let _el = rsx! { <div onMouseDown={h} /> };
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
    let items = vec!["apple", "banana", "cherry"];
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
