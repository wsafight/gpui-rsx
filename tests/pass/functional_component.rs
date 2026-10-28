use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx};

mod ui {
    use super::*;

    #[component]
    #[inline]
    pub fn Card(
        mut title: String,
        #[prop(default)] highlighted: bool,
        #[prop(default = 2u8)] depth: u8,
        #[children] children: Vec<AnyElement>,
        #[context] window: &mut Window,
        #[context] cx: &mut App,
    ) -> impl IntoElement {
        let _ = (window, cx, highlighted, depth);
        title.push('!');
        rsx! { <div>{title}{...children}</div> }
    }

    #[component]
    pub fn Label(r#type: String) -> impl IntoElement {
        div().child(r#type)
    }

    #[component]
    #[cfg(any())]
    pub fn Disabled(unknown: ThisTypeDoesNotExist) -> impl IntoElement {
        unknown
    }
}

fn main() {
    let active = true;
    let _ = rsx! {
        <ui::Card title="Inbox" highlightedIf={active}>
            <><ui::Label r#type="message" /><span>{"content"}</span></>
        </ui::Card>
    }.into_any_element();
}
