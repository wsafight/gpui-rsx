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
        <aside class="w-[212px] min-w-[212px] flex flex-col justify-between p-16 bg-zinc-900 border-r border-zinc-800">
            <div class="flex flex-col gap-20">
                <header class="flex flex-col gap-4">
                    <span class="text-xs font-bold text-cyan-400">{"RELAY"}</span>
                    <h1 class="text-xl font-bold text-white">{"Operations"}</h1>
                    <span class="text-xs text-zinc-500">{"Production workspace"}</span>
                </header>

                <nav class="flex flex-col gap-6">
                    <span class="text-xs text-zinc-500">{"INCIDENTS"}</span>
                    {for (scope, count) in scope_options.iter() {
                        <button
                            key={scope.label()}
                            class={if view.scope == *scope {
                                "flex items-center justify-between px-10 py-8 rounded-md bg-zinc-700 text-white cursor-pointer"
                            } else {
                                "flex items-center justify-between px-10 py-8 rounded-md text-zinc-400 cursor-pointer"
                            }}
                            onClick={cx.listener({
                                let scope = *scope;
                                move |view, _, _window, cx| {
                                    view.scope = scope;
                                    view.last_action = scope.action().into();
                                    cx.notify();
                                }
                            })}
                        >
                            <span>{scope.label()}</span>
                            <span class="text-xs text-zinc-500">{count.to_string()}</span>
                        </button>
                    }}
                </nav>

                <div class="flex flex-col gap-8 pt-12 border-t border-zinc-800">
                    <span class="text-xs text-zinc-500">{"AUTOMATION"}</span>
                    <button
                        class="flex items-center justify-between px-10 py-8 rounded-md bg-zinc-950 border border-zinc-800 cursor-pointer"
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

            <div class="flex flex-col gap-10 pt-12 border-t border-zinc-800">
                <div class="flex flex-col gap-4">
                    <span class="text-xs text-zinc-500">{"LAST ACTION"}</span>
                    <span class="text-sm text-zinc-300">{view.last_action.clone()}</span>
                </div>
                <button
                    class="px-10 py-8 rounded-md text-sm text-zinc-400 border border-zinc-700 cursor-pointer"
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
