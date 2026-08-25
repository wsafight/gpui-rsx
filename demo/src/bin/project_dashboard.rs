use gpui::prelude::*;
use gpui::*;
use gpui_platform::application;
use gpui_rsx::rsx;

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
                    title: "Validate GPUI 0.2 event contracts".into(),
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
                    title: "Document the Rust 1.88 MSRV".into(),
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
            .cloned()
            .collect::<Vec<_>>();
        let selected = self
            .items
            .iter()
            .find(|item| item.id == self.selected_id)
            .cloned()
            .unwrap_or_else(|| self.items[0].clone());
        let row_spacing = if self.compact { "py-6" } else { "py-10" };

        rsx! {
            <div class="size-full flex bg-neutral-950 text-neutral-100">
                <aside class="w-[220px] flex flex-col justify-between p-16 bg-neutral-900 border-r border-neutral-800">
                    <div class="flex flex-col gap-20">
                        <header class="flex flex-col gap-4">
                            <span class="text-xs font-bold text-emerald-400">{"GPUI RSX"}</span>
                            <h1 class="text-xl font-bold text-white">{"Release desk"}</h1>
                            <span class="text-sm text-neutral-500">{"v0.7 workspace"}</span>
                        </header>

                        <nav class="flex flex-col gap-6">
                            <button
                                class={if self.filter == Filter::All {
                                    "flex items-center justify-between px-10 py-8 rounded-md bg-neutral-700 text-white cursor-pointer"
                                } else {
                                    "flex items-center justify-between px-10 py-8 rounded-md text-neutral-400 cursor-pointer"
                                }}
                                onClick={cx.listener(|view, _, _window, cx| {
                                    view.filter = Filter::All;
                                    view.last_action = "Showing all work".into();
                                    cx.notify();
                                })}
                            >
                                <span>{"All work"}</span>
                                <span class="text-xs text-neutral-500">{total.to_string()}</span>
                            </button>
                            <button
                                class={if self.filter == Filter::Active {
                                    "flex items-center justify-between px-10 py-8 rounded-md bg-neutral-700 text-white cursor-pointer"
                                } else {
                                    "flex items-center justify-between px-10 py-8 rounded-md text-neutral-400 cursor-pointer"
                                }}
                                onClick={cx.listener(|view, _, _window, cx| {
                                    view.filter = Filter::Active;
                                    view.last_action = "Showing active work".into();
                                    cx.notify();
                                })}
                            >
                                <span>{"Active"}</span>
                                <span class="text-xs text-neutral-500">{active.to_string()}</span>
                            </button>
                            <button
                                class={if self.filter == Filter::Done {
                                    "flex items-center justify-between px-10 py-8 rounded-md bg-neutral-700 text-white cursor-pointer"
                                } else {
                                    "flex items-center justify-between px-10 py-8 rounded-md text-neutral-400 cursor-pointer"
                                }}
                                onClick={cx.listener(|view, _, _window, cx| {
                                    view.filter = Filter::Done;
                                    view.last_action = "Showing completed work".into();
                                    cx.notify();
                                })}
                            >
                                <span>{"Completed"}</span>
                                <span class="text-xs text-neutral-500">{completed.to_string()}</span>
                            </button>
                        </nav>
                    </div>

                    <div class="flex flex-col gap-6 pt-12 border-t border-neutral-800">
                        <span class="text-xs text-neutral-500">{"LAST ACTION"}</span>
                        <span class="text-sm text-neutral-300">{self.last_action.clone()}</span>
                    </div>
                </aside>

                <main class="flex-1 min-w-0 flex flex-col">
                    <header class="flex items-center justify-between px-20 py-16 border-b border-neutral-800">
                        <div class="flex flex-col gap-4">
                            <h2 class="text-2xl font-bold text-white">{"Project dashboard"}</h2>
                            <p class="text-sm text-neutral-400">
                                {"Interactive filters, keyed rows, conditional classes, and local state."}
                            </p>
                        </div>
                        <div class="flex gap-8">
                            <button
                                class="px-12 py-8 rounded-md bg-neutral-800 text-neutral-200 border border-neutral-700 cursor-pointer"
                                onClick={cx.listener(|view, _, _window, cx| {
                                    view.compact = !view.compact;
                                    view.last_action = if view.compact {
                                        "Compact density enabled".into()
                                    } else {
                                        "Comfortable density enabled".into()
                                    };
                                    cx.notify();
                                })}
                            >
                                {if self.compact { "Comfortable" } else { "Compact" }}
                            </button>
                            <button
                                class="px-12 py-8 rounded-md bg-emerald-500 text-neutral-950 font-bold cursor-pointer"
                                onClick={cx.listener(|view, _, _window, cx| {
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
                                })}
                            >
                                {"Add work item"}
                            </button>
                        </div>
                    </header>

                    <section class="flex gap-12 px-20 py-16 border-b border-neutral-800">
                        <div class="flex-1 flex flex-col gap-4 p-12 rounded-lg bg-neutral-900 border border-neutral-800">
                            <span class="text-xs text-neutral-500">{"TOTAL"}</span>
                            <span class="text-2xl font-bold text-white">{total.to_string()}</span>
                        </div>
                        <div class="flex-1 flex flex-col gap-4 p-12 rounded-lg bg-neutral-900 border border-neutral-800">
                            <span class="text-xs text-neutral-500">{"ACTIVE"}</span>
                            <span class="text-2xl font-bold text-amber-300">{active.to_string()}</span>
                        </div>
                        <div class="flex-1 flex flex-col gap-6 p-12 rounded-lg bg-neutral-900 border border-neutral-800">
                            <div class="flex items-center justify-between">
                                <span class="text-xs text-neutral-500">{"COMPLETION"}</span>
                                <span class="text-sm text-emerald-300">{format!("{completion}%")}</span>
                            </div>
                            <div class="h-[6px] rounded-full bg-neutral-800 overflow-hidden">
                                <div class={format!("h-full rounded-full bg-emerald-500 w-[{completion}%]")} />
                            </div>
                        </div>
                    </section>

                    <section class="flex flex-1 min-h-0">
                        <div class="flex-1 min-w-0 flex flex-col px-20 py-16">
                            <div class="flex items-center justify-between pb-10">
                                <h3 class="text-base font-bold text-white">{"Work queue"}</h3>
                                <span class="text-xs text-neutral-500">
                                    {format!("{} visible", visible_items.len())}
                                </span>
                            </div>

                            <div class="flex flex-col overflow-scroll">
                                {for item in visible_items.iter() {
                                    <button
                                        key={item.id}
                                        class={if item.id == self.selected_id {
                                            format!("flex items-center gap-12 px-12 {row_spacing} bg-neutral-800 border-l-2 border-emerald-500 cursor-pointer")
                                        } else {
                                            format!("flex items-center gap-12 px-12 {row_spacing} border-b border-neutral-800 cursor-pointer")
                                        }}
                                        onClick={cx.listener({
                                            let item_id = item.id;
                                            move |view, _, _window, cx| {
                                                view.selected_id = item_id;
                                                view.last_action = format!("Selected work item #{item_id}");
                                                cx.notify();
                                            }
                                        })}
                                    >
                                        <div
                                            class={if item.done {
                                                "size-8 rounded-full bg-emerald-500"
                                            } else {
                                                "size-8 rounded-full border-2 border-neutral-600"
                                            }}
                                        />
                                        <div class="flex-1 min-w-0 flex flex-col gap-3">
                                            <span
                                                class={if item.done {
                                                    "text-sm text-neutral-500 line-through truncate"
                                                } else {
                                                    "text-sm text-neutral-100 truncate"
                                                }}
                                            >
                                                {item.title.clone()}
                                            </span>
                                            <span class="text-xs text-neutral-500">
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

                        <aside class="w-[300px] flex flex-col justify-between p-20 bg-neutral-900 border-l border-neutral-800">
                            <div class="flex flex-col gap-20">
                                <header class="flex flex-col gap-8">
                                    <div class="flex items-center justify-between">
                                        <span class="text-xs text-neutral-500">
                                            {format!("WORK ITEM #{}", selected.id)}
                                        </span>
                                        <span class={selected.priority.badge_class()}>
                                            {selected.priority.label()}
                                        </span>
                                    </div>
                                    <h3 class="text-xl font-bold text-white">{selected.title.clone()}</h3>
                                </header>

                                <div class="flex flex-col gap-10">
                                    <div class="flex items-center justify-between py-8 border-b border-neutral-800">
                                        <span class="text-sm text-neutral-500">{"Status"}</span>
                                        <span class={if selected.done {
                                            "text-sm text-emerald-300"
                                        } else {
                                            "text-sm text-amber-300"
                                        }}>
                                            {if selected.done { "Completed" } else { "In progress" }}
                                        </span>
                                    </div>
                                    <div class="flex items-center justify-between py-8 border-b border-neutral-800">
                                        <span class="text-sm text-neutral-500">{"Owner"}</span>
                                        <span class="text-sm text-neutral-200">{selected.owner}</span>
                                    </div>
                                    <div class="flex items-center justify-between py-8 border-b border-neutral-800">
                                        <span class="text-sm text-neutral-500">{"Area"}</span>
                                        <span class="text-sm text-neutral-200">{selected.area}</span>
                                    </div>
                                </div>
                            </div>

                            <button
                                class={if selected.done {
                                    "w-full px-12 py-10 rounded-md bg-neutral-800 text-neutral-200 border border-neutral-700 cursor-pointer"
                                } else {
                                    "w-full px-12 py-10 rounded-md bg-emerald-500 text-neutral-950 font-bold cursor-pointer"
                                }}
                                onClick={cx.listener({
                                    let selected_id = selected.id;
                                    move |view, _, _window, cx| {
                                        if let Some(item) = view.items.iter_mut().find(|item| item.id == selected_id) {
                                            item.done = !item.done;
                                            view.last_action = if item.done {
                                                format!("Completed work item #{selected_id}")
                                            } else {
                                                format!("Reopened work item #{selected_id}")
                                            };
                                        }
                                        cx.notify();
                                    }
                                })}
                            >
                                {if selected.done { "Reopen item" } else { "Mark complete" }}
                            </button>
                        </aside>
                    </section>
                </main>
            </div>
        }
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1180.0), px(760.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..WindowOptions::default()
            },
            |_, cx| cx.new(|_| ProjectDashboard::default()),
        )
        .expect("failed to open project dashboard window");
        cx.activate(true);
    });
}
