use crate::domain::{DetailTab, IncidentStatus, Scope, SeverityFilter};
use crate::model::IncidentConsole;

#[gpui::test]
fn filtering_and_resolving_dispatch_through_the_console(cx: &mut gpui::TestAppContext) {
    use gpui::test::TestWindowExt;
    use gpui::{AppContext, Bounds, WindowBounds, WindowOptions, px, size};

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
    let click = |cx: &mut gpui::TestAppContext, target: &str| {
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click(target.to_owned(), cx);
        })
        .unwrap();
    };

    click(cx, "scope-Unassigned");
    view.read_with(cx, |view, _| assert_eq!(view.selected_id, 4203));
    click(cx, "detail-timeline");
    view.read_with(cx, |view, _| {
        assert!(view.detail_tab == DetailTab::Timeline)
    });
    click(cx, "severity-Critical");
    view.read_with(cx, |view, _| {
        assert!(view.snapshot().visible_ids.is_empty());
        assert!(view.selected_incident().is_none());
    });
    click(cx, "recover-queue");
    view.read_with(cx, |view, _| {
        assert!(view.scope == Scope::Active);
        assert!(view.severity_filter == SeverityFilter::All);
    });
    click(cx, "incident-4203");
    click(cx, "advance-incident");
    click(cx, "advance-incident");
    click(cx, "advance-incident");
    view.read_with(cx, |view, _| {
        assert!(!view.snapshot().visible_ids.contains(&4203));
        assert_ne!(view.selected_id, 4203);
    });
    view.update(cx, |view, cx| {
        for incident in &mut view.incidents {
            incident.status = IncidentStatus::Resolved;
        }
        view.clear_resolved();
        cx.notify();
    });
    click(cx, "recover-queue");
    view.read_with(cx, |view, _| {
        assert_eq!(view.snapshot().stats.active, 1);
        assert!(view.selected_incident().is_some());
    });
}

#[test]
fn snapshot_derives_metrics_filters_and_impact_order() {
    let mut console = IncidentConsole::default();
    let snapshot = console.snapshot();

    assert_eq!(snapshot.stats.total, 6);
    assert_eq!(snapshot.stats.active, 4);
    assert_eq!(snapshot.stats.critical, 1);
    assert_eq!(snapshot.stats.unassigned, 1);
    assert_eq!(snapshot.stats.affected_customers, 3060);
    assert_eq!(snapshot.visible_ids, [4201, 4202, 4204, 4203]);

    console.scope = Scope::Unassigned;
    assert_eq!(console.snapshot().visible_ids[0], 4203);
}

#[test]
fn assignment_and_lifecycle_transitions_record_events() {
    let mut console = IncidentConsole::default();
    console.selected_id = 4203;
    let initial_events = console.selected_incident().unwrap().events.len();

    console.assign_selected();
    console.advance_selected();
    let selected = console.selected_incident().unwrap();

    assert_eq!(selected.owner, Some("You"));
    assert_eq!(selected.status, IncidentStatus::Investigating);
    assert_eq!(selected.events.len(), initial_events + 2);
    assert_eq!(selected.events.last().unwrap().actor, "You");
}

#[test]
fn simulated_signals_respect_auto_triage() {
    let mut console = IncidentConsole::default();

    console.simulate_signal();
    let first = console.selected_incident().unwrap();
    assert_eq!(first.id, 4205);
    assert_eq!(first.owner, Some("Auto triage"));
    assert_eq!(first.status, IncidentStatus::Investigating);
    assert_eq!(first.events.len(), 2);

    console.auto_triage = false;
    console.simulate_signal();
    let second = console.selected_incident().unwrap();
    assert_eq!(second.id, 4206);
    assert_eq!(second.owner, None);
    assert_eq!(second.status, IncidentStatus::Triggered);
    assert_eq!(second.events.len(), 1);
}

#[test]
fn an_empty_queue_can_recover_without_restarting() {
    let mut console = IncidentConsole::default();
    for incident in &mut console.incidents {
        incident.status = IncidentStatus::Resolved;
    }

    console.clear_resolved();
    let empty = console.snapshot();
    assert!(empty.visible_ids.is_empty());
    assert!(console.selected_incident().is_none());

    console.simulate_signal();
    let recovered = console.snapshot();
    assert_eq!(recovered.stats.active, 1);
    assert_eq!(console.selected_incident().unwrap().id, 4205);
}

#[test]
fn filters_keep_the_selected_incident_in_the_visible_queue() {
    let mut console = IncidentConsole::default();
    console.detail_tab = DetailTab::Timeline;
    console.set_scope(Scope::Unassigned);
    assert_eq!(console.selected_id, 4203);
    assert!(console.detail_tab == DetailTab::Overview);
    assert_eq!(console.selected_incident().unwrap().id, 4203);

    console.set_severity_filter(SeverityFilter::Critical);
    assert!(console.snapshot().visible_ids.is_empty());
    assert!(console.selected_incident().is_none());

    console.reset_filters();
    assert_eq!(console.selected_id, 4201);
    assert_eq!(console.selected_incident().unwrap().id, 4201);
}

#[test]
fn resolving_or_assigning_an_incident_reconciles_the_selection() {
    let mut console = IncidentConsole::default();
    console.set_scope(Scope::Unassigned);
    console.assign_selected();
    assert!(console.snapshot().visible_ids.is_empty());
    assert!(console.selected_incident().is_none());

    console.reset_filters();
    console.advance_selected();
    console.advance_selected();
    let visible = console.snapshot().visible_ids;
    assert!(!visible.contains(&4201));
    assert!(visible.contains(&console.selected_id));
    assert_eq!(console.selected_incident().unwrap().id, console.selected_id);
}
