mod common;

use gpui::component::ActiveTheme as _;
use gpui::prelude::*;
use gpui::*;
use gpui_rsx::rsx;

struct HelloView;

impl Render for HelloView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="size-full flex flex-col justify-center gap-[24px] p-[32px] bg-background text-foreground">
                <section class="flex flex-col gap-[12px] max-w-[520px]">
                    <h1 class="text-3xl font-semibold">
                        {"Hello, GPUI."}
                    </h1>
                    <p class="text-base text-muted-foreground">
                        {"This window is rendered by an rsx! expression."}
                    </p>
                </section>

                <p class="text-sm text-muted-foreground pt-[20px] border-t border-border">
                    {"Static RSX compiles to GPUI builder calls. Edit hello.rs and run it again."}
                </p>
            </div>
        }
    }
}

fn main() {
    common::run(
        |cx| common::window(cx, "Hello · GPUI-RSX", 720.0, 480.0),
        |_| HelloView,
    );
}
