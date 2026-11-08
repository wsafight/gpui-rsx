use gpui::{prelude::*, *};

#[cfg(all(feature = "builder", feature = "markup"))]
compile_error!("select builder or markup, not both");

struct HotReloadView {
    renders: usize,
}

impl Render for HotReloadView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders += 1;
        content(self.renders)
    }
}

fn report(label: &str, renders: usize) -> String {
    let message = format!(
        "GPUI_RSX_PATCH pid={} renders={renders} label={label}",
        std::process::id()
    );
    println!("{message}");
    message
}

#[cfg(feature = "builder")]
fn content(renders: usize) -> impl IntoElement {
    div()
        .size_full()
        .p_4()
        .bg(rgb(0x1e293b))
        .text_color(rgb(0xf8fafc))
        .child(report("builder-v1", renders))
}

#[cfg(all(feature = "markup", not(feature = "builder")))]
fn content(renders: usize) -> impl IntoElement {
    gpui_rsx::rsx! {
        <div class="size-full p-4 bg-slate-800 text-slate-50">
            {report("markup-v1", renders)}
        </div>
    }
}

#[cfg(not(any(feature = "builder", feature = "markup")))]
fn content(renders: usize) -> impl IntoElement {
    use gpui_rsx::rsx as gpui_view;

    gpui_view! {
        <div class="size-full p-4 bg-slate-800 text-slate-50">
            {report("alias-v1", renders)}
        </div>
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stdout)
        .init();
    #[cfg(feature = "kit")]
    let application = gpui_kit::application().with_assets(gpui_kit::assets::Assets);
    #[cfg(not(feature = "kit"))]
    let application = gpui_platform::application();
    application.run(|cx: &mut App| {
        #[cfg(feature = "kit")]
        gpui_kit::init(cx);
        #[cfg(feature = "kit")]
        let window = gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| HotReloadView { renders: 0 })
        });
        #[cfg(not(feature = "kit"))]
        let window = cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_| HotReloadView { renders: 0 })
        });
        window.expect("failed to open hot-reload fixture");
        cx.activate(true);
    });
}
