use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{rsx, rsx_listener};

use crate::details;
use crate::model::{ConsoleStats, IncidentConsole};
use crate::queue;
use crate::sidebar;

impl Render for IncidentConsole {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.rendered_rows.set(0);
        #[cfg(feature = "native-profiler")]
        crate::profile::schedule(self, _window, cx);
        let snapshot = self.snapshot();
        let selected = self.selected_incident();
        let sidebar = sidebar::render(self, &snapshot.stats, cx);
        let queue = queue::render(self, cx);
        let details = details::render(self, selected, cx);

        rsx! {
            <div class="size-full flex bg-demo-canvas text-demo-text">
                {sidebar}
                <main class="flex-1 min-w-0 flex flex-col">
                    <header class="flex items-center justify-between px-20 py-14 border-b border-demo-border">
                        <div class="flex flex-col gap-4">
                            <h2 class="text-2xl font-bold text-white">{"Incident command"}</h2>
                            <span class="text-sm text-demo-muted">
                                {format!(
                                    "{} active across {} production services",
                                    snapshot.stats.active,
                                    snapshot.stats.service_count,
                                )}
                            </span>
                        </div>
                        <div class="flex gap-8">
                            {for count in [1_000usize, 10_000, 100_000] {
                                <button
                                    id={SharedString::from(format!("load-{count}"))}
                                    test_support
                                    key={count}
                                    role={Role::Button}
                                    focusable
                                    ariaLabel={format!("Load {count} incidents")}
                                    focusClass="border-demo-accent"
                                    class="px-10 py-8 rounded-md bg-demo-surface text-demo-muted border border-demo-border cursor-pointer"
                                    onClick={rsx_listener!(cx, notify, move |view, _, _, _| {
                                        view.load_rows(count);
                                    })}
                                >
                                    {format!("{}k", count / 1000)}
                                </button>
                            }}
                            <button
                                class="px-12 py-8 rounded-md bg-demo-surface text-zinc-200 border border-zinc-700 cursor-pointer"
                                onClick={rsx_listener!(cx, notify, |view, _, _window, _| {
                                    view.toggle_sort();
                                })}
                            >
                                {self.sort_mode.label()}
                            </button>
                            <button
                                class="px-12 py-8 rounded-md bg-demo-accent text-zinc-950 font-bold cursor-pointer"
                                onClick={rsx_listener!(cx, notify, |view, _, _window, _| {
                                    view.simulate_signal();
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
        <section class="flex gap-10 px-20 py-12 border-b border-demo-border">
            <div class="flex-1 flex items-center justify-between p-10 rounded-md bg-demo-surface border border-demo-border">
                <span class="text-xs text-demo-muted">{"Active"}</span>
                <span class="text-xl font-bold text-white">{stats.active.to_string()}</span>
            </div>
            <div class="flex-1 flex items-center justify-between p-10 rounded-md bg-rose-950 border border-rose-900">
                <span class="text-xs text-rose-400">{"Critical"}</span>
                <span class="text-xl font-bold text-rose-200">{stats.critical.to_string()}</span>
            </div>
            <div class="flex-1 flex items-center justify-between p-10 rounded-md bg-demo-surface border border-demo-border">
                <span class="text-xs text-demo-muted">{"Unassigned"}</span>
                <span class="text-xl font-bold text-amber-300">{stats.unassigned.to_string()}</span>
            </div>
            <div class="flex-1 flex items-center justify-between p-10 rounded-md bg-demo-surface border border-demo-border">
                <span class="text-xs text-demo-muted">{"Customers"}</span>
                <span class="text-xl font-bold text-demo-accent">
                    {stats.affected_customers.to_string()}
                </span>
            </div>
        </section>
    }
}
