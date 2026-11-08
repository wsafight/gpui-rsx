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
            window_min_size: Some(size(px(1120.0), px(640.0))),
            ..common::window(cx, "Incident Console · GPUI-RSX", 1320.0, 820.0)
        },
        |_| IncidentConsole::default(),
    );
}
