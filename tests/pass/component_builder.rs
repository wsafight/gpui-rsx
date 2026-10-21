use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx};

#[component]
#[derive(IntoElement)]
pub struct Badge {
    /// Text shown in the badge.
    label: SharedString,
    tone: SharedString,
    #[prop(default = 1u8)]
    weight: u8,
    #[prop(default)]
    outlined: bool,
    #[children]
    children: Vec<AnyElement>,
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let _ = (self.tone, self.weight, self.outlined);
        rsx! { <span class="px-2 rounded-full">{self.label}{...self.children}</span> }
    }
}

mod ui {
    use gpui::prelude::*;
    use gpui::*;

    #[gpui_rsx::component]
    #[derive(IntoElement)]
    pub struct Empty {}

    impl RenderOnce for Empty {
        fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
            div()
        }
    }
}

fn main() {
    let active = true;
    let _el = rsx! {
        <div>
            <Badge label="new" tone={"info"} weight={3u8} outlinedIf={active}>
                <span>{"!"}</span>
            </Badge>
            <Badge tone="muted" label={String::from("draft")} outlined />
            <ui::Empty />
        </div>
    };
    // The builder is usable without RSX as well.
    let _badge: Badge = Badge().label("x").tone("y").build();
}
