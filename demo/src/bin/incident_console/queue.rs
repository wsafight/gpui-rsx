use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{rsx, rsx_listener};

use crate::domain::{DetailTab, Incident, Scope, SeverityFilter};
use crate::model::IncidentConsole;

pub(crate) fn render(
    view: &IncidentConsole,
    cx: &mut Context<IncidentConsole>,
) -> impl IntoElement + use<> {
    let count = view.snapshot().visible_indices.len();
    let list = uniform_list(
        "incident-queue",
        count,
        cx.processor(|view, range: std::ops::Range<usize>, _, cx| {
            let snapshot = view.snapshot();
            view.rendered_rows
                .set(view.rendered_rows.get() + range.len());
            range
                .filter_map(|index| {
                    let position = *snapshot.visible_indices.get(index)?;
                    let incident = view.incidents().get(position)?;
                    Some(render_row(view, incident, cx))
                })
                .collect()
        }),
    )
    .track_scroll(&view.queue_scroll)
    .size_full();
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
                                match filter {
                                    SeverityFilter::All => "px-10 py-6 rounded-md bg-zinc-700 text-white cursor-pointer",
                                    SeverityFilter::Critical => "px-10 py-6 rounded-md bg-rose-950 text-rose-300 cursor-pointer",
                                    SeverityFilter::High => "px-10 py-6 rounded-md bg-amber-950 text-amber-300 cursor-pointer",
                                    SeverityFilter::Medium => "px-10 py-6 rounded-md bg-sky-950 text-sky-300 cursor-pointer",
                                }
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
                    {format!("{count} results")}
                </span>
            </div>

            <div class="flex-1 min-h-0 flex flex-col overflow-hidden">
                {if count == 0 {
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
                    }.into_any_element()
                } else {
                    list.into_any_element()
                }}
            </div>
        </div>
    }
}

fn render_row(
    view: &IncidentConsole,
    incident: &Incident,
    cx: &mut Context<IncidentConsole>,
) -> impl IntoElement + use<> {
    rsx! {
        <button
            id={SharedString::from(format!("incident-{}", incident.id))}
            test_support
            key={incident.id}
            role={Role::Button}
            ariaLabel={incident.title.clone()}
            ariaSelected={incident.id == view.selected_id}
            hoverClass="bg-demo-surface"
            class={if incident.id == view.selected_id {
                "w-full h-72 flex items-center gap-12 px-12 py-10 bg-demo-raised border-l-2 border-demo-accent cursor-pointer"
            } else {
                "w-full h-72 flex items-center gap-12 px-12 py-10 border-b border-demo-border cursor-pointer"
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
    }
}
