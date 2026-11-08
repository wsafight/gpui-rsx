mod common;

use gpui::component::{
    ActiveTheme as _, Disableable as _,
    button::{Button, ButtonVariants as _},
};
use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{rsx, rsx_listener, rsx_update};

#[derive(Default)]
struct CounterView {
    count: i32,
}

impl Render for CounterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if std::env::var_os("GPUI_RSX_DEV_READY").is_some() {
            static READY: std::sync::Once = std::sync::Once::new();
            READY.call_once(|| println!("GPUI_RSX_READY"));
        }
        let is_positive = self.count > 0;
        let is_negative = self.count < 0;

        rsx! {
            <div class="size-full overflow-y-scroll flex flex-col gap-[28px] p-[32px] bg-background text-foreground">
                <header class="flex flex-col gap-[8px]">
                    <h1 class="text-2xl font-semibold">
                        {"Counter"}
                    </h1>
                    <p class="text-sm text-muted-foreground">
                        {"Change the value with the buttons. Reset returns it to zero."}
                    </p>
                </header>

                <section class="flex flex-col gap-[24px] p-[24px] rounded-lg bg-muted border border-border">
                    <div
                        class="text-[64px] font-semibold"
                        whenClass={(is_positive, "text-emerald-500")}
                        whenClass={(is_negative, "text-rose-500")}
                    >
                        {self.count.to_string()}
                    </div>

                    <div class="flex flex-wrap gap-[12px]">
                        <Button
                            base={Button::new("counter-increment")}
                            label="Increase"
                            primary
                            onClick={rsx_listener!(cx, notify, |view, _, _window, _cx| {
                                view.count += 1;
                            })}
                        />

                        <Button
                            base={Button::new("counter-decrement")}
                            label="Decrease"
                            onClick={rsx_listener!(cx, notify, |view, _, _window, _cx| {
                                view.count -= 1;
                            })}
                        />

                        <Button
                            base={Button::new("counter-reset")}
                            label="Reset"
                            disabled={self.count == 0}
                            onClick={rsx_listener!(cx, |view, _, _window, cx| {
                                rsx_update!(view, cx, |view, _| view.count = 0);
                            })}
                        />
                    </div>
                </section>
            </div>
        }
    }
}

fn main() {
    common::run(
        |cx| common::window(cx, "Counter · GPUI-RSX", 720.0, 520.0),
        |_| CounterView::default(),
    );
}
