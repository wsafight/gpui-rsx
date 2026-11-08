mod common;

use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{rsx, rsx_fragment, rsx_listener};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Active,
    Done,
}

impl Filter {
    fn matches(self, item: &WorkItem) -> bool {
        match self {
            Self::All => true,
            Self::Active => !item.done,
            Self::Done => item.done,
        }
    }
}

#[derive(Clone, Copy)]
enum Priority {
    High,
    Medium,
    Low,
}

impl Priority {
    fn label(self) -> &'static str {
        match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        }
    }

    fn badge_class(self) -> &'static str {
        match self {
            Self::High => "px-8 py-4 rounded-md bg-rose-950 text-rose-300 border border-rose-800",
            Self::Medium => {
                "px-8 py-4 rounded-md bg-amber-950 text-amber-300 border border-amber-800"
            }
            Self::Low => "px-8 py-4 rounded-md bg-sky-950 text-sky-300 border border-sky-800",
        }
    }
}

#[derive(Clone)]
struct WorkItem {
    id: usize,
    title: String,
    owner: &'static str,
    area: &'static str,
    priority: Priority,
    done: bool,
}

struct ProjectDashboard {
    items: Vec<WorkItem>,
    filter: Filter,
    selected_id: usize,
    compact: bool,
    next_id: usize,
    last_action: String,
}

impl Default for ProjectDashboard {
    fn default() -> Self {
        Self {
            items: vec![
                WorkItem {
                    id: 101,
                    title: "Validate listener notification contracts".into(),
                    owner: "Mina",
                    area: "Runtime",
                    priority: Priority::High,
                    done: false,
                },
                WorkItem {
                    id: 102,
                    title: "Ship the project dashboard demo".into(),
                    owner: "Noah",
                    area: "Examples",
                    priority: Priority::High,
                    done: false,
                },
                WorkItem {
                    id: 103,
                    title: "Review strict class diagnostics".into(),
                    owner: "Ari",
                    area: "Macros",
                    priority: Priority::Medium,
                    done: true,
                },
                WorkItem {
                    id: 104,
                    title: "Document the 0.11 migration path".into(),
                    owner: "Mina",
                    area: "Docs",
                    priority: Priority::Medium,
                    done: true,
                },
                WorkItem {
                    id: 105,
                    title: "Profile dynamic class parsing".into(),
                    owner: "Noah",
                    area: "Performance",
                    priority: Priority::Low,
                    done: false,
                },
                WorkItem {
                    id: 106,
                    title: "Refresh component integration notes".into(),
                    owner: "Ari",
                    area: "Components",
                    priority: Priority::Low,
                    done: false,
                },
            ],
            filter: Filter::All,
            selected_id: 101,
            compact: false,
            next_id: 107,
            last_action: "Workspace loaded".into(),
        }
    }
}

impl ProjectDashboard {
    fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
        self.reconcile_selection();
    }

    fn reconcile_selection(&mut self) {
        if !self
            .items
            .iter()
            .any(|item| item.id == self.selected_id && self.filter.matches(item))
        {
            self.selected_id = self
                .items
                .iter()
                .find(|item| self.filter.matches(item))
                .map_or(0, |item| item.id);
        }
    }

    fn toggle_item(&mut self, item_id: usize) {
        if let Some(item) = self.items.iter_mut().find(|item| item.id == item_id) {
            item.done = !item.done;
            self.last_action = if item.done {
                format!("Completed work item #{item_id}")
            } else {
                format!("Reopened work item #{item_id}")
            };
        }
        self.reconcile_selection();
    }
}

impl Render for ProjectDashboard {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let total = self.items.len();
        let completed = self.items.iter().filter(|item| item.done).count();
        let active = total - completed;
        let completion = completed * 100 / total.max(1);
        let visible_items = self
            .items
            .iter()
            .filter(|item| self.filter.matches(item))
            .collect::<Vec<_>>();
        let selected = self
            .items
            .iter()
            .find(|item| item.id == self.selected_id && self.filter.matches(item));
        let row_spacing = if self.compact { "py-6" } else { "py-10" };

