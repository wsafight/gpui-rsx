use gpui::*;

pub(crate) fn run<V, Options, Build>(options: Options, build: Build)
where
    V: Render + 'static,
    Options: FnOnce(&mut App) -> WindowOptions + 'static,
    Build: FnOnce(&mut Context<V>) -> V + 'static,
{
    application()
        .with_assets(assets::Assets)
        .run(move |cx: &mut App| {
            gpui::init(cx);
            let window_options = options(cx);
            gpui::open_window(window_options, cx, |_, cx| cx.new(build))
                .expect("failed to open demo window");
            cx.activate(true);
        });
}
