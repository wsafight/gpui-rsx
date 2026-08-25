use gpui::prelude::*;
use gpui::*;
use gpui_platform::application;
use gpui_rsx::rsx;

#[derive(Default)]
struct CounterView {
    count: i32,
}

impl Render for CounterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_positive = self.count > 0;
        let is_negative = self.count < 0;

        rsx! {
            <div class="size-full flex flex-col gap-5 p-6 bg-neutral-950">
                <header class="flex flex-col gap-2">
                    <h1 class="text-3xl font-bold text-white">
                        {"Counter"}
                    </h1>
                    <p class="text-sm text-neutral-400">
                        {"This demo uses GPUI listeners generated from RSX attributes."}
                    </p>
                </header>

                <section class="flex flex-col gap-4 p-5 rounded-lg bg-neutral-900 border border-neutral-700">
                    <div
                        class={if is_positive { "text-emerald-400" } else if is_negative { "text-red-400" } else { "text-neutral-200" }}
                    >
                        {format!("Count: {}", self.count)}
                    </div>

                    <div class="flex gap-3">
                        <button
                            class="px-4 py-2 rounded-md bg-blue-500 text-white cursor-pointer"
                            onClick={cx.listener(|view, _, _window, cx| {
                                view.count += 1;
                                cx.notify();
                            })}
                        >
                            {"Increment"}
                        </button>

                        <button
                            class="px-4 py-2 rounded-md bg-red-500 text-white cursor-pointer"
                            onClick={cx.listener(|view, _, _window, cx| {
                                view.count -= 1;
                                cx.notify();
                            })}
                        >
                            {"Decrement"}
                        </button>

                        <button
                            class="px-4 py-2 rounded-md bg-neutral-700 text-white cursor-pointer"
                            onClick={cx.listener(|view, _, _window, cx| {
                                view.count = 0;
                                cx.notify();
                            })}
                        >
                            {"Reset"}
                        </button>
                    </div>
                </section>
            </div>
        }
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_| CounterView::default())
        })
        .expect("failed to open counter window");
        cx.activate(true);
    });
}
