#[path = "../common/mod.rs"]
mod common;
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
    common::run(
        |cx| WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1320.0), px(820.0)),
                cx,
            ))),
            ..WindowOptions::default()
        },
        |_| IncidentConsole::default(),
    );
}
