//! Counter 示例 - 展示 GPUI-RSX 的基本用法

use gpui::*;
use gpui_rsx::rsx;

struct CounterView {
    count: i32,
}

impl CounterView {
    fn new() -> Self {
        Self { count: 0 }
    }
}

impl Render for CounterView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // ✅ 使用 RSX 宏
        rsx! {
            <div class="flex flex-col gap-4 p-4" bg={rgb(0xf3f4f6)}>
                <div class="text-2xl font-bold">
                    {format!("Count: {}", self.count)}
                </div>

                <div class="flex gap-2">
                    <button
                        bg={rgb(0x3b82f6)}
                        text_color={rgb(0xffffff)}
                        px_4
                        py_2
                        rounded_md
                        onClick={cx.listener(|view, _, cx| {
                            view.count += 1;
                            cx.notify();
                        })}
                    >
                        {"Increment"}
                    </button>

                    <button
                        bg={rgb(0xef4444)}
                        text_color={rgb(0xffffff)}
                        px_4
                        py_2
                        rounded_md
                        onClick={cx.listener(|view, _, cx| {
                            view.count -= 1;
                            cx.notify();
                        })}
                    >
                        {"Decrement"}
                    </button>

                    <button
                        bg={rgb(0x6b7280)}
                        text_color={rgb(0xffffff)}
                        px_4
                        py_2
                        rounded_md
                        onClick={cx.listener(|view, _, cx| {
                            view.count = 0;
                            cx.notify();
                        })}
                    >
                        {"Reset"}
                    </button>
                </div>

                {if self.count > 0 {
                    rsx! {
                        <div class="text-green-600">
                            {"Positive!"}
                        </div>
                    }
                } else if self.count < 0 {
                    rsx! {
                        <div class="text-red-600">
                            {"Negative!"}
                        </div>
                    }
                } else {
                    rsx! {
                        <div class="text-gray-600">
                            {"Zero"}
                        </div>
                    }
                }}
            </div>
        }
    }
}

// 对比：传统 GPUI 写法
impl CounterView {
    #[allow(dead_code)]
    fn render_traditional(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .bg(rgb(0xf3f4f6))
            .child(
                div()
                    .text_2xl()
                    .font_bold()
                    .child(format!("Count: {}", self.count)),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .bg(rgb(0x3b82f6))
                            .text_color(rgb(0xffffff))
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .on_click(cx.listener(|view, _, cx| {
                                view.count += 1;
                                cx.notify();
                            }))
                            .child("Increment"),
                    )
                    .child(
                        div()
                            .bg(rgb(0xef4444))
                            .text_color(rgb(0xffffff))
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .on_click(cx.listener(|view, _, cx| {
                                view.count -= 1;
                                cx.notify();
                            }))
                            .child("Decrement"),
                    )
                    .child(
                        div()
                            .bg(rgb(0x6b7280))
                            .text_color(rgb(0xffffff))
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .on_click(cx.listener(|view, _, cx| {
                                view.count = 0;
                                cx.notify();
                            }))
                            .child("Reset"),
                    ),
            )
            .child(if self.count > 0 {
                div().text_color(rgb(0x22c55e)).child("Positive!")
            } else if self.count < 0 {
                div().text_color(rgb(0xef4444)).child("Negative!")
            } else {
                div().text_color(rgb(0x6b7280)).child("Zero")
            })
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(|_cx| CounterView::new())
        });
    });
}
