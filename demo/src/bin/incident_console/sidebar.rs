use gpui::prelude::*;
use gpui::*;
use gpui_rsx::rsx;

use crate::domain::Scope;
use crate::model::{ConsoleStats, IncidentConsole};

pub(crate) fn render(
    view: &IncidentConsole,
    stats: &ConsoleStats,
    cx: &mut Context<IncidentConsole>,
) -> impl IntoElement + use<> {
    let scope_options = [
        (Scope::Active, stats.active),
        (Scope::Unassigned, stats.unassigned),
        (Scope::All, stats.total),
    ];

    rsx! {
        <aside class="w-[212px] min-w-[212px] flex flex-col justify-between p-16 bg-demo-surface border-r border-demo-border">
            <div class="flex flex-col gap-20">
                <header class="flex flex-col gap-4">
                    <span class="text-xs font-bold text-demo-accent">{"GPUI-RSX demo"}</span>
                    <h1 class="text-xl font-bold text-white">{"Operations"}</h1>
                    <span class="text-xs text-demo-muted">{"Production workspace"}</span>
                </header>

                <nav class="flex flex-col gap-6">
                    <span class="text-xs text-demo-muted">{"Incidents"}</span>
                    {for (scope, count) in scope_options.iter() {
                        <button
                            key={scope.label()}
                            class={if view.scope == *scope {
                                "flex items-center justify-between px-10 py-8 rounded-md bg-zinc-700 text-white cursor-pointer"
                            } else {
                                "flex items-center justify-between px-10 py-8 rounded-md text-demo-muted cursor-pointer"
                            }}
                            onClick={cx.listener({
                                let scope = *scope;
                                move |view, _, _window, cx| {
                                    view.set_scope(scope);
                                    cx.notify();
                                }
                            })}
                        >
                            <span>{scope.label()}</span>
                            <span class="text-xs text-demo-muted">{count.to_string()}</span>
                        </button>
                    }}
                </nav>

                <div class="flex flex-col gap-8 pt-12 border-t border-demo-border">
                    <span class="text-xs text-demo-muted">{"Automation"}</span>
                    <button
                        class="flex items-center justify-between px-10 py-8 rounded-md bg-demo-canvas border border-demo-border cursor-pointer"
                        onClick={cx.listener(|view, _, _window, cx| {
                            view.auto_triage = !view.auto_triage;
                            view.last_action = if view.auto_triage {
                                "Auto triage enabled".into()
                            } else {
                                "Auto triage paused".into()
                            };
                            cx.notify();
                        })}
                    >
                        <span class="text-sm text-zinc-300">{"Auto triage"}</span>
                        <div class={if view.auto_triage {
                            "w-[28px] h-[16px] flex items-center justify-end px-2 rounded-full bg-emerald-500"
                        } else {
                            "w-[28px] h-[16px] flex items-center px-2 rounded-full bg-zinc-700"
                        }}>
                            <div class="size-10 rounded-full bg-white" />
                        </div>
                    </button>
                </div>
            </div>

            <div class="flex flex-col gap-10 pt-12 border-t border-demo-border">
                <div class="flex flex-col gap-4">
                    <span class="text-xs text-demo-muted">{"Last action"}</span>
                    <span class="text-sm text-zinc-300">{view.last_action.clone()}</span>
                </div>
                <button
                    class="px-10 py-8 rounded-md text-sm text-demo-muted border border-zinc-700 cursor-pointer"
                    onClick={cx.listener(|view, _, _window, cx| {
                        view.clear_resolved();
                        cx.notify();
                    })}
                >
                    {format!("Clear resolved ({})", stats.resolved)}
                </button>
            </div>
        </aside>
    }
}
