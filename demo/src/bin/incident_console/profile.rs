use crate::model::IncidentConsole;
use gpui::{Context, ScrollStrategy, Window, profiler::FrameDurationSnapshot};
use serde_json::json;
use std::{path::PathBuf, time::Instant};

#[cfg(test)]
#[path = "tests/profile.rs"]
mod tests;

pub(crate) struct Profile {
    output: PathBuf,
    frames: usize,
    pending: bool,
    baseline: Option<FrameDurationSnapshot>,
    rows: Vec<usize>,
    intervals: Vec<f64>,
    previous_frame: Instant,
    started: Instant,
}

impl Profile {
    pub(crate) fn from_environment() -> Option<Self> {
        std::env::var_os("RSX_NATIVE_REPORT").map(|output| Self {
            output: output.into(),
            frames: 0,
            pending: false,
            baseline: None,
            rows: Vec::new(),
            intervals: Vec::new(),
            previous_frame: Instant::now(),
            started: Instant::now(),
        })
    }

    fn finish(&self, window: &Window, records: usize) -> Result<(), String> {
        let mut metrics = window.frame_duration_snapshot();
        let baseline = self.baseline.as_ref().ok_or("Missing warmup baseline")?;
        for (histogram, initial) in [
            (
                &mut metrics.draw_duration_histogram,
                &baseline.draw_duration_histogram,
            ),
            (
                &mut metrics.present_interval_histogram,
                &baseline.present_interval_histogram,
            ),
            (
                &mut metrics.dirty_to_present_histogram,
                &baseline.dirty_to_present_histogram,
            ),
        ] {
            histogram
                .subtract(initial)
                .map_err(|error| error.to_string())?;
        }
        let draw = &metrics.draw_duration_histogram;
        let present = &metrics.present_interval_histogram;
        let dirty = &metrics.dirty_to_present_histogram;
        let mut intervals = self.intervals.clone();
        intervals.sort_by(f64::total_cmp);
        let interval = |quantile: f64| {
            intervals
                .get(((intervals.len() as f64 * quantile).ceil() as usize).saturating_sub(1))
                .copied()
        };
        let report = json!({
            "schema": 1,
            "scenario": "scroll-and-select",
            "records": records,
            "profile": if cfg!(debug_assertions) { "debug" } else { "release" },
            "warmup_frames": 10,
            "sample_frames": self.rows.len(),
            "max_rendered_rows": self.rows.iter().max().copied().unwrap_or(0),
            "elapsed_seconds": self.started.elapsed().as_secs_f64(),
            "frame_callback_interval_ms": {"count": intervals.len(), "p50": interval(0.5), "p95": interval(0.95)},
            "draw_ms": {"count": draw.len(), "p50": draw.value_at_quantile(0.5) as f64 / 1e6, "p95": draw.value_at_quantile(0.95) as f64 / 1e6, "p99": draw.value_at_quantile(0.99) as f64 / 1e6},
            "present_interval_ms": {"count": present.len(), "p50": present.value_at_quantile(0.5) as f64 / 1e6, "p95": present.value_at_quantile(0.95) as f64 / 1e6},
            "dirty_to_present_ms": {"count": dirty.len(), "p50": dirty.value_at_quantile(0.5) as f64 / 1e6, "p95": dirty.value_at_quantile(0.95) as f64 / 1e6},
            "limitations": "CPU-side GPUI measurements; not GPU completion, display scan-out or physical input latency. No allocation or memory measurement."
        });
        std::fs::write(
            &self.output,
            serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        println!("GPUI_RSX_NATIVE {}", report);
        Ok(())
    }
}

pub(crate) fn schedule(
    view: &mut IncidentConsole,
    window: &Window,
    cx: &mut Context<IncidentConsole>,
) {
    let Some(profile) = view.profile.as_mut() else {
        return;
    };
    if profile.pending {
        return;
    }
    profile.pending = true;
    let entity = cx.entity().downgrade();
    window.on_next_frame(move |window, cx| {
        let _ = entity.update(cx, |view, cx| {
            let records = view.incidents().len();
            let profile = view.profile.as_mut().unwrap();
            let now = Instant::now();
            let interval = now.duration_since(profile.previous_frame).as_secs_f64() * 1000.0;
            profile.previous_frame = now;
            profile.pending = false;
            profile.frames += 1;
            if profile.frames == 10 {
                profile.baseline = Some(window.frame_duration_snapshot());
            } else if profile.frames > 10 {
                profile.rows.push(view.rendered_rows.get());
                profile.intervals.push(interval);
            }
            if profile.frames >= 70 {
                if let Err(error) = profile.finish(window, records) {
                    eprintln!("Native probe failed: {error}");
                }
                cx.quit();
                return;
            }
            let frame = profile.frames;
            let snapshot = view.snapshot();
            if !snapshot.visible_ids.is_empty() {
                let index = (frame * 997) % snapshot.visible_ids.len();
                view.selected_id = snapshot.visible_ids[index];
                view.queue_scroll.scroll_to_item(index, ScrollStrategy::Top);
            }
            cx.notify();
        });
    });
}
