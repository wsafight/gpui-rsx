use gpui::prelude::*;
use gpui::*;
use gpui_platform::application;
use gpui_rsx::rsx;

#[derive(Clone)]
struct Task {
    id: usize,
    title: &'static str,
    owner: &'static str,
    done: bool,
}

struct TaskListView {
    tasks: Vec<Task>,
}

impl Default for TaskListView {
    fn default() -> Self {
        Self {
            tasks: vec![
                Task {
                    id: 1,
                    title: "Sync GPUI dependency",
                    owner: "Core",
                    done: true,
                },
                Task {
                    id: 2,
                    title: "Review generated class paths",
                    owner: "UI",
                    done: true,
                },
                Task {
                    id: 3,
                    title: "Add demos for 0.7",
                    owner: "Docs",
                    done: false,
                },
            ],
        }
    }
}

impl Render for TaskListView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="size-full flex flex-col gap-4 p-6 bg-stone-950">
                <header class="flex flex-col gap-2">
                    <h1 class="text-3xl font-bold text-white">
                        {"Task List"}
                    </h1>
                    <p class="text-sm text-stone-400">
                        {"Loop rendering with key and conditional literal classes."}
                    </p>
                </header>

                <section class="flex flex-col gap-3">
                    {for task in self.tasks.iter() {
                        <div
                            key={task.id}
                            class={if task.done {
                                "flex items-center justify-between gap-4 p-4 rounded-lg bg-emerald-950 border border-emerald-700"
                            } else {
                                "flex items-center justify-between gap-4 p-4 rounded-lg bg-stone-900 border border-stone-700"
                            }}
                        >
                            <div class="flex flex-col gap-1">
                                <span class="text-base text-white">{task.title}</span>
                                <span class="text-sm text-stone-400">
                                    {format!("#{} - Owner: {}", task.id, task.owner)}
                                </span>
                            </div>
                            <span
                                class={if task.done {
                                    "px-3 py-1 rounded-md bg-emerald-500 text-white"
                                } else {
                                    "px-3 py-1 rounded-md bg-amber-400 text-stone-950"
                                }}
                            >
                                {if task.done { "Done" } else { "Open" }}
                            </span>
                        </div>
                    }}
                </section>
            </div>
        }
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_| TaskListView::default())
        })
        .expect("failed to open task list window");
        cx.activate(true);
    });
}
