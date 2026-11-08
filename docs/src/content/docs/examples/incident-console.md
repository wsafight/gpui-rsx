---
title: Incident Console
description: A modular, multi-pane GPUI-RSX application with filters, state transitions, and an event timeline.
---

The Incident Console models an operations queue: filter incidents, assign an owner, advance the response status, and inspect the event timeline. Its panels share one state model, with local sample data and no external services.

![Incident Console showing the incident queue, operational metrics, and selected incident details](/gpui-rsx/incident-console.png)

## Run It

The demo uses GPUI-RSX 0.11, GPUI Kit 0.8.1, and the repository's pinned Rust toolchain:

```bash
cargo run --manifest-path demo/Cargo.toml --bin incident_console --locked
```

The initial window is `1320 x 820` and contains a stable navigation rail, an incident queue, and a detail inspector.

## Interaction Model

| Area | Behavior |
| --- | --- |
| Scope | Switch between active, unassigned, and historical incidents. |
| Severity | Filter the current queue by critical, high, or medium severity. |
| Sorting | Toggle between customer impact and newest signal. |
| Automation | Enable or pause automatic ownership and investigation for simulated signals. |
| Lifecycle | Move an incident through Triggered, Investigating, Monitoring, and Resolved. |
| Timeline | Review alert, action, recovery, and note events for the selected incident. |
| Simulation | Inject deterministic production signals without external services. |

## Source Layout

The binary uses Cargo's directory-style target so the entry point stays small:

```text
demo/src/bin/incident_console/
├── main.rs         window startup only
├── domain.rs       incident types and lifecycle rules
├── model.rs        derived state and application transitions
├── sample_data.rs  initial incidents and deterministic signals
├── view.rs         top-level composition and metrics
├── sidebar.rs      scopes, automation, and history controls
├── queue.rs        filters and keyed incident rows
├── details.rs      overview actions and event timeline
└── tests.rs        metrics, lifecycle, simulation, and recovery tests
```

This is the same boundary recommended in [Best Practices](/gpui-rsx/guides/best-practices/#structure-views-by-workflow): split around meaningful UI and workflow concepts, not every small element.

## Derived View State

The snapshot contains counts and sorted incident IDs. Rendering looks up those IDs in the model. State transitions reconcile selection with the current filters, so the detail panel always refers to a visible incident:

```rust
let snapshot = self.snapshot();
let visible_incidents = snapshot.visible_ids.iter()
    .filter_map(|id| self.incidents.iter().find(|incident| incident.id == *id))
    .collect::<Vec<_>>();
let sidebar = sidebar::render(self, &snapshot.stats, cx);
let queue = queue::render(self, &visible_incidents, cx);
let details = details::render(self, self.selected_incident(), cx);

rsx! {
    <div class="size-full flex bg-demo-canvas">
        {sidebar}
        <main class="flex-1 min-w-0">
            {queue}
            {details}
        </main>
    </div>
}
```

## Data-Driven Controls

Repeated controls use ordinary Rust data, RSX loop syntax, and `key` for stable listener identity:

```rust
{for filter in SeverityFilter::OPTIONS.iter() {
    <button
        key={filter.label()}
        class={if view.severity_filter == *filter {
            filter.selected_class()
        } else {
            "px-10 py-6 rounded-md text-demo-muted cursor-pointer"
        }}
        onClick={rsx_listener!(cx, notify, {
            let filter = *filter;
            move |view, _, _window, _cx| {
                view.set_severity_filter(filter);
            }
        })}
    >
        {filter.label()}
    </button>
}}
```

## State Transitions

Listeners delegate incident changes to named methods. `rsx_listener!(cx, notify, ...)` sends the notification after the method returns; the model itself does not notify:

```rust
onClick={rsx_listener!(cx, notify, |view, _, _window, _cx| {
    view.advance_selected();
})}
```

When a filter has no results, the queue offers **Reset filters**. If the active queue itself is empty, **Simulate incident** adds a signal. Resolving or assigning an incident also updates selection if it leaves the current queue.

The shared colors live in `demo/rsx.toml`; the panels use `bg-demo-surface`, `text-demo-muted`, and related tokens instead of repeating color values. Run the state tests with `cargo test --manifest-path demo/Cargo.toml --bin incident_console --locked`. For smaller examples of the same APIs, see the [demo index](/gpui-rsx/examples/demos/).

Browse the [complete source](https://github.com/wsafight/gpui-rsx/tree/main/demo/src/bin/incident_console) or use this example as a compile-checked starting point for a GPUI desktop application.
