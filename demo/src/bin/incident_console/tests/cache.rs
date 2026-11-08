use crate::domain::{
    DetailTab, Incident, IncidentStatus, Scope, Severity, SeverityFilter, SortMode,
};
use crate::model::IncidentConsole;
use crate::sample_data;
use std::{cmp::Reverse, collections::HashSet, rc::Rc};

fn assert_fresh_snapshot(console: &IncidentConsole) {
    let snapshot = console.snapshot();
    let records = console.incidents();
    let active = records
        .iter()
        .filter(|record| record.is_active())
        .collect::<Vec<_>>();
    assert_eq!(snapshot.stats.total, records.len());
    assert_eq!(snapshot.stats.active, active.len());
    assert_eq!(snapshot.stats.resolved, records.len() - active.len());
    assert_eq!(
        snapshot.stats.critical,
        active
            .iter()
            .filter(|record| record.severity == Severity::Critical)
            .count()
    );
    assert_eq!(
        snapshot.stats.unassigned,
        active
            .iter()
            .filter(|record| record.owner.is_none())
            .count()
    );
    assert_eq!(
        snapshot.stats.affected_customers,
        active.iter().map(|record| record.customers).sum::<u32>()
    );
    assert_eq!(
        snapshot.stats.service_count,
        records
            .iter()
            .map(|record| record.service)
            .collect::<HashSet<_>>()
            .len()
    );
    let mut visible = records
        .iter()
        .filter(|record| console.scope.matches(record) && console.severity_filter.matches(record))
        .collect::<Vec<_>>();
    match console.sort_mode {
        SortMode::Impact => {
            visible.sort_by_key(|record| (record.severity.sort_key(), Reverse(record.customers)))
        }
        SortMode::Newest => visible.sort_by_key(|record| record.age_minutes),
    }
    assert_eq!(
        snapshot.visible_ids,
        visible.iter().map(|record| record.id).collect::<Vec<_>>()
    );
    assert_eq!(snapshot.visible_ids.len(), snapshot.visible_indices.len());
    for (id, position) in snapshot.visible_ids.iter().zip(&snapshot.visible_indices) {
        assert_eq!(records[*position].id, *id);
    }
    if let Some(selected) = console.selected_incident() {
        assert_eq!(selected.id, console.selected_id);
        assert!(snapshot.visible_ids.contains(&selected.id));
    }
    assert!(Rc::ptr_eq(&snapshot, &console.snapshot()));
}

#[test]
fn every_query_combination_matches_uncached_records() {
    let mut console = IncidentConsole::default();
    console.load_rows(1_000);
    for scope in [Scope::Active, Scope::Unassigned, Scope::All] {
        for filter in SeverityFilter::OPTIONS {
            for sort in [SortMode::Impact, SortMode::Newest] {
                console.scope = scope;
                console.severity_filter = filter;
                console.sort_mode = sort;
                assert_fresh_snapshot(&console);
            }
        }
    }
}

#[test]
fn every_data_mutation_invalidates_the_cache_without_mutating_old_snapshots() {
    let mutations: [fn(&mut IncidentConsole); 5] = [
        IncidentConsole::assign_selected,
        IncidentConsole::advance_selected,
        IncidentConsole::escalate_selected,
        IncidentConsole::simulate_signal,
        IncidentConsole::clear_resolved,
    ];
    for mutate in mutations {
        let mut console = IncidentConsole::default();
        console.set_scope(Scope::All);
        let old = console.snapshot();
        let old_ids = old.visible_ids.clone();
        let old_indices = old.visible_indices.clone();
        mutate(&mut console);
        assert!(!Rc::ptr_eq(&old, &console.snapshot()));
        assert_eq!(old.visible_ids, old_ids);
        assert_eq!(old.visible_indices, old_indices);
        assert_fresh_snapshot(&console);
    }
}

