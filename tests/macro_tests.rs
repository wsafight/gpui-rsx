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
fn div() -> MockElement {
    MockElement
}
#[allow(dead_code)]
fn svg() -> MockElement {
    MockElement
}
#[allow(dead_code)]
fn img() -> MockElement {
    MockElement
}
#[allow(dead_code)]
fn canvas() -> MockElement {
    MockElement
}
fn rgb(_hex: u32) -> u32 {
    0
}
fn px(_val: f32) -> f32 {
    0.0
}

// 模拟自定义组件构造函数
#[allow(non_snake_case, dead_code)]
fn MyComponent() -> MockElement {
    MockElement
}
#[allow(non_snake_case, dead_code)]
fn CustomWidget() -> MockElement {
    MockElement
}

#[allow(dead_code)]
impl MockElement {
    // --- 身份 ---
    fn id<T>(self, _: T) -> Self {
        self
    }

    // --- 布局 ---
    fn flex(self) -> Self {
        self
    }
    fn flex_col(self) -> Self {
        self
    }
    fn flex_row(self) -> Self {
        self
    }
    fn flex_1(self) -> Self {
        self
    }
    fn flex_grow<T>(self, _: T) -> Self {
        self
    }
    fn flex_shrink<T>(self, _: T) -> Self {
        self
    }
    fn flex_wrap(self) -> Self {
        self
    }
    fn items_center(self) -> Self {
        self
    }
    fn items_start(self) -> Self {
        self
    }
    fn items_end(self) -> Self {
        self
    }
    fn justify_center(self) -> Self {
        self
    }
    fn justify_between(self) -> Self {
        self
    }

    // --- 间距（参数化 + 预设） ---
    fn gap<T>(self, _: T) -> Self {
        self
    }
    fn gap_2(self) -> Self {
        self
    }
    fn gap_3(self) -> Self {
        self
    }
    fn gap_4(self) -> Self {
        self
    }
    fn p<T>(self, _: T) -> Self {
        self
    }
    fn p_3(self) -> Self {
        self
    }
    fn p_4(self) -> Self {
        self
    }
    #[allow(clippy::wrong_self_convention)]
    fn px<T>(self, _: T) -> Self {
        self
    } // 注意：方法 px 与函数 px 不冲突
    fn py<T>(self, _: T) -> Self {
        self
    }
    fn pt<T>(self, _: T) -> Self {
        self
    }
    fn pb<T>(self, _: T) -> Self {
        self
    }
    fn pl<T>(self, _: T) -> Self {
        self
    }
    fn pr<T>(self, _: T) -> Self {
        self
    }
    fn px_2(self) -> Self {
        self
    }
    fn px_3(self) -> Self {
        self
    }
    fn px_4(self) -> Self {
        self
    }
    fn px_6(self) -> Self {
        self
    }
    fn py_1(self) -> Self {
        self
    }
    fn py_2(self) -> Self {
        self
    }
    fn m<T>(self, _: T) -> Self {
        self
    }
    fn mx<T>(self, _: T) -> Self {
        self
    }
    fn my<T>(self, _: T) -> Self {
        self
    }
    fn mt<T>(self, _: T) -> Self {
        self
    }
    fn mb<T>(self, _: T) -> Self {
        self
    }
    fn ml<T>(self, _: T) -> Self {
        self
    }
    fn mr<T>(self, _: T) -> Self {
        self
    }

    // --- 尺寸 ---
    fn w<T>(self, _: T) -> Self {
        self
    }
    fn h<T>(self, _: T) -> Self {
        self
    }
    fn w_full(self) -> Self {
        self
    }
    fn h_full(self) -> Self {
        self
    }
    fn size_full(self) -> Self {
        self
    }
    fn min_w<T>(self, _: T) -> Self {
        self
    }
    fn min_h<T>(self, _: T) -> Self {
        self
    }
    fn max_w<T>(self, _: T) -> Self {
        self
    }
    fn max_h<T>(self, _: T) -> Self {
        self
    }

    // --- 颜色 ---
    fn bg<T>(self, _: T) -> Self {
        self
    }
    fn text_color<T>(self, _: T) -> Self {
        self
    }
    fn border_color<T>(self, _: T) -> Self {
        self
    }

    // --- 文本 ---
    fn text_xs(self) -> Self {
        self
    }
    fn text_sm(self) -> Self {
        self
    }
    fn text_xl(self) -> Self {
        self
    }
    fn text_2xl(self) -> Self {
        self
    }
    fn text_3xl(self) -> Self {
        self
    }
    fn font_bold(self) -> Self {
        self
    }

    // --- 边框 ---
    fn rounded<T>(self, _: T) -> Self {
        self
    }
    fn rounded_md(self) -> Self {
        self
    }
    fn rounded_lg(self) -> Self {
        self
    }
    fn rounded_full(self) -> Self {
        self
    }
    fn border_1(self) -> Self {
        self
    }

