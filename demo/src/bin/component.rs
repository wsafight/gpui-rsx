use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::label::Label;
use gpui_component::Sizable as _;
use gpui_platform::application;
use gpui_rsx::rsx;

struct ComponentView;

impl Render for ComponentView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="size-full flex flex-col gap-4 p-6 bg-zinc-950">
                <header class="flex flex-col gap-2">
                    <h1 class="text-3xl font-bold text-white">
                        {"gpui-component"}
                    </h1>
                    <p class="text-sm text-zinc-400">
                        {"External component builders composed through RSX base constructors."}
                    </p>
                </header>

                <section class="flex flex-col gap-3 p-4 rounded-lg bg-zinc-900 border border-zinc-700">
                    <Label
                        base={Label::new("Component label").secondary("rendered by gpui-component")}
                        class="text-sm"
                    />

                    <div class="flex gap-3">
                        <Button
                            base={Button::new("component-primary")}
                            label={"Primary"}
                            primary
                            small
                        />
                        <Button
                            base={Button::new("component-success")}
                            label={"Success"}
                            success
                            small
                        />
                        <Button
                            base={Button::new("component-ghost")}
                            label={"Ghost"}
                            ghost
                            small
                        />
                    </div>
                </section>
            </div>
        }
    }
}

fn main() {
    application().run(|cx: &mut App| {
        gpui_component::init(cx);
        let _ = cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| ComponentView));
        cx.activate(true);
    });
}
