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
use model::IncidentConsole;

fn main() {
    application()
        .with_assets(assets::Assets)
        .run(|cx: &mut App| {
            gpui::init(cx);
            let bounds = Bounds::centered(None, size(px(1320.0), px(820.0)), cx);
            gpui::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..WindowOptions::default()
                },
                cx,
                |_, cx| cx.new(|_| IncidentConsole::default()),
            )
            .expect("failed to open incident console window");
            cx.activate(true);
        });
}
