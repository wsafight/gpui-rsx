use std::time::Duration;

use gpui::{prelude::*, *};
use gpui_rsx::rsx as gpui_view;

struct LiveView {
    seconds: usize,
    clicks: usize,
}

impl LiveView {
    fn new(cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |view, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                if view
                    .update(cx, |view, cx| {
                        view.seconds += 1;
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();

        Self {
            seconds: 0,
            clicks: 0,
        }
    }
}

fn report(revision: &str, seconds: usize, clicks: usize) -> String {
    let pid = std::process::id();
    println!("GPUI_RSX_LIVE pid={pid} seconds={seconds} clicks={clicks} revision={revision}");
    format!("PID {pid}   /   运行 {seconds} 秒   /   点击 {clicks} 次")
}

impl Render for LiveView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        gpui_view! {
            <div class="size-full flex flex-col gap-6 p-8 bg-slate-950 text-slate-50">
                <div class="text-2xl font-bold">{"GPUI-RSX 热更新演示"}</div>
                <div class="flex flex-col gap-4 p-6 rounded-xl bg-emerald-600 text-white">
                    <div class="text-3xl font-bold">{"REVISION B / 绿色"}</div>
                    <div class="text-lg">{"源码已更新，秒数和点击计数继续保留。"}</div>
                    <div class="text-lg">
                        {report("revision-b", self.seconds, self.clicks)}
                    </div>
                </div>
                <button
                    class="px-5 py-3 rounded-lg bg-slate-700 text-white cursor-pointer"
                    onClick={cx.listener(|view, _, _window, cx| {
                        view.clicks += 1;
                        cx.notify();
                    })}
                >
                    {"点我增加计数，观察更新后是否保留"}
                </button>
                <div class="text-sm text-slate-300">
                    {"秒数递增是定时器；版本文字和面板配色改变来自源码热更新。"}
                </div>
            </div>
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stdout)
        .init();

    gpui_platform::application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(760.0), px(470.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(LiveView::new),
        )
        .expect("failed to open live hot-reload demo");
        cx.activate(true);
    });
}
