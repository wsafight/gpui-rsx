use gpui::*;
use gpui_rsx::component;
use std::rc::Rc;

type OnPress = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[component]
fn Button(#[prop(callback)] on_press: OnPress) -> impl IntoElement {
    let _ = on_press;
    div()
}

fn main() {}
