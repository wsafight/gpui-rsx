use gpui::prelude::*;
use gpui::*;
use gpui_rsx::rsx;

use crate::domain::{DetailTab, Incident, SeverityFilter};
use crate::model::IncidentConsole;

pub(crate) fn render(
    view: &IncidentConsole,
    incidents: &[Incident],
    cx: &mut Context<IncidentConsole>,
) -> impl IntoElement + use<> {
    rsx! {
        <div class="flex-1 min-w-0 flex flex-col px-20 py-14">
            <div class="flex items-center justify-between pb-10">
                <div class="flex gap-6">
                    {for filter in SeverityFilter::OPTIONS.iter() {
                        <button
                            key={filter.label()}
                            class={if view.severity_filter == *filter {
                                filter.selected_class()
                            } else {
                                "px-10 py-6 rounded-md text-zinc-500 cursor-pointer"
                            }}
                            onClick={cx.listener({
                                let filter = *filter;
                                move |view, _, _window, cx| {
                                    view.severity_filter = filter;
                                    cx.notify();
                                }
                            })}
                        >
                            {filter.label()}
                        </button>
                    }}
                </div>
                <span class="text-xs text-zinc-500">
                    {format!("{} results", incidents.len())}
                </span>
            </div>

            <div class="flex flex-col overflow-scroll">
                {if incidents.is_empty() {
                    rsx! {
                        <div class="flex-1 flex flex-col items-center justify-center gap-6 p-20 border border-zinc-800">
                            <span class="text-base text-zinc-300">{"No matching incidents"}</span>
                            <span class="text-sm text-zinc-600">{"Change the scope or severity filter."}</span>
                        </div>
                    }
                } else {
                    rsx! {
                        <div class="flex flex-col">
                            {for incident in incidents.iter() {
                                <button
                                    key={incident.id}
                                    class={if incident.id == view.selected_id {
                                        "w-full flex items-center gap-12 px-12 py-10 bg-zinc-800 border-l-2 border-cyan-400 cursor-pointer"
                                    } else {
                                        "w-full flex items-center gap-12 px-12 py-10 border-b border-zinc-800 cursor-pointer"
                                    }}
                                    onClick={cx.listener({
                                        let incident_id = incident.id;
                                        move |view, _, _window, cx| {
                                            view.selected_id = incident_id;
                                            view.detail_tab = DetailTab::Overview;
                                            view.last_action = format!("Selected incident #{incident_id}");
                                            cx.notify();
                                        }
                                    })}
                                >
                                    <div class={incident.severity.dot_class()} />
                                    <div class="flex-1 min-w-0 flex flex-col gap-4">
                                        <span class="text-sm text-zinc-100 truncate">
                                            {incident.title.clone()}
                                        </span>
                                        <span class="text-xs text-zinc-500">
                                            {format!("#{}  {}  {}", incident.id, incident.service, incident.region)}
                                        </span>
                                    </div>
                                    <div class="w-[92px] flex flex-col items-end gap-4">
                                        <span class={incident.status.text_class()}>
                                            {incident.status.label()}
                                        </span>
                                        <span class="text-xs text-zinc-500">{incident.age_label()}</span>
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
