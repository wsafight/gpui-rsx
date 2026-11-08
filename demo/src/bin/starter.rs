use gpui::component::{
    ActiveTheme as _,
    button::{Button, ButtonVariants as _},
};
use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{rsx, rsx_listener};

#[derive(Default)]
struct CounterView {
    count: usize,
}

impl Render for CounterView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="size-full flex flex-col gap-[16px] p-[24px]"
                bg={cx.theme().background} text_color={cx.theme().foreground}>
                <h1>{format!("Count: {}", self.count)}</h1>
                <Button base={Button::new("increment")} label="Increment" primary
                    onClick={rsx_listener!(cx, notify, |view, _, _, _| {
                        view.count += 1;
                    })} />
            </div>
        }
    }
}

fn main() {
    application()
        .with_assets(assets::Assets)
        .run(|cx: &mut App| {
            gpui::init(cx);
            gpui::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|_| CounterView::default())
            })
            .expect("failed to open counter");
            cx.activate(true);
        });
}
