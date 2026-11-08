use gpui::component::ActiveTheme as _;
use gpui::{
    AnyElement, App, FontWeight, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
};
use gpui_rsx::{component, rsx};

#[component]
pub(super) fn Badge(
    label: SharedString,
    #[prop(default)] highlighted: bool,
    #[prop(default = 2u8)] depth: u8,
    #[context] window: &mut Window,
    #[context] cx: &mut App,
) -> impl IntoElement {
    let _ = (window.viewport_size(), depth);
    rsx! { <span class="p-4 rounded-md" whenClass={(highlighted, "bg-primary text-primary-foreground")}>{label}</span> }
}

#[component]
#[derive(gpui::IntoElement)]
pub(super) struct Panel {
    title: SharedString,
    #[prop(default)]
    compact: bool,
    #[children]
    children: Vec<AnyElement>,
}

impl RenderOnce for Panel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        rsx! {
            <section class="flex flex-col gap-8 border border-border rounded-lg p-12"
                whenClass={(self.compact, "gap-4 p-8")}>
                <h2 styled class="text-lg">{self.title}</h2>
                {...self.children}
            </section>
        }
    }
}
