use std::cell::{Cell, RefCell};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::domain::{
    DetailTab, EventTone, Incident, IncidentStatus, Scope, Severity, SeverityFilter, SortMode,
    TimelineEvent,
};
use crate::sample_data;

pub(crate) struct ConsoleStats {
    pub(crate) total: usize,
    pub(crate) active: usize,
    pub(crate) critical: usize,
    pub(crate) unassigned: usize,
    pub(crate) resolved: usize,
    pub(crate) affected_customers: u32,
    pub(crate) service_count: usize,
}

pub(crate) struct ConsoleSnapshot {
    pub(crate) stats: ConsoleStats,
    pub(crate) visible_ids: Vec<u32>,
    pub(crate) visible_indices: Vec<usize>,
}

struct CachedSnapshot {
    scope: Scope,
    filter: SeverityFilter,
    sort: SortMode,
    snapshot: Rc<ConsoleSnapshot>,
}

pub(crate) struct IncidentConsole {
    incidents: Vec<Incident>,
    positions: HashMap<u32, usize>,
    snapshot_cache: RefCell<Option<CachedSnapshot>>,
    pub(crate) queue_scroll: gpui::UniformListScrollHandle,
    pub(crate) rendered_rows: Cell<usize>,
    pub(crate) scope: Scope,
    pub(crate) severity_filter: SeverityFilter,
    pub(crate) sort_mode: SortMode,
    pub(crate) detail_tab: DetailTab,
    pub(crate) selected_id: u32,
    pub(crate) auto_triage: bool,
    pub(crate) last_action: String,
    next_incident_id: u32,
    next_event_id: usize,
    simulation_round: usize,
    #[cfg(feature = "native-profiler")]
    pub(crate) profile: Option<crate::profile::Profile>,
}

impl Default for IncidentConsole {
    fn default() -> Self {
        let incidents = sample_data::incidents();
        let positions = incidents
            .iter()
            .enumerate()
            .map(|(index, incident)| (incident.id, index))
            .collect();
        Self {
            incidents,
            positions,
            snapshot_cache: RefCell::new(None),
            queue_scroll: gpui::UniformListScrollHandle::new(),
            rendered_rows: Cell::new(0),
            scope: Scope::Active,
            severity_filter: SeverityFilter::All,
            sort_mode: SortMode::Impact,
            detail_tab: DetailTab::Overview,
            selected_id: 4201,
            auto_triage: true,
            last_action: "Operations workspace synchronized".into(),
            next_incident_id: 4205,
            next_event_id: 12,
            simulation_round: 0,
            #[cfg(feature = "native-profiler")]
            profile: None,
        }
    }
}

impl IncidentConsole {
    pub(crate) fn snapshot(&self) -> Rc<ConsoleSnapshot> {
        if let Some(cached) = self.snapshot_cache.borrow().as_ref()
            && cached.scope == self.scope
            && cached.filter == self.severity_filter
            && cached.sort == self.sort_mode
        {
            return cached.snapshot.clone();
        }
        let total = self.incidents.len();
        let mut active = 0;
        let mut critical = 0;
        let mut unassigned = 0;
        let mut affected_customers = 0;
        let mut services = HashSet::new();
        for incident in &self.incidents {
            services.insert(incident.service);
            if incident.is_active() {
                active += 1;
                affected_customers += incident.customers;
                if incident.severity == Severity::Critical {
                    critical += 1;
                }
                if incident.owner.is_none() {
                    unassigned += 1;
                }
            }
        }

        let mut visible_incidents = self
            .incidents
            .iter()
            .enumerate()
            .filter(|(_, incident)| {
                self.scope.matches(incident) && self.severity_filter.matches(incident)
            })
            .map(|(index, incident)| {
                (
                    incident.id,
                    incident.severity.sort_key(),
                    Reverse(incident.customers),
                    incident.age_minutes,
                    index,
                )
            })
            .collect::<Vec<_>>();
        match self.sort_mode {
            SortMode::Impact => visible_incidents
                .sort_by_key(|(_, severity, customers, _, _)| (*severity, *customers)),
            SortMode::Newest => visible_incidents.sort_by_key(|(_, _, _, age, _)| *age),
        }
        let visible_ids = visible_incidents
            .iter()
            .map(|(id, _, _, _, _)| *id)
            .collect();
        let visible_indices = visible_incidents
            .into_iter()
            .map(|(_, _, _, _, index)| index)
            .collect();

        let snapshot = Rc::new(ConsoleSnapshot {
            stats: ConsoleStats {
                total,
                active,
                critical,
                unassigned,
                resolved: total - active,
                affected_customers,
                service_count: services.len(),
            },
            visible_ids,
            visible_indices,
        });
        self.snapshot_cache.replace(Some(CachedSnapshot {
            scope: self.scope,
            filter: self.severity_filter,
            sort: self.sort_mode,
            snapshot: snapshot.clone(),
        }));
        snapshot
    }

