use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx};

use gpui_rsx_demo::button::FunctionalButton;

struct AppView {
    focus_handle: FocusHandle,
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        rsx! {
            <div>
                <FunctionalButton label={"Inactive Button".to_string()} is_active={false} />
                <div ref={&self.focus_handle}>
                    {"Focus Target"}
                </div>
                <div on_click={move |_event, window, cx| focus_handle.focus(window, cx)}>
                    {"Click to Focus"}
                </div>
            </div>
        }
    }
}

use gpui_platform::application;

fn main() {
    application().run(|cx: &mut App| {
        let _ = cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|cx| AppView {
                focus_handle: cx.focus_handle(),
            })
        });
        cx.activate(true);
    });
}
