use crate::domain::{DetailTab, IncidentStatus, Scope, SeverityFilter};
use crate::model::IncidentConsole;

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