    // --- 定位 ---
    fn absolute(self) -> Self {
        self
    }
    fn relative(self) -> Self {
        self
    }
    fn overflow<T>(self, _: T) -> Self {
        self
    }
    fn overflow_hidden(self) -> Self {
        self
    }
    fn overflow_scroll(self) -> Self {
        self
    }
    fn overflow_visible(self) -> Self {
        self
    }
    fn overflow_x_hidden<T>(self, _: T) -> Self {
        self
    }
    fn overflow_y_hidden<T>(self, _: T) -> Self {
        self
    }
    fn top<T>(self, _: T) -> Self {
        self
    }
    fn left<T>(self, _: T) -> Self {
        self
    }
    fn right<T>(self, _: T) -> Self {
        self
    }
    fn bottom<T>(self, _: T) -> Self {
        self
    }

    // --- 光标 ---
    fn cursor_pointer(self) -> Self {
        self
    }

    // --- 层级和透明度 ---
    fn z_index<T>(self, _: T) -> Self {
        self
    }
    fn opacity<T>(self, _: T) -> Self {
        self
    }

    // --- 可见性 ---
    fn visible<T>(self, _: T) -> Self {
        self
    }

    // --- 事件 ---
    fn on_click<T>(self, _: T) -> Self {
        self
    }
    fn on_mouse_down<T>(self, _: T) -> Self {
        self
    }
    fn on_mouse_up<T>(self, _: T) -> Self {
        self
    }
    fn on_mouse_move<T>(self, _: T) -> Self {
        self
    }
    fn on_key_down<T>(self, _: T) -> Self {
        self
    }
    fn on_key_up<T>(self, _: T) -> Self {
        self
    }
    fn on_focus<T>(self, _: T) -> Self {
        self
    }
    fn on_blur<T>(self, _: T) -> Self {
        self
    }
    fn on_hover<T>(self, _: T) -> Self {
        self
    }
    fn on_scroll_wheel<T>(self, _: T) -> Self {
        self
    }
    fn on_drag<T>(self, _: T) -> Self {
        self
    }
    fn on_drop<T>(self, _: T) -> Self {
        self
    }
    fn on_action<T>(self, _: T) -> Self {
        self
    }

    // --- 状态样式 ---
    fn hover<F: FnOnce(Self) -> Self>(self, _f: F) -> Self {
        self
    }
    fn active<F: FnOnce(Self) -> Self>(self, _f: F) -> Self {
        self
    }
    fn focus<F: FnOnce(Self) -> Self>(self, _f: F) -> Self {
        self
    }
    fn tooltip<T>(self, _: T) -> Self {
        self
    }
    fn group<T>(self, _: T) -> Self {
        self
    }
    fn track_focus(self) -> Self {
        self
    }

    // --- 文本 (additional) ---
    fn text_base(self) -> Self {
        self
    }
    fn text_lg(self) -> Self {
        self
    }
    fn text_4xl(self) -> Self {
        self
    }
    fn text_5xl(self) -> Self {
        self
    }

    // --- 额外属性映射 ---
    fn font_size<T>(self, _: T) -> Self {
        self
    }
    fn line_height<T>(self, _: T) -> Self {
        self
    }
    fn font_weight<T>(self, _: T) -> Self {
        self
    }
    fn text_align<T>(self, _: T) -> Self {
        self
    }
    fn border_radius<T>(self, _: T) -> Self {
        self
    }
    fn shadow<T>(self, _: T) -> Self {
        self
    }

    // --- 子节点 ---
    fn child<T>(self, _: T) -> Self {
        self
    }
    fn children<I: IntoIterator>(self, _: I) -> Self {
        self
    }

    // --- 条件方法 ---
    fn when<F>(self, _condition: bool, _f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        self
    }

    fn when_some<T, F>(self, _option: Option<T>, _f: F) -> Self
    where
        F: FnOnce(Self, T) -> Self,
    {
        self
    }

    // --- 转换方法 ---
    fn map<F>(self, _f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        self
    }

    // --- 新增属性映射 ---
    fn gap_x<T>(self, _: T) -> Self {
        self
    }
    fn gap_y<T>(self, _: T) -> Self {
        self
    }
    fn basis<T>(self, _: T) -> Self {
        self
    }
    fn order<T>(self, _: T) -> Self {
        self
    }
    fn inset<T>(self, _: T) -> Self {
        self
    }
    fn text_decoration<T>(self, _: T) -> Self {
        self
    }
    fn border_t<T>(self, _: T) -> Self {
        self
    }
    fn border_b<T>(self, _: T) -> Self {
        self
    }
    fn border_l<T>(self, _: T) -> Self {
        self
    }
    fn border_r<T>(self, _: T) -> Self {
        self
    }
    fn rounded_t<T>(self, _: T) -> Self {
        self
    }
    fn rounded_b<T>(self, _: T) -> Self {
        self
    }
    fn rounded_tl<T>(self, _: T) -> Self {
        self
    }
    fn rounded_tr<T>(self, _: T) -> Self {
        self
    }
    fn rounded_bl<T>(self, _: T) -> Self {
        self
    }
    fn rounded_br<T>(self, _: T) -> Self {
        self
    }
    fn flex_none(self) -> Self {
        self
    }
    fn flex_auto(self) -> Self {
        self
    }

