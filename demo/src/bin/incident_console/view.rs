use gpui::prelude::*;
use gpui::*;
use gpui_rsx::rsx;

use crate::details;
use crate::model::{ConsoleStats, IncidentConsole};
use crate::queue;
use crate::sidebar;

impl Render for IncidentConsole {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let snapshot = self.snapshot();
        let sidebar = sidebar::render(self, &snapshot.stats, cx);
        let queue = queue::render(self, &snapshot.visible_incidents, cx);
        let details = details::render(self, snapshot.selected.as_ref(), cx);

        rsx! {
            <div class="size-full flex bg-neutral-950 text-neutral-100">
                {sidebar}
                <main class="flex-1 min-w-0 flex flex-col">
                    <header class="flex items-center justify-between px-20 py-14 border-b border-zinc-800">
                        <div class="flex flex-col gap-4">
                            <h2 class="text-2xl font-bold text-white">{"Incident command"}</h2>
                            <span class="text-sm text-zinc-400">
                                {format!(
                                    "{} active across {} production services",
                                    snapshot.stats.active,
                                    snapshot.stats.service_count,
                                )}
                            </span>
                        </div>
                        <div class="flex gap-8">
                            <button
                                class="px-12 py-8 rounded-md bg-zinc-900 text-zinc-200 border border-zinc-700 cursor-pointer"
                                onClick={cx.listener(|view, _, _window, cx| {
                                    view.toggle_sort();
                                    cx.notify();
                                })}
                            >
                                {self.sort_mode.label()}
                            </button>
                            <button
                                class="px-12 py-8 rounded-md bg-cyan-400 text-zinc-950 font-bold cursor-pointer"
                                onClick={cx.listener(|view, _, _window, cx| {
                                    view.simulate_signal();
                                    cx.notify();
                                })}
                            >
                                {"Inject signal"}
                            </button>
                        </div>
                    </header>

                    {render_metrics(&snapshot.stats)}

                    <section class="flex flex-1 min-h-0">
                        {queue}
                        {details}
                    </section>
                </main>
            </div>
        }
    }
}

fn render_metrics(stats: &ConsoleStats) -> impl IntoElement + use<> {
    rsx! {
        <section class="flex gap-10 px-20 py-12 border-b border-zinc-800">
            <div class="flex-1 flex items-center justify-between p-10 rounded-md bg-zinc-900 border border-zinc-800">
                <span class="text-xs text-zinc-500">{"ACTIVE"}</span>
                <span class="text-xl font-bold text-white">{stats.active.to_string()}</span>
            </div>
            <div class="flex-1 flex items-center justify-between p-10 rounded-md bg-rose-950 border border-rose-900">
                <span class="text-xs text-rose-400">{"CRITICAL"}</span>
                <span class="text-xl font-bold text-rose-200">{stats.critical.to_string()}</span>
            </div>
            <div class="flex-1 flex items-center justify-between p-10 rounded-md bg-zinc-900 border border-zinc-800">
                <span class="text-xs text-zinc-500">{"UNASSIGNED"}</span>
                <span class="text-xl font-bold text-amber-300">{stats.unassigned.to_string()}</span>
            </div>
            <div class="flex-1 flex items-center justify-between p-10 rounded-md bg-zinc-900 border border-zinc-800">
                <span class="text-xs text-zinc-500">{"CUSTOMERS"}</span>
                <span class="text-xl font-bold text-cyan-300">
                    {stats.affected_customers.to_string()}
                </span>
            </div>
        </section>
    }
}
