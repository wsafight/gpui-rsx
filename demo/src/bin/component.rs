use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::input::{Input, InputState};
use gpui_component::label::Label;
use gpui_component::popover::Popover;
use gpui_component::tab::Tab;
use gpui_component::Sizable as _;
use gpui_component::StyledExt as _;
use gpui_platform::application;
use gpui_rsx::rsx;

struct ComponentView;

#[allow(dead_code)]
fn stateful_input_contract(window: &mut Window, cx: &mut App) -> impl IntoElement {
    let state = cx.new(|cx| InputState::new(window, cx));
    rsx! { <Input base={Input::new(&state)} /> }
}

#[allow(dead_code)]
fn facade_reexport_contract() -> impl IntoElement {
    let _edges: gpui_component::Edges<Pixels> = gpui_component::Edges::all(px(4.0));
    rsx! { <div base={div().h_flex()} /> }
}

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

                    <Tab
                        base={Tab::new()
                            .label("Overview")
                            .aria_label("Overview tab")}
                        underline
                    />

                    <Popover
                        base={Popover::new("component-popover")
                            .trigger(Button::new("component-popover-trigger").label("Open"))
                            .content(|_, _, _| div().child("Popover content"))}
                    />
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
