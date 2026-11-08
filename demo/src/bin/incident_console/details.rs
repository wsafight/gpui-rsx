use gpui::prelude::*;
use gpui::*;
use gpui_rsx::rsx;

use crate::domain::{DetailTab, Incident, IncidentStatus};
use crate::model::IncidentConsole;

pub(crate) fn render(
    view: &IncidentConsole,
    selected: Option<&Incident>,
    cx: &mut Context<IncidentConsole>,
) -> impl IntoElement + use<> {
    rsx! {
        <aside class="w-[356px] min-w-[356px] flex flex-col bg-demo-surface border-l border-demo-border">
            {if let Some(selected) = selected {
                rsx! {
                    <div class="size-full flex flex-col">
                        <header class="flex flex-col gap-10 p-18 border-b border-demo-border">
                            <div class="flex items-center justify-between">
                                <span class="text-xs text-demo-muted">
                                    {format!("INCIDENT #{}", selected.id)}
                                </span>
                                <span class={selected.severity.badge_class()}>
                                    {selected.severity.label()}
                                </span>
                            </div>
                            <h3 class="text-xl font-bold text-white">{selected.title.clone()}</h3>
                            <div class="flex items-center justify-between">
                                <span class={selected.status.text_class()}>{selected.status.label()}</span>
                                <span class="text-xs text-demo-muted">{selected.age_label()}</span>
                            </div>
                        </header>

                        <div class="flex px-18 pt-12 border-b border-demo-border">
                            <button
                                class={if view.detail_tab == DetailTab::Overview {
                                    "flex-1 px-10 py-8 text-sm text-white border-b-2 border-demo-accent cursor-pointer"
                                } else {
                                    "flex-1 px-10 py-8 text-sm text-demo-muted cursor-pointer"
                                }}
                                onClick={cx.listener(|view, _, _window, cx| {
                                    view.detail_tab = DetailTab::Overview;
                                    cx.notify();
                                })}
                            >
                                {"Overview"}
                            </button>
                            <button
                                class={if view.detail_tab == DetailTab::Timeline {
                                    "flex-1 px-10 py-8 text-sm text-white border-b-2 border-demo-accent cursor-pointer"
                                } else {
                                    "flex-1 px-10 py-8 text-sm text-demo-muted cursor-pointer"
                                }}
                                onClick={cx.listener(|view, _, _window, cx| {
                                    view.detail_tab = DetailTab::Timeline;
                                    cx.notify();
                                })}
                            >
                                {format!("Timeline ({})", selected.events.len())}
                            </button>
                        </div>

                        {if view.detail_tab == DetailTab::Overview {
                            rsx! {
                                <div class="flex-1 min-h-0 flex flex-col justify-between p-18 overflow-scroll">
                                    <div class="flex flex-col gap-16">
                                        <div class="flex flex-col gap-8">
                                            <div class="flex items-center justify-between py-6 border-b border-demo-border">
                                                <span class="text-sm text-demo-muted">{"Service"}</span>
                                                <span class="text-sm text-zinc-200">{selected.service}</span>
                                            </div>
                                            <div class="flex items-center justify-between py-6 border-b border-demo-border">
                                                <span class="text-sm text-demo-muted">{"Region"}</span>
                                                <span class="text-sm text-zinc-200">{selected.region}</span>
                                            </div>
                                            <div class="flex items-center justify-between py-6 border-b border-demo-border">
                                                <span class="text-sm text-demo-muted">{"Owner"}</span>
                                                <span class="text-sm text-zinc-200">
                                                    {selected.owner.unwrap_or("Unassigned")}
                                                </span>
                                            </div>
                                        </div>

                                        <div class="flex gap-8">
                                            <div class="flex-1 flex flex-col gap-4 p-10 rounded-md bg-demo-canvas border border-demo-border">
                                                <span class="text-xs text-demo-muted">{"Customers"}</span>
                                                <span class="text-lg font-bold text-white">
                                                    {selected.customers.to_string()}
                                                </span>
                                            </div>
                                            <div class="flex-1 flex flex-col gap-4 p-10 rounded-md bg-demo-canvas border border-demo-border">
                                                <span class="text-xs text-demo-muted">{"Latency"}</span>
                                                <span class="text-lg font-bold text-amber-300">
                                                    {format!("{} ms", selected.latency_ms)}
                                                </span>
                                            </div>
                                            <div class="flex-1 flex flex-col gap-4 p-10 rounded-md bg-demo-canvas border border-demo-border">
                                                <span class="text-xs text-demo-muted">{"Errors"}</span>
                                                <span class="text-lg font-bold text-rose-300">
                                                    {format!("{:.1}%", selected.error_rate)}
                                                </span>
                                            </div>
                                        </div>

                                        <div class="flex flex-col gap-8">
                                            <span class="text-xs text-demo-muted">{"Response progress"}</span>
                                            <div class="flex items-center gap-6">
                                                <div class="flex-1 h-[4px] rounded-full bg-rose-500" />
                                                <div class={if selected.status != IncidentStatus::Triggered {
                                                    "flex-1 h-[4px] rounded-full bg-amber-400"
                                                } else {
                                                    "flex-1 h-[4px] rounded-full bg-zinc-700"
                                                }} />
                                                <div class={if matches!(selected.status, IncidentStatus::Monitoring | IncidentStatus::Resolved) {
                                                    "flex-1 h-[4px] rounded-full bg-sky-400"
                                                } else {
                                                    "flex-1 h-[4px] rounded-full bg-zinc-700"
                                                }} />
                                                <div class={if selected.status == IncidentStatus::Resolved {
                                                    "flex-1 h-[4px] rounded-full bg-emerald-400"
                                                } else {
                                                    "flex-1 h-[4px] rounded-full bg-zinc-700"
                                                }} />
                                            </div>
                                        </div>
                                    </div>

                                    <div class="flex flex-col gap-8 pt-16 border-t border-demo-border">
                                        <div class="flex gap-8">
                                            <button
                                                class="flex-1 px-10 py-8 rounded-md text-sm text-zinc-300 border border-zinc-700 cursor-pointer"
                                                onClick={cx.listener(|view, _, _window, cx| {
                                                    view.assign_selected();
                                                    cx.notify();
                                                })}
                                            >
                                                {"Assign to me"}
                                            </button>
                                            <button
                                                class="flex-1 px-10 py-8 rounded-md text-sm text-rose-300 border border-rose-900 cursor-pointer"
                                                onClick={cx.listener(|view, _, _window, cx| {
                                                    view.escalate_selected();
                                                    cx.notify();
                                                })}
                                            >
                                                {"Escalate"}
                                            </button>
                                        </div>
                                        <button
                                            class={if selected.status == IncidentStatus::Resolved {
                                                "w-full px-12 py-10 rounded-md bg-demo-raised text-zinc-200 border border-zinc-700 cursor-pointer"
                                            } else {
                                                "w-full px-12 py-10 rounded-md bg-demo-accent text-zinc-950 font-bold cursor-pointer"
                                            }}
                                            onClick={cx.listener(|view, _, _window, cx| {
                                                view.advance_selected();
                                                cx.notify();
                                            })}
                                        >
                                            {selected.status.action_label()}
                                        </button>
                                    </div>
                                </div>
                            }
                        } else {
                            rsx! {
                                <div class="flex-1 min-h-0 flex flex-col gap-12 p-18 overflow-scroll">
                                    {for event in selected.events.iter().rev() {
                                        <div key={event.id} class="flex gap-10 pb-12 border-b border-demo-border">
                                            <div class="flex flex-col items-center gap-4 pt-4">
                                                <div class={event.tone.dot_class()} />
                                                <div class="w-[1px] h-[34px] bg-demo-raised" />
                                            </div>
                                            <div class="flex-1 min-w-0 flex flex-col gap-4">
                                                <div class="flex items-center justify-between">
                                                    <span class="text-sm font-bold text-zinc-200">{event.actor}</span>
                                                    <span class="text-xs text-demo-muted">
                                                        {format!("EVT-{:03} / {}", event.id, event.stamp)}
                                                    </span>
                                                </div>
                                                <span class="text-sm text-demo-muted">{event.message.clone()}</span>
                                            </div>
                                        </div>
                                    }}
                                </div>
                            }
                        }}
                    </div>
                }
            } else {
                rsx! {
                    <div class="size-full flex flex-col items-center justify-center gap-6 p-20">
                        <span class="text-base text-zinc-300">{"No incident selected"}</span>
                        <span class="text-sm text-demo-muted">{"Reset the queue filters or simulate an incident."}</span>
                    </div>
                }
            }}
        </aside>
    }
}
