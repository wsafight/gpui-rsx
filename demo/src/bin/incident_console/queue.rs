use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{rsx, rsx_listener};

use crate::domain::{DetailTab, Incident, Scope, SeverityFilter};
use crate::model::IncidentConsole;

pub(crate) fn render(
    view: &IncidentConsole,
    incidents: &[&Incident],
    cx: &mut Context<IncidentConsole>,
) -> impl IntoElement + use<> {
    rsx! {
        <div class="flex-1 min-w-0 flex flex-col px-20 py-14">
            <div class="flex items-center justify-between pb-10">
                <div class="flex gap-6">
                    {for filter in SeverityFilter::OPTIONS.iter() {
                        <button
                            id={SharedString::from(format!("severity-{}", filter.label()))}
                            test_support
                            key={filter.label()}
                            hoverClass="bg-demo-raised"
                            class={if view.severity_filter == *filter {
                                filter.selected_class()
                            } else {
                                "px-10 py-6 rounded-md text-demo-muted cursor-pointer"
                            }}
                            onClick={rsx_listener!(cx, notify, {
                                let filter = *filter;
                                move |view, _, _window, _| {
                                    view.set_severity_filter(filter);
                                }
                            })}
                        >
                            {filter.label()}
                        </button>
                    }}
                </div>
                <span class="text-xs text-demo-muted">
                    {format!("{} results", incidents.len())}
                </span>
            </div>

            <div class="flex-1 min-h-0 flex flex-col overflow-scroll">
                {if incidents.is_empty() {
                    rsx! {
                        <div class="flex-1 flex flex-col items-center justify-center gap-6 p-20 border border-demo-border">
                            <span class="text-base text-zinc-300">{"No matching incidents"}</span>
                            <span class="text-sm text-demo-muted">{"Change the scope or severity filter."}</span>
                            <button
                                id="recover-queue"
                                test_support
                                class="px-12 py-8 rounded-md bg-demo-accent text-zinc-950 cursor-pointer"
                                onClick={rsx_listener!(cx, notify, |view, _, _window, _| {
                                    if view.scope != Scope::Active || view.severity_filter != SeverityFilter::All {
                                        view.reset_filters();
                                    } else {
                                        view.simulate_signal();
                                    }
                                })}
                            >
                                {if view.scope != Scope::Active || view.severity_filter != SeverityFilter::All {
                                    "Reset filters"
                                } else {
                                    "Simulate incident"
                                }}
                            </button>
                        </div>
                    }
                } else {
                    rsx! {
                        <div class="flex flex-col">
                            {for incident in incidents.iter() {
                                <button
                                    id={SharedString::from(format!("incident-{}", incident.id))}
                                    test_support
                                    key={incident.id}
                                    role={Role::Button}
                                    ariaLabel={incident.title.clone()}
                                    ariaSelected={incident.id == view.selected_id}
                                    hoverClass="bg-demo-surface"
                                    class={if incident.id == view.selected_id {
                                        "w-full flex items-center gap-12 px-12 py-10 bg-demo-raised border-l-2 border-demo-accent cursor-pointer"
                                    } else {
                                        "w-full flex items-center gap-12 px-12 py-10 border-b border-demo-border cursor-pointer"
                                    }}
                                    onClick={rsx_listener!(cx, notify, {
                                        let incident_id = incident.id;
                                        move |view, _, _window, _| {
                                            view.selected_id = incident_id;
                                            view.detail_tab = DetailTab::Overview;
                                            view.last_action = format!("Selected incident #{incident_id}");
                                        }
                                    })}
                                >
                                    <div class={incident.severity.dot_class()} />
                                    <div class="flex-1 min-w-0 flex flex-col gap-4">
                                        <span class="text-sm text-demo-text truncate">
                                            {incident.title.clone()}
                                        </span>
                                        <span class="text-xs text-demo-muted">
                                            {format!("#{}  {}  {}", incident.id, incident.service, incident.region)}
                                        </span>
                                    </div>
                                    <div class="w-[92px] flex flex-col items-end gap-4">
                                        <span class={incident.status.text_class()}>
                                            {incident.status.label()}
                                        </span>
                                        <span class="text-xs text-demo-muted">{incident.age_label()}</span>
                                    </div>
                                </button>
                            }}
                        </div>
                    }
                }}
            </div>
        </div>
    }
}