    pub(crate) fn incidents(&self) -> &[Incident] {
        &self.incidents
    }

    pub(crate) fn replace_incidents(&mut self, incidents: Vec<Incident>) {
        self.next_incident_id = incidents
            .iter()
            .map(|incident| incident.id)
            .max()
            .unwrap_or(4204)
            + 1;
        self.incidents = incidents;
        self.reindex();
        self.reconcile_selection();
    }

    pub(crate) fn load_rows(&mut self, count: usize) {
        self.scope = Scope::All;
        self.severity_filter = SeverityFilter::All;
        self.replace_incidents(sample_data::scaled_incidents(count));
        self.last_action = format!("Loaded {count} incidents");
    }

    fn reindex(&mut self) {
        self.positions = self
            .incidents
            .iter()
            .enumerate()
            .map(|(index, incident)| (incident.id, index))
            .collect();
        self.snapshot_cache.borrow_mut().take();
    }

    pub(crate) fn selected_incident(&self) -> Option<&Incident> {
        self.positions
            .get(&self.selected_id)
            .and_then(|index| self.incidents.get(*index))
            .filter(|incident| {
                self.scope.matches(incident) && self.severity_filter.matches(incident)
            })
    }

    fn reconcile_selection(&mut self) {
        let snapshot = self.snapshot();
        if !snapshot.visible_ids.contains(&self.selected_id) {
            self.selected_id = snapshot.visible_ids.first().copied().unwrap_or(0);
            self.detail_tab = DetailTab::Overview;
        }
        if let Some(index) = snapshot
            .visible_ids
            .iter()
            .position(|id| *id == self.selected_id)
        {
            self.queue_scroll
                .scroll_to_item(index, gpui::ScrollStrategy::Nearest);
        }
    }

    pub(crate) fn set_scope(&mut self, scope: Scope) {
        self.scope = scope;
        self.last_action = scope.action().into();
        self.reconcile_selection();
    }

    pub(crate) fn set_severity_filter(&mut self, filter: SeverityFilter) {
        self.severity_filter = filter;
        self.last_action = format!("Severity filter: {}", filter.label());
        self.reconcile_selection();
    }

    pub(crate) fn reset_filters(&mut self) {
        self.scope = Scope::Active;
        self.severity_filter = SeverityFilter::All;
        self.last_action = "Showing all active incidents".into();
        self.reconcile_selection();
    }

    fn record_event(
        &mut self,
        incident_id: u32,
        actor: &'static str,
        message: impl Into<String>,
        tone: EventTone,
    ) {
        let event = TimelineEvent::new(self.next_event_id, "just now", actor, message, tone);
        self.next_event_id += 1;
        if let Some(index) = self.positions.get(&incident_id)
            && let Some(incident) = self.incidents.get_mut(*index)
        {
            incident.events.push(event);
        }
    }

