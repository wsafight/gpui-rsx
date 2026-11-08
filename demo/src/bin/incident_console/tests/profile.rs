use super::*;
use gpui::test::TestWindowExt;
use gpui::{
    App, AppContext, Bounds, Entity, TestAppContext, WindowBounds, WindowOptions, px, size,
};
use serde_json::Value;
use std::{
    fs,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

struct ReportDirectory(PathBuf);

impl ReportDirectory {
    fn new() -> Self {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "rsx-profiler-{}-{unique}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn output(&self) -> PathBuf {
        self.0.join("report.json")
    }
}

impl Drop for ReportDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn profile(output: PathBuf) -> Profile {
    Profile {
        output,
        frames: 0,
        pending: false,
        baseline: None,
        rows: Vec::new(),
        intervals: Vec::new(),
        previous_frame: Instant::now(),
        started: Instant::now(),
    }
}

fn with_console(
    cx: &mut TestAppContext,
    run: impl FnOnce(Entity<IncidentConsole>, &mut Window, &mut App),
) {
    cx.update(gpui::init);
    let (handle, view) = cx.update(|cx| {
        gpui::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                    Default::default(),
                    size(px(1320.0), px(820.0)),
                ))),
                ..Default::default()
            },
            cx,
            |_, cx| cx.new(|_| IncidentConsole::default()),
        )
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| run(view, window, cx))
        .unwrap();
}

#[gpui::test]
fn profiler_warms_up_once_and_samples_exactly_sixty_frames(cx: &mut TestAppContext) {
    let directory = ReportDirectory::new();
    with_console(cx, |view, window, cx| {
        view.update(cx, |view, cx| {
            view.load_rows(1_000);
            view.profile = Some(profile(directory.output()));
            cx.notify();
        });
        for frame in 1..=70 {
            window.render_frame(cx);
            view.update(cx, |view, cx| {
                schedule(view, window, cx);
                schedule(view, window, cx);
            });
            assert!(view.read(cx).profile.as_ref().unwrap().pending);
            assert!(window.simulate_next_frame(cx) > 0);
            let state = view.read(cx).profile.as_ref().unwrap();
            assert_eq!(state.frames, frame);
            assert!(!state.pending);
            assert_eq!(state.baseline.is_some(), frame >= 10);
            assert_eq!(state.rows.len(), frame.saturating_sub(10));
            assert_eq!(state.intervals.len(), state.rows.len());
            assert_eq!(directory.output().exists(), frame == 70);
        }
        let report: Value = serde_json::from_slice(&fs::read(directory.output()).unwrap()).unwrap();
        assert_eq!(report["records"], 1_000);
        assert_eq!(report["warmup_frames"], 10);
        assert_eq!(report["sample_frames"], 60);
        assert_eq!(report["frame_callback_interval_ms"]["count"], 60);
        let rows = report["max_rendered_rows"].as_u64().unwrap();
        assert!(rows > 0 && rows < 100);
    });
}

#[gpui::test]
fn profiler_reports_quantiles_and_subtracts_the_warmup_baseline(cx: &mut TestAppContext) {
    let directory = ReportDirectory::new();
    with_console(cx, |_, window, _| {
        let mut state = profile(directory.output());
        state.baseline = Some(window.frame_duration_snapshot());
        state.rows = (0..60).map(|index| index % 9 + 1).collect();
        state.intervals = (1..=60).rev().map(f64::from).collect();
        state.finish(window, 100_000).unwrap();
        let report: Value = serde_json::from_slice(&fs::read(directory.output()).unwrap()).unwrap();
        assert_eq!(report["schema"], 1);
        assert_eq!(report["scenario"], "scroll-and-select");
        assert_eq!(
            report["profile"],
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
        );
        assert_eq!(report["sample_frames"], 60);
        assert_eq!(report["max_rendered_rows"], 9);
        assert_eq!(
            report["frame_callback_interval_ms"],
            json!({"count": 60, "p50": 30.0, "p95": 57.0})
        );
        for metric in ["draw_ms", "present_interval_ms", "dirty_to_present_ms"] {
            assert_eq!(report[metric]["count"], 0);
            assert_eq!(report[metric]["p50"], 0.0);
            assert_eq!(report[metric]["p95"], 0.0);
        }
        assert!(report["elapsed_seconds"].as_f64().unwrap() >= 0.0);
        assert!(
            report["limitations"]
                .as_str()
                .unwrap()
                .contains("not GPU completion")
        );
    });
}

#[gpui::test]
fn profiler_rejects_missing_baselines_underflow_and_unwritable_reports(cx: &mut TestAppContext) {
    let directory = ReportDirectory::new();
    with_console(cx, |_, window, _| {
        let mut state = profile(directory.output());
        assert_eq!(
            state.finish(window, 1_000).unwrap_err(),
            "Missing warmup baseline"
        );
        assert!(!directory.output().exists());
        let mut baseline = window.frame_duration_snapshot();
        baseline.draw_duration_histogram.record(1).unwrap();
        state.baseline = Some(baseline);
        assert!(state.finish(window, 1_000).is_err());
        assert!(!directory.output().exists());
        state.baseline = Some(window.frame_duration_snapshot());
        state.output = directory.0.join("missing/report.json");
        assert!(state.finish(window, 1_000).is_err());
        assert!(!state.output.exists());
    });
}
