use crate::domain::{IncidentStatus, Scope};
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
    assert_eq!(
        snapshot
            .visible_incidents
            .iter()
            .map(|incident| incident.id)
            .collect::<Vec<_>>(),
        [4201, 4202, 4204, 4203]
    );

    console.scope = Scope::Unassigned;
    assert_eq!(console.snapshot().visible_incidents[0].id, 4203);
}

#[test]
fn assignment_and_lifecycle_transitions_record_events() {
    let mut console = IncidentConsole::default();
    console.selected_id = 4203;
    let initial_events = console.snapshot().selected.unwrap().events.len();

    console.assign_selected();
    console.advance_selected();
    let selected = console.snapshot().selected.unwrap();

    assert_eq!(selected.owner, Some("You"));
    assert_eq!(selected.status, IncidentStatus::Investigating);
    assert_eq!(selected.events.len(), initial_events + 2);
    assert_eq!(selected.events.last().unwrap().actor, "You");
}

#[test]
fn simulated_signals_respect_auto_triage() {
    let mut console = IncidentConsole::default();

    console.simulate_signal();
    let first = console.snapshot().selected.unwrap();
    assert_eq!(first.id, 4205);
    assert_eq!(first.owner, Some("Auto triage"));
    assert_eq!(first.status, IncidentStatus::Investigating);
    assert_eq!(first.events.len(), 2);

    console.auto_triage = false;
    console.simulate_signal();
    let second = console.snapshot().selected.unwrap();
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
    assert!(empty.visible_incidents.is_empty());
    assert!(empty.selected.is_none());

    console.simulate_signal();
    let recovered = console.snapshot();
    assert_eq!(recovered.stats.active, 1);
    assert_eq!(recovered.selected.unwrap().id, 4205);
}
