use crate::domain::{EventTone, Incident, IncidentStatus, Severity, TimelineEvent};

pub(crate) struct Signal {
    pub(crate) title: &'static str,
    pub(crate) service: &'static str,
    pub(crate) region: &'static str,
    pub(crate) severity: Severity,
    pub(crate) customers: u32,
    pub(crate) latency_ms: u32,
    pub(crate) error_rate: f32,
}

pub(crate) fn simulated_signal(round: usize) -> Signal {
    match round % 4 {
        0 => Signal {
            title: "API gateway error budget burn",
            service: "Gateway",
            region: "us-east-1",
            severity: Severity::Critical,
            customers: 1320,
            latency_ms: 1120,
            error_rate: 6.7,
        },
        1 => Signal {
            title: "Notification queue depth rising",
            service: "Messaging",
            region: "eu-west-1",
            severity: Severity::High,
            customers: 480,
            latency_ms: 690,
            error_rate: 2.9,
        },
        2 => Signal {
            title: "Analytics ingestion lag detected",
            service: "Analytics",
            region: "global",
            severity: Severity::Medium,
            customers: 170,
            latency_ms: 410,
            error_rate: 1.2,
        },
        _ => Signal {
            title: "Session cache eviction spike",
            service: "Sessions",
            region: "ap-northeast-1",
            severity: Severity::High,
            customers: 760,
            latency_ms: 830,
            error_rate: 4.1,
        },
    }
}

pub(crate) fn incidents() -> Vec<Incident> {
    vec![
        Incident::new(
            4201,
            "Checkout latency above regional SLO",
            "Checkout",
            "eu-west-1",
            Severity::Critical,
        )
        .owner("Nia")
        .status(IncidentStatus::Investigating)
        .age(18)
        .impact(1840, 1480, 8.4)
        .events(vec![
            event(
                1,
                "18m ago",
                "SLO monitor",
                "P95 latency crossed the 900 ms threshold.",
                EventTone::Alert,
            ),
            event(
                2,
                "14m ago",
                "Nia",
                "Traffic shifted away from the degraded zone.",
                EventTone::Action,
            ),
            event(
                3,
                "6m ago",
                "Deploy bot",
                "Rollback completed on checkout-api.",
                EventTone::Recovery,
            ),
        ]),
        Incident::new(
            4202,
            "Search index freshness is delayed",
            "Search",
            "us-east-1",
            Severity::High,
        )
        .owner("Owen")
        .status(IncidentStatus::Monitoring)
        .age(43)
        .impact(620, 740, 2.1)
        .events(vec![
            event(
                4,
                "43m ago",
                "Freshness monitor",
                "Index lag exceeded the five minute budget.",
                EventTone::Alert,
            ),
            event(
                5,
                "11m ago",
                "Owen",
                "Backlog drained; monitoring replica convergence.",
                EventTone::Recovery,
            ),
        ]),
        Incident::new(
            4203,
            "Webhook delivery retries elevated",
            "Webhooks",
            "global",
            Severity::Medium,
        )
        .age(9)
        .impact(210, 320, 1.4)
        .events(vec![event(
            6,
            "9m ago",
            "Queue monitor",
            "Retry volume increased for two destination networks.",
            EventTone::Alert,
        )]),
        Incident::new(
            4204,
            "Token refresh failures after key rotation",
            "Identity",
            "ap-southeast-1",
            Severity::High,
        )
        .owner("Mara")
        .status(IncidentStatus::Investigating)
        .age(27)
        .impact(390, 560, 3.8)
        .events(vec![
            event(
                7,
                "27m ago",
                "Auth monitor",
                "Refresh failure rate crossed the alert threshold.",
                EventTone::Alert,
            ),
            event(
                8,
                "19m ago",
                "Mara",
                "Key propagation audit started across edge regions.",
                EventTone::Action,
            ),
        ]),
        Incident::new(
            4198,
            "Billing export jobs stalled",
            "Billing",
            "us-west-2",
            Severity::Medium,
        )
        .owner("Jules")
        .status(IncidentStatus::Resolved)
        .age(126)
        .impact(84, 190, 0.6)
        .events(vec![
            event(
                9,
                "2h 6m ago",
                "Job monitor",
                "Export queue stopped making progress.",
                EventTone::Alert,
            ),
            event(
                10,
                "48m ago",
                "Jules",
                "Workers recovered and the backlog cleared.",
                EventTone::Recovery,
            ),
        ]),
        Incident::new(
            4195,
            "Image processing saturation",
            "Media",
            "eu-central-1",
            Severity::High,
        )
        .owner("Nia")
        .status(IncidentStatus::Resolved)
        .age(188)
        .impact(450, 880, 4.2)
        .events(vec![event(
            11,
            "3h 8m ago",
            "Capacity monitor",
            "Worker pool scaled and processing latency recovered.",
            EventTone::Recovery,
        )]),
    ]
}

fn event(
    id: usize,
    stamp: &'static str,
    actor: &'static str,
    message: &'static str,
    tone: EventTone,
) -> TimelineEvent {
    TimelineEvent::new(id, stamp, actor, message, tone)
}
