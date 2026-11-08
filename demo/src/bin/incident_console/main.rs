#[path = "../common/mod.rs"]
mod common;
mod details;
mod domain;
mod model;
#[cfg(feature = "native-profiler")]
mod profile;
mod queue;
mod sample_data;
mod sidebar;
#[cfg(test)]
mod tests;
mod view;

use gpui::*;
use model::IncidentConsole;

fn main() {
    let rows = std::env::var("RSX_DEMO_ROWS").ok().map(|value| {
        value
            .parse::<usize>()
            .ok()
            .filter(|count| *count <= 100_000)
            .unwrap_or_else(|| {
                eprintln!("RSX_DEMO_ROWS must be an integer between 0 and 100000");
                std::process::exit(2);
            })
    });
    common::run(
        |cx| WindowOptions {
            window_min_size: Some(size(px(1120.0), px(640.0))),
            ..common::window(cx, "Incident Console · GPUI-RSX", 1320.0, 820.0)
        },
        move |_| {
            let mut view = IncidentConsole::default();
            if let Some(rows) = rows {
                view.load_rows(rows);
            }
            #[cfg(feature = "native-profiler")]
            {
                view.profile = profile::Profile::from_environment();
            }
            view
        },
    );
}
