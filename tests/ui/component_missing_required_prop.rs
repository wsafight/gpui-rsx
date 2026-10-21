use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx};

#[component]
#[derive(IntoElement)]
struct Card {
    title: SharedString,
    #[prop(default)]
    elevated: bool,
}

impl RenderOnce for Card {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let _ = self.elevated;
        div().child(self.title)
    }
}

fn require_element(_: impl IntoElement) {}

fn main() {
    require_element(rsx! { <Card elevated /> });
    let _card = Card().elevated().build();
}