    pub(crate) fn assign_selected(&mut self) {
        let incident_id = self.selected_id;
        let Some(incident) = self
            .incidents
            .iter_mut()
            .find(|incident| incident.id == incident_id)
        else {
            return;
        };
        incident.owner = Some("You");
        self.snapshot_cache.borrow_mut().take();
        self.record_event(
            incident_id,
            "You",
            "Ownership accepted from the response queue.",
            EventTone::Note,
        );
        self.last_action = format!("Assigned incident #{incident_id} to you");
        self.reconcile_selection();
    }

    pub(crate) fn advance_selected(&mut self) {
        let incident_id = self.selected_id;
        let Some(status) = self
            .incidents
            .iter_mut()
            .find(|incident| incident.id == incident_id)
            .map(|incident| {
                incident.status = incident.status.next();
                incident.status
            })
        else {
            return;
        };
        self.snapshot_cache.borrow_mut().take();
        let tone = if status == IncidentStatus::Resolved {
            EventTone::Recovery
        } else {
            EventTone::Action
        };
        self.record_event(
            incident_id,
            "You",
            format!("Incident moved to {}.", status.label()),
            tone,
        );
        self.last_action = format!("Incident #{incident_id} is now {}", status.label());
        self.reconcile_selection();
    }

    pub(crate) fn escalate_selected(&mut self) {
        let incident_id = self.selected_id;
        let Some((previous, severity)) = self
            .incidents
            .iter_mut()
            .find(|incident| incident.id == incident_id)
            .map(|incident| {
                let previous = incident.severity;
                incident.severity = incident.severity.escalated();
                (previous, incident.severity)
            })
        else {
            return;
        };
        self.snapshot_cache.borrow_mut().take();
        let message = if previous == severity {
            "Severity reviewed and remains Critical.".to_string()
        } else {
            format!("Severity escalated to {}.", severity.label())
        };
        self.record_event(incident_id, "You", message, EventTone::Alert);
        self.last_action = format!("Reviewed severity for incident #{incident_id}");
        self.reconcile_selection();
    }

    pub(crate) fn toggle_sort(&mut self) {
        self.sort_mode = self.sort_mode.toggled();
        self.last_action = match self.sort_mode {
            SortMode::Impact => "Sorted by customer impact".into(),
            SortMode::Newest => "Sorted by newest signal".into(),
        };
        self.reconcile_selection();
    }

    pub(crate) fn simulate_signal(&mut self) {
        let signal = sample_data::simulated_signal(self.simulation_round);
        self.simulation_round += 1;
        let incident_id = self.next_incident_id;
        self.next_incident_id += 1;

        let mut incident = Incident::new(
            incident_id,
            signal.title,
            signal.service,
            signal.region,
            signal.severity,
        )
        .impact(signal.customers, signal.latency_ms, signal.error_rate)
        .events(vec![TimelineEvent::new(
            self.next_event_id,
            "just now",
            "Signal simulator",
            "A new production signal opened this incident.",
            EventTone::Alert,
        )]);
        self.next_event_id += 1;

        if self.auto_triage {
            incident.owner = Some("Auto triage");
            incident.status = IncidentStatus::Investigating;
            incident.events.push(TimelineEvent::new(
                self.next_event_id,
                "just now",
                "Auto triage",
                "Runbook matched and initial investigation started.",
                EventTone::Action,
            ));
            self.next_event_id += 1;
        }

        self.incidents.insert(0, incident);
        self.reindex();
        self.selected_id = incident_id;
        self.scope = Scope::Active;
        self.severity_filter = SeverityFilter::All;
        self.detail_tab = DetailTab::Overview;
        self.last_action = format!("Injected production signal #{incident_id}");
        self.reconcile_selection();
    }

    pub(crate) fn clear_resolved(&mut self) {
        let previous_len = self.incidents.len();
        self.incidents.retain(Incident::is_active);
        self.reindex();
        let removed = previous_len - self.incidents.len();
        self.reconcile_selection();
        self.last_action = format!("Cleared {removed} resolved incidents");
    }
}
