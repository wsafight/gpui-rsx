#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    Active,
    Unassigned,
    All,
}

impl Scope {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Unassigned => "Unassigned",
            Self::All => "All incidents",
        }
    }

    pub(crate) fn action(self) -> &'static str {
        match self {
            Self::Active => "Showing active incidents",
            Self::Unassigned => "Showing unassigned incidents",
            Self::All => "Showing incident history",
        }
    }

    pub(crate) fn matches(self, incident: &Incident) -> bool {
        match self {
            Self::Active => incident.is_active(),
            Self::Unassigned => incident.is_active() && incident.owner.is_none(),
            Self::All => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SeverityFilter {
    All,
    Critical,
    High,
    Medium,
}

impl SeverityFilter {
    pub(crate) const OPTIONS: [Self; 4] = [Self::All, Self::Critical, Self::High, Self::Medium];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Critical => "Critical",
            Self::High => "High",
            Self::Medium => "Medium",
        }
    }

    pub(crate) fn selected_class(self) -> &'static str {
        match self {
            Self::All => "px-10 py-6 rounded-md bg-zinc-700 text-white cursor-pointer",
            Self::Critical => "px-10 py-6 rounded-md bg-rose-950 text-rose-300 cursor-pointer",
            Self::High => "px-10 py-6 rounded-md bg-amber-950 text-amber-300 cursor-pointer",
            Self::Medium => "px-10 py-6 rounded-md bg-sky-950 text-sky-300 cursor-pointer",
        }
    }

    pub(crate) fn matches(self, incident: &Incident) -> bool {
        match self {
            Self::All => true,
            Self::Critical => incident.severity == Severity::Critical,
            Self::High => incident.severity == Severity::High,
            Self::Medium => incident.severity == Severity::Medium,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SortMode {
    Impact,
    Newest,
}

impl SortMode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Impact => "Sort: impact",
            Self::Newest => "Sort: newest",
        }
    }

    pub(crate) fn toggled(self) -> Self {
        match self {
            Self::Impact => Self::Newest,
            Self::Newest => Self::Impact,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DetailTab {
    Overview,
    Timeline,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Severity {
    Critical,
    High,
    Medium,
}

impl Severity {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::High => "High",
            Self::Medium => "Medium",
        }
    }

    pub(crate) fn sort_key(self) -> u8 {
        match self {
            Self::Critical => 0,
            Self::High => 1,
            Self::Medium => 2,
        }
    }

    pub(crate) fn escalated(self) -> Self {
        match self {
            Self::Critical => Self::Critical,
            Self::High => Self::Critical,
            Self::Medium => Self::High,
        }
    }

    pub(crate) fn badge_class(self) -> &'static str {
        match self {
            Self::Critical => {
                "px-8 py-4 rounded-md bg-rose-950 text-rose-300 border border-rose-800"
            }
            Self::High => {
                "px-8 py-4 rounded-md bg-amber-950 text-amber-300 border border-amber-800"
            }
            Self::Medium => "px-8 py-4 rounded-md bg-sky-950 text-sky-300 border border-sky-800",
        }
    }

    pub(crate) fn dot_class(self) -> &'static str {
        match self {
            Self::Critical => "size-8 rounded-full bg-rose-500",
            Self::High => "size-8 rounded-full bg-amber-400",
            Self::Medium => "size-8 rounded-full bg-sky-400",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IncidentStatus {
    Triggered,
    Investigating,
    Monitoring,
    Resolved,
}

impl IncidentStatus {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Triggered => "Triggered",
            Self::Investigating => "Investigating",
            Self::Monitoring => "Monitoring",
            Self::Resolved => "Resolved",
        }
    }

    pub(crate) fn next(self) -> Self {
        match self {
            Self::Triggered => Self::Investigating,
            Self::Investigating => Self::Monitoring,
            Self::Monitoring => Self::Resolved,
            Self::Resolved => Self::Investigating,
        }
    }

    pub(crate) fn action_label(self) -> &'static str {
        match self {
            Self::Triggered => "Start investigation",
            Self::Investigating => "Begin monitoring",
            Self::Monitoring => "Resolve incident",
            Self::Resolved => "Reopen incident",
        }
    }

    pub(crate) fn text_class(self) -> &'static str {
        match self {
            Self::Triggered => "text-sm text-rose-300",
            Self::Investigating => "text-sm text-amber-300",
            Self::Monitoring => "text-sm text-sky-300",
            Self::Resolved => "text-sm text-emerald-300",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum EventTone {
    Alert,
    Action,
    Recovery,
    Note,
}

impl EventTone {
    pub(crate) fn dot_class(self) -> &'static str {
        match self {
            Self::Alert => "size-8 rounded-full bg-rose-500",
            Self::Action => "size-8 rounded-full bg-sky-400",
            Self::Recovery => "size-8 rounded-full bg-emerald-400",
            Self::Note => "size-8 rounded-full bg-neutral-500",
        }
    }
}

#[derive(Clone)]
pub(crate) struct TimelineEvent {
    pub(crate) id: usize,
    pub(crate) stamp: String,
    pub(crate) actor: &'static str,
    pub(crate) message: String,
    pub(crate) tone: EventTone,
}

impl TimelineEvent {
    pub(crate) fn new(
        id: usize,
        stamp: impl Into<String>,
        actor: &'static str,
        message: impl Into<String>,
        tone: EventTone,
    ) -> Self {
        Self {
            id,
            stamp: stamp.into(),
            actor,
            message: message.into(),
            tone,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Incident {
    pub(crate) id: u32,
    pub(crate) title: String,
    pub(crate) service: &'static str,
    pub(crate) region: &'static str,
    pub(crate) owner: Option<&'static str>,
    pub(crate) severity: Severity,
    pub(crate) status: IncidentStatus,
    pub(crate) age_minutes: u32,
    pub(crate) customers: u32,
    pub(crate) latency_ms: u32,
    pub(crate) error_rate: f32,
    pub(crate) events: Vec<TimelineEvent>,
}

impl Incident {
    pub(crate) fn new(
        id: u32,
        title: impl Into<String>,
        service: &'static str,
        region: &'static str,
        severity: Severity,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            service,
            region,
            owner: None,
            severity,
            status: IncidentStatus::Triggered,
            age_minutes: 0,
            customers: 0,
            latency_ms: 0,
            error_rate: 0.0,
            events: Vec::new(),
        }
    }

    pub(crate) fn owner(mut self, owner: &'static str) -> Self {
        self.owner = Some(owner);
        self
    }

    pub(crate) fn status(mut self, status: IncidentStatus) -> Self {
        self.status = status;
        self
    }

    pub(crate) fn age(mut self, minutes: u32) -> Self {
        self.age_minutes = minutes;
        self
    }

    pub(crate) fn impact(mut self, customers: u32, latency_ms: u32, error_rate: f32) -> Self {
        self.customers = customers;
        self.latency_ms = latency_ms;
        self.error_rate = error_rate;
        self
    }

    pub(crate) fn events(mut self, events: Vec<TimelineEvent>) -> Self {
        self.events = events;
        self
    }

    pub(crate) fn is_active(&self) -> bool {
        self.status != IncidentStatus::Resolved
    }

    pub(crate) fn age_label(&self) -> String {
        if self.age_minutes >= 60 {
            format!("{}h {}m", self.age_minutes / 60, self.age_minutes % 60)
        } else {
            format!("{}m", self.age_minutes)
        }
    }
}
