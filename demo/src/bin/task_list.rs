mod common;

use gpui::component::{ActiveTheme as _, button::Button};
use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{rsx, rsx_listener};

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
                    title: "Review the interactive examples",
                    owner: "Docs",
                    done: false,
                },
            ],
        }
    }
}

impl TaskListView {
    fn completed_count(&self) -> usize {
        self.tasks.iter().filter(|task| task.done).count()
    }

    fn toggle_task(&mut self, task_id: usize) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == task_id) {
            task.done = !task.done;
        }
    }
}

impl Render for TaskListView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="size-full overflow-y-scroll flex flex-col gap-[24px] p-[32px] bg-background text-foreground">
                <header class="flex flex-col gap-[8px]">
                    <h1 class="text-2xl font-semibold">
                        {"Task list"}
                    </h1>
                    <p class="text-sm text-muted-foreground">
                        {format!("{} of {} tasks complete. Each row keeps its task ID.", self.completed_count(), self.tasks.len())}
                    </p>
                </header>

                <section class="flex flex-col">
                    {for task in self.tasks.iter() {
                        <div
                            key={task.id}
                            class="flex items-center justify-between gap-[20px] py-[18px] border-b border-border"
                        >
                            <div class="flex-1 min-w-0 flex flex-col gap-[6px]">
                                <span class="text-base" whenClass={(task.done, "text-muted-foreground line-through")}>{task.title}</span>
                                <span class="text-sm text-muted-foreground">
                                    {format!("#{} assigned to {}", task.id, task.owner)}
                                </span>
                            </div>
                            <Button
                                key={task.id}
                                base={Button::new(SharedString::from(format!("task-toggle-{}", task.id)))}
                                label={if task.done { "Reopen" } else { "Complete" }}
                                onClick={rsx_listener!(cx, notify, {
                                    let task_id = task.id;
                                    move |view, _, _window, _cx| {
                                        view.toggle_task(task_id);
                                    }
                                })}
                            />
                        </div>
                    }}
                </section>
            </div>
        }
    }
}

fn main() {
    common::run(
        |cx| common::window(cx, "Task list · GPUI-RSX", 800.0, 560.0),
        |_| TaskListView::default(),
    );
}

#[cfg(test)]
mod tests {
    use super::TaskListView;

    #[test]
    fn completion_tracks_task_identity_after_reordering() {
        let mut view = TaskListView::default();
        view.tasks.reverse();
        view.toggle_task(3);
        assert_eq!(view.completed_count(), 3);
        assert!(view.tasks.iter().find(|task| task.id == 3).unwrap().done);
        view.toggle_task(3);
        assert_eq!(view.completed_count(), 2);
    }

    #[test]
    fn unknown_tasks_leave_completion_unchanged() {
        let mut view = TaskListView::default();
        view.toggle_task(99);
        assert_eq!(view.completed_count(), 2);
        assert_eq!(view.tasks.len(), 3);
    }
}
