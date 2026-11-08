mod common;

use gpui::component::ActiveTheme as _;
use gpui::component::Sizable as _;
use gpui::component::StyledExt as _;
use gpui::component::button::{Button, ButtonVariants as _};
use gpui::component::input::{Input, InputState};
use gpui::component::label::Label;
use gpui::component::popover::Popover;
use gpui::component::tab::Tab;
use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx};

#[component]
fn StatusBadge(
    label: SharedString,
    #[prop(default)] highlighted: bool,
    #[context] cx: &mut App,
) -> impl IntoElement {
    rsx! {
        <span
            class="px-[10px] py-[5px] rounded-md text-sm bg-muted text-muted-foreground"
            whenClass={(highlighted, "bg-primary text-primary-foreground")}
        >
            {label}
        </span>
    }
}

#[component]
fn StatusCard(
    title: SharedString,
    #[children] children: Vec<AnyElement>,
    #[context] cx: &mut App,
) -> impl IntoElement {
    rsx! {
        <section class="flex flex-col gap-[16px] p-[20px] rounded-lg border border-border" textColor={cx.theme().foreground}>
            <h2 class="text-lg font-semibold">{title}</h2>
            <div class="flex flex-wrap gap-[8px]">{...children}</div>
        </section>
    }
}

#[derive(Default)]
struct ComponentView {
    input: Option<Entity<InputState>>,
}

#[allow(dead_code)]
fn stateful_input_contract(window: &mut Window, cx: &mut App) -> impl IntoElement {
    let state = cx.new(|cx| InputState::new(window, cx));
    rsx! { <Input base={Input::new(&state)} /> }
}

#[allow(dead_code)]
fn facade_reexport_contract() -> impl IntoElement {
    let _edges: gpui::component::Edges<Pixels> = gpui::component::Edges::all(px(4.0));
    rsx! { <div base={div().h_flex()} /> }
}

impl Render for ComponentView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input = self
            .input
            .get_or_insert_with(|| cx.new(|cx| InputState::new(window, cx)));
        rsx! {
            <div class="size-full flex flex-col gap-[24px] p-[32px] bg-background text-foreground overflow-scroll">
                <header class="flex flex-col gap-[8px]">
                    <h1 class="text-2xl font-semibold">
                        {"gpui-component"}
                    </h1>
                    <p class="text-sm text-muted-foreground">
                        {"Native controls composed with RSX. Type in the input or open the popover."}
                    </p>
                </header>

                <StatusCard title="Function components">
                    <>
                        <StatusBadge label="Required props checked" highlighted />
                        <StatusBadge label="Children without wrappers" />
                    </>
                </StatusCard>

                <section class="flex flex-col gap-[20px] p-[20px] rounded-lg border border-border">
                    <Label
                        base={Label::new("Component label").secondary("rendered by gpui-component")}
                        class="text-sm"
                    />

                    <Input base={Input::new(&*input)} />

                    <div class="flex flex-wrap gap-[12px]">
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
    common::run(
        |cx| common::window(cx, "Components · GPUI-RSX", 800.0, 680.0),
        |_| ComponentView::default(),
    );
}
