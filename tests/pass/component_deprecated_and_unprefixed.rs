#![deny(deprecated)]

use gpui_rsx::component;

mod ui {
    use super::component;
    // Only the trait the hand-written bodies need; generated code must not rely on the prelude.
    use gpui::ParentElement as _;

    #[component]
    #[deprecated(note = "use NewCard")]
    pub fn OldCard(
        title: String,
        #[children] children: Vec<gpui::AnyElement>,
    ) -> impl gpui::IntoElement {
        gpui::div().child(title).children(children)
    }

    #[component]
    #[deprecated(note = "use NewBadge")]
    #[derive(gpui::IntoElement)]
    pub struct OldBadge {
        label: String,
        #[children]
        children: Vec<gpui::AnyElement>,
    }

    #[allow(deprecated)]
    impl gpui::RenderOnce for OldBadge {
        fn render(self, _: &mut gpui::Window, _: &mut gpui::App) -> impl gpui::IntoElement {
            gpui::div().child(self.label).children(self.children)
        }
    }

    #[component]
    pub fn r#Plain(#[children] children: Vec<gpui::AnyElement>) -> impl gpui::IntoElement {
        gpui::div().children(children)
    }
}

#[allow(deprecated)]
fn render() -> impl gpui::IntoElement {
    ui::OldCard()
        .title("Inbox")
        .child(ui::OldBadge().label("3").child(gpui::div()))
}

fn main() {
    let _ = render();
    let _ = ui::Plain().child(gpui::div());
}