        rsx! {
            <div class="size-full flex bg-demo-canvas text-demo-text">
                <aside class="w-[220px] flex flex-col justify-between p-16 bg-demo-surface border-r border-demo-border">
                    <div class="flex flex-col gap-20">
                        <header class="flex flex-col gap-4">
                            <span class="text-xs font-bold text-emerald-400">
                                {"GPUI-RSX demo"}
                            </span>
                            <h1 class="text-xl font-bold text-white">
                                {"Release desk"}
                            </h1>
                            <span class="text-sm text-demo-muted">
                                {"Local project workspace"}
                            </span>
                        </header>
                        <nav class="flex flex-col gap-6">
                            <button class={if self.filter == Filter::All {
                                "flex items-center justify-between px-10 py-8 rounded-md bg-neutral-700 text-white cursor-pointer"
                            } else {
                                "flex items-center justify-between px-10 py-8 rounded-md text-demo-muted cursor-pointer"
                            }} onClick={cx.listener(|view, _, _window, cx| {
                                view.set_filter(Filter::All);
                                view.last_action = "Showing all work".into();
                                cx.notify();
                            })}>
                                <span>
                                    {"All work"}
                                </span>
                                <span class="text-xs text-demo-muted">
                                    {total.to_string()}
                                </span>
                            </button>
                            <button class={if self.filter == Filter::Active {
                                "flex items-center justify-between px-10 py-8 rounded-md bg-neutral-700 text-white cursor-pointer"
                            } else {
                                "flex items-center justify-between px-10 py-8 rounded-md text-demo-muted cursor-pointer"
                            }} onClick={cx.listener(|view, _, _window, cx| {
                                view.set_filter(Filter::Active);
                                view.last_action = "Showing active work".into();
                                cx.notify();
                            })}>
                                <span>
                                    {"Active"}
                                </span>
                                <span class="text-xs text-demo-muted">
                                    {active.to_string()}
                                </span>
                            </button>
                            <button class={if self.filter == Filter::Done {
                                "flex items-center justify-between px-10 py-8 rounded-md bg-neutral-700 text-white cursor-pointer"
                            } else {
                                "flex items-center justify-between px-10 py-8 rounded-md text-demo-muted cursor-pointer"
                            }} onClick={cx.listener(|view, _, _window, cx| {
                                view.set_filter(Filter::Done);
                                view.last_action = "Showing completed work".into();
                                cx.notify();
                            })}>
                                <span>
                                    {"Completed"}
                                </span>
                                <span class="text-xs text-demo-muted">
                                    {completed.to_string()}
                                </span>
                            </button>
                        </nav>
                    </div>
                    <div class="flex flex-col gap-6 pt-12 border-t border-demo-border">
                        <span class="text-xs text-demo-muted">
                            {"Last action"}
                        </span>
                        <span class="text-sm text-neutral-300">
                            {self.last_action.clone()}
                        </span>
                    </div>
                </aside>
                <main class="flex-1 min-w-0 flex flex-col">
                    <header class="flex items-center justify-between px-20 py-16 border-b border-demo-border">
                        <div class="flex flex-col gap-4">
                            <h2 class="text-2xl font-bold text-white">
                                {"Project dashboard"}
                            </h2>
                            <p class="text-sm text-demo-muted">
                                {"Review work, update its status, or add a follow-up item."}
                            </p>
                        </div>
                        <div class="flex gap-8">
                            <button class="px-12 py-8 rounded-md bg-demo-raised text-neutral-200 border border-neutral-700 cursor-pointer" onClick={cx.listener(|view, _, _window, cx| {
                                view.compact = !view.compact;
                                view.last_action = if view.compact {
                                    "Compact density enabled".into()
                                } else {
                                    "Comfortable density enabled".into()
                                };
                                cx.notify();
                            })}>
                                {if self.compact { "Comfortable" } else { "Compact" }}
                            </button>
                            <button class="px-12 py-8 rounded-md bg-emerald-500 text-neutral-950 font-bold cursor-pointer" onClick={cx.listener(|view, _, _window, cx| {
                                let id = view.next_id;
                                view.next_id += 1;
                                view.items.push(WorkItem {
                                    id,
                                    title: format!("Follow-up work item #{id}"),
                                    owner: "Mina",
                                    area: "Planning",
                                    priority: Priority::Medium,
                                    done: false,
                                });
                                view.selected_id = id;
                                view.filter = Filter::All;
                                view.last_action = format!("Added work item #{id}");
                                cx.notify();
                            })}>
                                {"Add work item"}
                            </button>
                        </div>
                    </header>
                    <section class="flex gap-12 px-20 py-16 border-b border-demo-border">
                        <div class="flex-1 flex flex-col gap-4 p-12 rounded-lg bg-demo-surface border border-demo-border">
                            <span class="text-xs text-demo-muted">
                                {"Total"}
                            </span>
                            <span class="text-2xl font-bold text-white">
                                {total.to_string()}
                            </span>
                        </div>
                        <div class="flex-1 flex flex-col gap-4 p-12 rounded-lg bg-demo-surface border border-demo-border">
                            <span class="text-xs text-demo-muted">
                                {"Active"}
                            </span>
                            <span class="text-2xl font-bold text-amber-300">
                                {active.to_string()}
                            </span>
                        </div>
                        <div class="flex-1 flex flex-col gap-6 p-12 rounded-lg bg-demo-surface border border-demo-border">
                            <div class="flex items-center justify-between">
                                <span class="text-xs text-demo-muted">
                                    {"Completion"}
                                </span>
                                <span class="text-sm text-emerald-300">
                                    {format!("{completion}%")}
                                </span>
                            </div>
                            <div class="h-[6px] rounded-full bg-demo-raised overflow-hidden">
                                <div class={format!("h-full rounded-full bg-emerald-500 w-[{completion}%]")} />
                            </div>
                        </div>
                    </section>
                    <section class="flex flex-1 min-h-0">
                        <div class="flex-1 min-w-0 flex flex-col px-20 py-16">
                            <div class="flex items-center justify-between pb-10">
                                <h3 class="text-base font-bold text-white">
                                    {"Work queue"}
                                </h3>
                                <span class="text-xs text-demo-muted">
                                    {format!("{} visible", visible_items.len())}
                                </span>
                            </div>
                            <div class="flex-1 min-h-0 flex flex-col overflow-scroll">
                                {for item in visible_items.iter() {
                                    <button key={item.id} class={if item.id == self.selected_id {
                                        format!("flex items-center gap-12 px-12 {row_spacing} bg-demo-raised border-l-2 border-emerald-500 cursor-pointer")
                                    } else {
                                        format!("flex items-center gap-12 px-12 {row_spacing} border-b border-demo-border cursor-pointer")
                                    }} onClick={cx.listener({
                                        let item_id = item.id;
                                        move |view, _, _window, cx| {
                                            view.selected_id = item_id;
                                            view.last_action = format!("Selected work item #{item_id}");
                                            cx.notify();
                                        }
                                    })}>
                                        <div class={if item.done {
                                            "size-8 rounded-full bg-emerald-500"
                                        } else {
                                            "size-8 rounded-full border-2 border-neutral-600"
                                        }} />
                                        <div class="flex-1 min-w-0 flex flex-col gap-3">
                                            <span class={if item.done {
                                                "text-sm text-demo-muted line-through truncate"
                                            } else {
                                                "text-sm text-demo-text truncate"
                                            }}>
                                                {item.title.clone()}
                                            </span>
                                            <span class="text-xs text-demo-muted">
                                                {format!("#{}  {}  {}", item.id, item.area, item.owner)}
                                            </span>
                                        </div>
                                        <span class={item.priority.badge_class()}>
                                            {item.priority.label()}
                                        </span>
                                    </button>
                                }}
                            </div>
                        </div>
                        <aside class="w-[300px] flex flex-col justify-between p-20 bg-demo-surface border-l border-demo-border">
                            {...if let Some(selected) = selected {
                                rsx_fragment! {
                                    <>
                                        <div class="flex flex-col gap-20">
                                            <header class="flex flex-col gap-8">
                                                <div class="flex items-center justify-between">
                                                    <span class="text-xs text-demo-muted">
                                                        {format!("Work item #{}", selected.id)}
                                                    </span>
                                                    <span class={selected.priority.badge_class()}>
                                                        {selected.priority.label()}
                                                    </span>
                                                </div>
                                                <h3 class="text-xl font-bold text-white">{selected.title.clone()}</h3>
                                            </header>

                                            <div class="flex flex-col gap-10">
                                                <div class="flex items-center justify-between py-8 border-b border-demo-border">
                                                    <span class="text-sm text-demo-muted">{"Status"}</span>
                                                    <span class={if selected.done {
                                                        "text-sm text-emerald-300"
                                                    } else {
                                                        "text-sm text-amber-300"
                                                    }}>
                                                        {if selected.done { "Completed" } else { "In progress" }}
                                                    </span>
                                                </div>
                                                <div class="flex items-center justify-between py-8 border-b border-demo-border">
                                                    <span class="text-sm text-demo-muted">{"Owner"}</span>
                                                    <span class="text-sm text-neutral-200">{selected.owner}</span>
                                                </div>
                                                <div class="flex items-center justify-between py-8 border-b border-demo-border">
                                                    <span class="text-sm text-demo-muted">{"Area"}</span>
                                                    <span class="text-sm text-neutral-200">{selected.area}</span>
                                                </div>
                                            </div>
                                        </div>

                                        <button
                                            class={if selected.done {
                                                "w-full px-12 py-10 rounded-md bg-demo-raised text-neutral-200 border border-neutral-700 cursor-pointer"
                                            } else {
                                                "w-full px-12 py-10 rounded-md bg-emerald-500 text-neutral-950 font-bold cursor-pointer"
                                            }}
                                            onClick={rsx_listener!(cx, notify, {
                                                let selected_id = selected.id;
                                                move |view, _, _window, _cx| {
                                                    view.toggle_item(selected_id);
                                                }
                                            })}
                                        >
                                            {if selected.done { "Reopen item" } else { "Mark complete" }}
                                        </button>
                                    </>
                                }
                            } else {
                                rsx_fragment! {
                                    <>
                                        <div class="flex flex-col gap-12">
                                            <h3 class="text-lg font-semibold">{"No work in this view"}</h3>
                                            <p class="text-sm text-demo-muted">{"Show all work or add an item to continue."}</p>
                                            <button class="px-12 py-10 rounded-md bg-demo-raised cursor-pointer"
                                                onClick={rsx_listener!(cx, notify, |view, _, _, _| view.set_filter(Filter::All))}>
                                                {"Show all work"}
                                            </button>
                                        </div>
                                    </>
                                }
                            }}
                        </aside>
                    </section>
                </main>
            </div>
        }
    }
}

