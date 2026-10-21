//! Compile-only contracts for semantic features against real GPUI, not the doctest stub.
use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx, rsx_strict, rsx_tw};

// Kit's macro facade and GPUI derives must resolve through the Cargo alias too.
gpui::actions!(rsx_contract, [Confirm]);

#[component]
#[derive(IntoElement)]
struct Card {
    title: SharedString,
    #[prop(default)]
    elevated: bool,
    #[children]
    children: Vec<AnyElement>,
}

impl RenderOnce for Card {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        rsx! {
            <div class="flex flex-col gap-2" whenClass={(self.elevated, "shadow-md")}>
                <h2>{self.title}</h2>
                {...self.children}
            </div>
        }
    }
}

fn component_contract() -> impl IntoElement {
    rsx! {
        <Card title="Inbox" elevated>
            <span>{"3 unread"}</span>
        </Card>
    }
}

fn state_contract() -> impl IntoElement {
    rsx_strict! {
        <div class="group/menu flex">
            <button class="hover:bg-blue-600 focus:border-2 focus-visible:underline active:opacity-75 group-hover/menu:italic group-active/menu:underline" />
        </div>
    }
}

fn scale_contract() -> impl IntoElement {
    rsx_tw! { <div class="gap-4 p-2.5 w-64 h-[14px]" /> }
}

fn dynamic_contract(classes: &str) -> impl IntoElement {
    rsx_strict! { <div class={classes} onUnknownClass={|_: &str| {}} /> }
}

fn theme_contract(cx: &App) -> impl IntoElement {
    use gpui::component::ActiveTheme as _;
    // `list` also names Theme's ListSettings. The preset must select colors.list.
    rsx_strict! { <div class="bg-list text-foreground border border-border" /> }
}

fn main() {
    let _ = (
        component_contract,
        state_contract,
        scale_contract,
        dynamic_contract,
        theme_contract,
    );
}
