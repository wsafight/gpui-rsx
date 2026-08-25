mod details;
mod domain;
mod model;
mod queue;
mod sample_data;
mod sidebar;
#[cfg(test)]
mod tests;
mod view;

use gpui::*;
use gpui_platform::application;
use model::IncidentConsole;

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1320.0), px(820.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..WindowOptions::default()
            },
            |_, cx| cx.new(|_| IncidentConsole::default()),
        )
        .expect("failed to open incident console window");
        cx.activate(true);
    });
}
