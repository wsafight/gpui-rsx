use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx};

#[component]
fn Greeting(label: SharedString) -> impl IntoElement {
    div().child(label)
}

fn require_element(_: impl IntoElement) {}

fn main() {
    require_element(rsx! { <Greeting /> });
    let _ = Greeting().build();
}
