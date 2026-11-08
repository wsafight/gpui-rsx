use crate::{
    domain::{Scope, SeverityFilter},
    model::IncidentConsole,
};
use gpui::test::TestWindowExt;
use gpui::{
    AppContext, Bounds, ScrollStrategy, TestAppContext, WindowBounds, WindowOptions, px, size,
};

#[gpui::test]
fn dataset_buttons_reset_distant_scroll_and_dispatch_current_row_ids(cx: &mut TestAppContext) {
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
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("load-100000", cx);
        assert_eq!(view.read(cx).incidents().len(), 100_000);
        let snapshot = view.read(cx).snapshot();
        let last_index = snapshot.visible_ids.len() - 1;
        let distant_id = snapshot.visible_ids[last_index];
        view.update(cx, |view, cx| {
            view.queue_scroll
                .scroll_to_item(last_index, ScrollStrategy::Top);
            cx.notify();
        });
        window.render_frame(cx);
        window.click(format!("incident-{distant_id}"), cx);
        assert_eq!(view.read(cx).selected_id, distant_id);
        assert!(view.read(cx).rendered_rows.get() < 100);
        window.click("load-1000", cx);
        assert_eq!(view.read(cx).incidents().len(), 1_000);
        let selected = view.read(cx).selected_id;
        assert!(window.try_find(format!("incident-{selected}")).is_some());
        window.click(format!("incident-{selected}"), cx);
        assert_eq!(view.read(cx).selected_incident().unwrap().id, selected);
        assert!(view.read(cx).rendered_rows.get() > 0 && view.read(cx).rendered_rows.get() < 100);
        view.update(cx, |view, cx| {
            view.load_rows(0);
            cx.notify();
        });
        window.render_frame(cx);
        assert!(window.try_find(format!("incident-{selected}")).is_none());
        window.click("recover-queue", cx);
        assert!(view.read(cx).scope == Scope::Active);
        window.click("recover-queue", cx);
        assert_eq!(view.read(cx).incidents().len(), 1);
        let recovered = view.read(cx).selected_id;
        assert!(window.try_find(format!("incident-{recovered}")).is_some());
    })
    .unwrap();
}

#[gpui::test]
fn filtering_a_distant_selection_rebuilds_only_the_visible_range(cx: &mut TestAppContext) {
    cx.update(gpui::init);
    let (handle, view) = cx.update(|cx| {
        gpui::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                    Default::default(),
                    size(px(1120.0), px(640.0)),
                ))),
                ..Default::default()
            },
            cx,
            |_, cx| {
                cx.new(|_| {
                    let mut console = IncidentConsole::default();
                    console.load_rows(100_000);
                    console
                })
            },
        )
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let snapshot = view.read(cx).snapshot();
        view.update(cx, |view, cx| {
            let target = snapshot.visible_ids.len() - 1;
            view.selected_id = snapshot.visible_ids[target];
            view.queue_scroll
                .scroll_to_item(target, ScrollStrategy::Top);
            cx.notify();
        });
        window.render_frame(cx);
        window.click("severity-Critical", cx);
        assert!(view.read(cx).severity_filter == SeverityFilter::Critical);
        let filtered = view.read(cx).snapshot();
        assert!(!filtered.visible_ids.is_empty());
        let selected = view.read(cx).selected_id;
        assert!(filtered.visible_ids.contains(&selected));
        assert!(window.try_find(format!("incident-{selected}")).is_some());
        assert!(view.read(cx).rendered_rows.get() > 0 && view.read(cx).rendered_rows.get() < 100);
        assert!(filtered.visible_ids.len() < 100_000);
    })
    .unwrap();
}