fn main() {
    common::run(
        |cx| WindowOptions {
            window_min_size: Some(size(px(1024.0), px(600.0))),
            ..common::window(cx, "Project dashboard · GPUI-RSX", 1180.0, 760.0)
        },
        |_| ProjectDashboard::default(),
    );
}

#[cfg(test)]
mod tests {
    use super::{Filter, ProjectDashboard};

    #[test]
    fn selection_follows_the_visible_filter() {
        let mut dashboard = ProjectDashboard::default();
        dashboard.set_filter(Filter::Done);
        let selected = dashboard
            .items
            .iter()
            .find(|item| item.id == dashboard.selected_id)
            .unwrap();
        assert!(selected.done);
        dashboard.toggle_item(dashboard.selected_id);
        assert!(
            dashboard
                .items
                .iter()
                .any(|item| item.id == dashboard.selected_id && item.done)
        );
    }

    #[test]
    fn empty_filter_recovers_when_showing_all_work() {
        let mut dashboard = ProjectDashboard::default();
        for item in &mut dashboard.items {
            item.done = true;
        }
        dashboard.set_filter(Filter::Active);
        assert_eq!(dashboard.selected_id, 0);
        dashboard.set_filter(Filter::All);
        assert_eq!(dashboard.selected_id, 101);
    }
}
