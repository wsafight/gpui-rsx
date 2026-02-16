//! Todo App 示例 - 展示更复杂的 GPUI-RSX 用法

use gpui::*;
use gpui_rsx::rsx;

#[derive(Clone)]
struct Todo {
    id: usize,
    text: String,
    completed: bool,
}

struct TodoApp {
    todos: Vec<Todo>,
    input: String,
    filter: Filter,
}

#[derive(Clone, Copy, PartialEq)]
enum Filter {
    All,
    Active,
    Completed,
}

impl TodoApp {
    fn new() -> Self {
        Self {
            todos: vec![
                Todo {
                    id: 0,
                    text: "学习 GPUI".to_string(),
                    completed: true,
                },
                Todo {
                    id: 1,
                    text: "使用 GPUI-RSX".to_string(),
                    completed: false,
                },
                Todo {
                    id: 2,
                    text: "构建桌面应用".to_string(),
                    completed: false,
                },
            ],
            input: String::new(),
            filter: Filter::All,
        }
    }

    fn add_todo(&mut self) {
        if !self.input.trim().is_empty() {
            let id = self.todos.len();
            self.todos.push(Todo {
                id,
                text: self.input.clone(),
                completed: false,
            });
            self.input.clear();
        }
    }

    fn toggle_todo(&mut self, id: usize) {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = !todo.completed;
        }
    }

    fn delete_todo(&mut self, id: usize) {
        self.todos.retain(|t| t.id != id);
    }

    fn filtered_todos(&self) -> Vec<&Todo> {
        match self.filter {
            Filter::All => self.todos.iter().collect(),
            Filter::Active => self.todos.iter().filter(|t| !t.completed).collect(),
            Filter::Completed => self.todos.iter().filter(|t| t.completed).collect(),
        }
    }

    fn active_count(&self) -> usize {
        self.todos.iter().filter(|t| !t.completed).count()
    }
}

impl Render for TodoApp {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let filtered = self.filtered_todos();

        rsx! {
            <div class="flex flex-col gap-4 p-4" bg={rgb(0xffffff)}>
                // Header
                <div class="text-3xl font-bold" text_color={rgb(0xb83280)}>
                    {"todos"}
                </div>

                // Input
                <div class="flex gap-2">
                    <input
                        class="flex-1 px-4 py-2 rounded-md"
                        bg={rgb(0xf3f4f6)}
                        placeholder="What needs to be done?"
                    />
                    <button
                        bg={rgb(0x3b82f6)}
                        text_color={rgb(0xffffff)}
                        px_6
                        py_2
                        rounded_md
                        onClick={cx.listener(|view, _, cx| {
                            view.add_todo();
                            cx.notify();
                        })}
                    >
                        {"Add"}
                    </button>
                </div>

                // Todo List
                <div class="flex flex-col gap-2">
                    {filtered.iter().map(|todo| {
                        let todo_id = todo.id;
                        let completed = todo.completed;

                        rsx! {
                            <div
                                class="flex gap-3 items-center p-3 rounded-md"
                                bg={if completed {
                                    rgb(0xf3f4f6)
                                } else {
                                    rgb(0xffffff)
                                }}
                            >
                                // Checkbox
                                <div
                                    w={px(20.0)}
                                    h={px(20.0)}
                                    rounded={px(4.0)}
                                    bg={if completed {
                                        rgb(0x3b82f6)
                                    } else {
                                        rgb(0xe5e7eb)
                                    }}
                                    onClick={cx.listener(move |view, _, cx| {
                                        view.toggle_todo(todo_id);
                                        cx.notify();
                                    })}
                                />

                                // Text
                                <div
                                    class="flex-1"
                                    text_color={if completed {
                                        rgb(0x9ca3af)
                                    } else {
                                        rgb(0x1f2937)
                                    }}
                                >
                                    {todo.text.clone()}
                                </div>

                                // Delete Button
                                <button
                                    bg={rgb(0xef4444)}
                                    text_color={rgb(0xffffff)}
                                    px_3
                                    py_1
                                    rounded_md
                                    onClick={cx.listener(move |view, _, cx| {
                                        view.delete_todo(todo_id);
                                        cx.notify();
                                    })}
                                >
                                    {"Delete"}
                                </button>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                // Footer
                <div class="flex gap-4 items-center justify-between">
                    // Item count
                    <div text_color={rgb(0x6b7280)}>
                        {format!("{} items left", self.active_count())}
                    </div>

                    // Filters
                    <div class="flex gap-2">
                        {self.render_filter_button(Filter::All, "All", cx)}
                        {self.render_filter_button(Filter::Active, "Active", cx)}
                        {self.render_filter_button(Filter::Completed, "Completed", cx)}
                    </div>

                    // Clear completed
                    <button
                        text_color={rgb(0x6b7280)}
                        onClick={cx.listener(|view, _, cx| {
                            view.todos.retain(|t| !t.completed);
                            cx.notify();
                        })}
                    >
                        {"Clear completed"}
                    </button>
                </div>
            </div>
        }
    }
}

impl TodoApp {
    fn render_filter_button(
        &self,
        filter: Filter,
        label: &str,
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        let is_active = self.filter == filter;

        rsx! {
            <button
                px_3
                py_1
                rounded_md
                bg={if is_active {
                    rgb(0x3b82f6)
                } else {
                    rgb(0xe5e7eb)
                }}
                text_color={if is_active {
                    rgb(0xffffff)
                } else {
                    rgb(0x6b7280)
                }}
                onClick={cx.listener(move |view, _, cx| {
                    view.filter = filter;
                    cx.notify();
                })}
            >
                {label}
            </button>
        }
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(|_cx| TodoApp::new())
        });
    });
}