#[test]
fn query_setters_reconcile_selection_through_repeated_mutations() {
    let mut console = IncidentConsole::default();
    console.load_rows(1_000);
    for round in 0..32 {
        console.set_scope([Scope::All, Scope::Unassigned, Scope::Active][round % 3]);
        console.set_severity_filter(SeverityFilter::OPTIONS[round % 4]);
        console.toggle_sort();
        assert_fresh_snapshot(&console);
        let visible = console.snapshot();
        assert!(
            visible.visible_ids.is_empty() || visible.visible_ids.contains(&console.selected_id)
        );
        match round % 4 {
            0 => console.assign_selected(),
            1 => console.advance_selected(),
            2 => console.escalate_selected(),
            _ => console.simulate_signal(),
        }
        if round % 5 == 0 {
            console.clear_resolved();
        }
        assert_fresh_snapshot(&console);
    }
}

#[test]
fn replacing_and_reordering_records_rebuilds_id_positions() {
    let mut console = IncidentConsole::default();
    console.load_rows(1_000);
    console.selected_id = 10_417;
    let selected_title = console.selected_incident().unwrap().title.clone();
    let old = console.snapshot();
    let mut reversed = console.incidents().to_vec();
    reversed.reverse();
    console.replace_incidents(reversed);
    assert!(!Rc::ptr_eq(&old, &console.snapshot()));
    assert_eq!(console.selected_id, 10_417);
    assert_eq!(console.selected_incident().unwrap().title, selected_title);
    assert_fresh_snapshot(&console);
    console.replace_incidents(console.incidents()[..2].to_vec());
    assert_ne!(console.selected_id, 10_417);
    assert_fresh_snapshot(&console);
    console.simulate_signal();
    let ids = console
        .incidents()
        .iter()
        .map(|record| record.id)
        .collect::<HashSet<_>>();
    assert_eq!(ids.len(), console.incidents().len());
    assert_fresh_snapshot(&console);
}

#[test]
fn empty_and_single_row_datasets_survive_all_actions_and_recovery() {
    let mut console = IncidentConsole::default();
    console.load_rows(0);
    assert_eq!(console.selected_id, 0);
    assert!(console.selected_incident().is_none());
    let empty = console.snapshot();
    console.assign_selected();
    console.advance_selected();
    console.escalate_selected();
    assert!(Rc::ptr_eq(&empty, &console.snapshot()));
    console.clear_resolved();
    console.toggle_sort();
    assert_fresh_snapshot(&console);
    console.load_rows(1);
    assert_eq!(console.selected_id, 10_000);
    console.detail_tab = DetailTab::Timeline;
    console.set_severity_filter(SeverityFilter::Medium);
    assert_eq!(console.selected_id, 0);
    assert!(console.detail_tab == DetailTab::Overview);
    console.reset_filters();
    assert_eq!(console.selected_id, 10_000);
    assert_fresh_snapshot(&console);
}

#[test]
fn equal_sort_keys_keep_input_order_and_business_ids() {
    let mut console = IncidentConsole::default();
    let records = [900, 5, 42].map(|id| {
        Incident::new(id, "Tie", "API", "global", Severity::High)
            .impact(100, 10, 0.1)
            .age(5)
            .status(IncidentStatus::Triggered)
    });
    console.replace_incidents(records.into());
    for sort in [SortMode::Impact, SortMode::Newest] {
        console.sort_mode = sort;
        assert_eq!(console.snapshot().visible_ids, [900, 5, 42]);
        assert_fresh_snapshot(&console);
    }
}

#[test]
fn scaled_data_has_unique_ids_and_does_not_copy_template_event_histories() {
    for count in [0, 1, 6, 1_000, 100_000] {
        let records = sample_data::scaled_incidents(count);
        assert_eq!(records.len(), count);
        assert_eq!(
            records
                .iter()
                .map(|record| record.id)
                .collect::<HashSet<_>>()
                .len(),
            count
        );
        assert!(records.iter().all(|record| record.events.is_empty()));
        if let Some(last) = records.last() {
            assert_eq!(last.id, 10_000 + count as u32 - 1);
        }
    }
    let first = sample_data::scaled_incidents(1_000);
    let second = sample_data::scaled_incidents(1_000);
    for (left, right) in first.iter().zip(&second) {
        assert_eq!(
            (left.id, &left.title, left.age_minutes),
            (right.id, &right.title, right.age_minutes)
        );
    }
}
