use gpui::prelude::*;
use gpui::*;
use gpui_platform::application;
use gpui_rsx::rsx;

struct HelloView;

impl Render for HelloView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="size-full flex flex-col gap-4 p-6 bg-slate-950">
                <section class="flex flex-col gap-2 p-6 rounded-lg bg-slate-900 border border-slate-700">
                    <h1 class="text-3xl font-bold text-white">
                        {"GPUI-RSX"}
                    </h1>
                    <p class="text-base text-slate-300">
                        {"A small GPUI window rendered with JSX-like Rust syntax."}
                    </p>
                </section>

                <section class="flex gap-3">
                    <span class="px-4 py-2 rounded-md bg-blue-500 text-white">
                        {"Static classes"}
                    </span>
                    <span class="px-4 py-2 rounded-md bg-emerald-500 text-white">
                        {"GPUI builders"}
                    </span>
                    <span class="px-4 py-2 rounded-md bg-amber-400 text-slate-950">
                        {"Zero runtime parser"}
                    </span>
                </section>
            </div>
        }
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| HelloView))
            .expect("failed to open hello window");
        cx.activate(true);
    });
}