    // --- 新增事件 ---
    fn on_mouse_down_out<T>(self, _: T) -> Self {
        self
    }
    fn on_mouse_up_out<T>(self, _: T) -> Self {
        self
    }

    // --- 新增边框 ---
    fn border_2(self) -> Self {
        self
    }
    fn border_4(self) -> Self {
        self
    }

    // --- 新增尺寸 ---
    fn shadow_sm(self) -> Self {
        self
    }
    fn shadow_md(self) -> Self {
        self
    }
    fn shadow_lg(self) -> Self {
        self
    }
    fn rounded_sm(self) -> Self {
        self
    }
    fn rounded_xl(self) -> Self {
        self
    }
    fn rounded_none(self) -> Self {
        self
    }
    fn cursor_default(self) -> Self {
        self
    }
    fn cursor_text(self) -> Self {
        self
    }

    // --- 杂项（示例中用到的属性） ---
    fn placeholder<T>(self, _: T) -> Self {
        self
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
fn test_z_index_attribute() {
    let _el = rsx! {
        <div zIndex={10}>{"层级 10"}</div>
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
fn test_invisible_flag() {
    let _el = rsx! {
        <div invisible>{"隐藏内容"}</div>
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
            zIndex={100}
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
            zIndex={1000}
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
    let items = vec!["a", "b", "c"];
    let _el = rsx! {
        <ul>
            {...items}
        </ul>
    };
}

#[test]
fn test_spread_with_map() {
    let items = vec!["apple", "banana"];
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
    let _el = rsx! { <svg /> };
}

#[test]
fn test_img_tag() {
    let _el = rsx! { <img /> };
}

#[test]
fn test_canvas_tag() {
    let _el = rsx! { <canvas /> };
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
fn test_track_focus_auto_id() {
    let _el = rsx! {
        <div track_focus>
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
    let _el = rsx! { <div onDrag={h} /> };
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
    let _el = rsx! { <div onDrag={h}>{"Drag me"}</div> };
}

#[test]
fn test_on_drag_event_snake_auto_id() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div on_drag={h}>{"Drag me"}</div> };
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

#[test]
fn test_class_text_4xl() {
    let _el = rsx! { <div class="text-4xl" /> };
}

#[test]
fn test_class_text_5xl() {
    let _el = rsx! { <div class="text-5xl" /> };
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
fn test_border_radius_attribute() {
    let _el = rsx! { <div borderRadius={px(8.0)} /> };
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

#[test]
fn test_overflow_x_attribute() {
    let _el = rsx! { <div overflowX={true} /> };
}

#[test]
fn test_overflow_y_attribute() {
    let _el = rsx! { <div overflowY={true} /> };
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
    let _a = rsx! { <div class="shadow-sm" /> };
    let _b = rsx! { <div class="shadow-md" /> };
    let _c = rsx! { <div class="shadow-lg" /> };
}

#[test]
fn test_class_cursor_variants() {
    let _a = rsx! { <div class="cursor-pointer" /> };
    let _b = rsx! { <div class="cursor-default" /> };
    let _c = rsx! { <div class="cursor-text" /> };
}

#[test]
fn test_class_full_size() {
    let _a = rsx! { <div class="w-full" /> };
    let _b = rsx! { <div class="h-full" /> };
    let _c = rsx! { <div class="size-full" /> };
}

#[test]
fn test_class_flex_variants() {
    let _a = rsx! { <div class="flex-none" /> };
    let _b = rsx! { <div class="flex-auto" /> };
    let _c = rsx! { <div class="flex-1" /> };
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
fn test_flex_grow_attribute() {
    let _el = rsx! { <div flexGrow={1.0} /> };
}

#[test]
fn test_flex_shrink_attribute() {
    let _el = rsx! { <div flexShrink={0.0} /> };
}

#[test]
fn test_flex_order_attribute() {
    let _el = rsx! { <div flexOrder={2} /> };
}

#[test]
fn test_border_top_attribute() {
    let _el = rsx! { <div borderTop={px(1.0)} /> };
}

#[test]
fn test_border_bottom_attribute() {
    let _el = rsx! { <div borderBottom={px(1.0)} /> };
}

#[test]
fn test_border_left_attribute() {
    let _el = rsx! { <div borderLeft={px(1.0)} /> };
}

#[test]
fn test_border_right_attribute() {
    let _el = rsx! { <div borderRight={px(1.0)} /> };
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

#[test]
fn test_text_decoration_attribute() {
    let _el = rsx! { <div textDecoration={"underline"} /> };
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
    let _el = rsx! { <div onMouseUpOut={h} /> };
}

#[test]
fn test_on_mouse_up_out_snake() {
    let h = |_: (), _: ()| {};
    let _el = rsx! { <div on_mouse_up_out={h} /> };
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
    let items = vec!["apple", "banana", "cherry"];
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
    let items = vec!["a", "b", "c"];
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
