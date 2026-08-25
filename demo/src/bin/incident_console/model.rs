use std::cmp::Reverse;
use std::collections::BTreeSet;

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
    pub(crate) visible_incidents: Vec<Incident>,
    pub(crate) selected: Option<Incident>,
}

pub(crate) struct IncidentConsole {
    pub(crate) incidents: Vec<Incident>,
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
}

impl Default for IncidentConsole {
    fn default() -> Self {
        Self {
            incidents: sample_data::incidents(),
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
        }
    }
}

impl IncidentConsole {
    pub(crate) fn snapshot(&self) -> ConsoleSnapshot {
        let total = self.incidents.len();
        let active = self
            .incidents
            .iter()
            .filter(|incident| incident.is_active())
            .count();
        let critical = self
            .incidents
            .iter()
            .filter(|incident| incident.is_active() && incident.severity == Severity::Critical)
            .count();
        let unassigned = self
            .incidents
            .iter()
            .filter(|incident| incident.is_active() && incident.owner.is_none())
            .count();
        let affected_customers = self
            .incidents
            .iter()
            .filter(|incident| incident.is_active())
            .map(|incident| incident.customers)
            .sum();
        let service_count = self
            .incidents
            .iter()
            .map(|incident| incident.service)
            .collect::<BTreeSet<_>>()
            .len();

        let mut visible_incidents = self
            .incidents
            .iter()
            .filter(|incident| {
                self.scope.matches(incident) && self.severity_filter.matches(incident)
            })
            .cloned()
            .collect::<Vec<_>>();
        match self.sort_mode {
            SortMode::Impact => visible_incidents.sort_by_key(|incident| {
                (incident.severity.sort_key(), Reverse(incident.customers))
            }),
            SortMode::Newest => {
                visible_incidents.sort_by_key(|incident| incident.age_minutes);
            }
        }

        ConsoleSnapshot {
            stats: ConsoleStats {
                total,
                active,
                critical,
                unassigned,
                resolved: total - active,
                affected_customers,
                service_count,
            },
            visible_incidents,
            selected: self
                .incidents
                .iter()
                .find(|incident| incident.id == self.selected_id)
                .or_else(|| self.incidents.first())
                .cloned(),
        }
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
        if let Some(incident) = self
            .incidents
            .iter_mut()
            .find(|incident| incident.id == incident_id)
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
        self.record_event(
            incident_id,
            "You",
            "Ownership accepted from the response queue.",
            EventTone::Note,
        );
        self.last_action = format!("Assigned incident #{incident_id} to you");
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
        let message = if previous == severity {
            "Severity reviewed and remains Critical.".to_string()
        } else {
            format!("Severity escalated to {}.", severity.label())
        };
        self.record_event(incident_id, "You", message, EventTone::Alert);
        self.last_action = format!("Reviewed severity for incident #{incident_id}");
    }

    pub(crate) fn toggle_sort(&mut self) {
        self.sort_mode = self.sort_mode.toggled();
        self.last_action = match self.sort_mode {
            SortMode::Impact => "Sorted by customer impact".into(),
            SortMode::Newest => "Sorted by newest signal".into(),
        };
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
        self.selected_id = incident_id;
        self.scope = Scope::Active;
        self.severity_filter = SeverityFilter::All;
        self.detail_tab = DetailTab::Overview;
        self.last_action = format!("Injected production signal #{incident_id}");
    }

    pub(crate) fn clear_resolved(&mut self) {
        let previous_len = self.incidents.len();
        self.incidents.retain(Incident::is_active);
        let removed = previous_len - self.incidents.len();
        if !self
            .incidents
            .iter()
            .any(|incident| incident.id == self.selected_id)
        {
            self.selected_id = self.incidents.first().map_or(0, |incident| incident.id);
        }
        self.last_action = format!("Cleared {removed} resolved incidents");
    }
}
