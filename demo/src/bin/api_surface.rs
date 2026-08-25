use gpui::prelude::*;
use gpui::*;
use gpui_component::tooltip::Tooltip;
use gpui_platform::application;
use gpui_rsx::rsx;
use std::time::Duration;

struct ApiSurfaceView {
    expanded: bool,
}

impl Default for ApiSurfaceView {
    fn default() -> Self {
        Self { expanded: true }
    }
}

impl Render for ApiSurfaceView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dynamic_class = "flex flex-grow-1 flex-shrink-1 self-center text-sky-300";

        rsx! {
            <div class="size-full flex flex-col gap-4 p-6 bg-zinc-950">
                <header class="flex flex-col gap-2">
                    <h1 class="text-3xl font-bold text-white">
                        {"API Surface"}
                    </h1>
                    <p class="text-sm text-zinc-400">
                        {"Compile coverage for GPUI helper methods used by gpui-rsx."}
                    </p>
                </header>

                <section class="flex flex-col gap-3 p-4 rounded-lg bg-zinc-900 border border-zinc-700">
                    <div class="flex items-stretch content-stretch justify-evenly gap-3">
                        <div class="flex-grow flex-shrink px-3 py-2 rounded-md bg-blue-500 text-white">
                            {"grow"}
                        </div>
                        <div class="flex-grow-0 flex-shrink-0 px-3 py-2 rounded-md bg-emerald-500 text-white">
                            {"fixed"}
                        </div>
                        <div class="flex-grow-1 flex-shrink-1 px-3 py-2 rounded-md bg-amber-400 text-zinc-950">
                            {"explicit"}
                        </div>
                    </div>

                    <div class="flex gap-3">
                        <span class="self-start px-3 py-1 rounded-md bg-zinc-800 text-zinc-100">{"self-start"}</span>
                        <span class="self-end px-3 py-1 rounded-md bg-zinc-800 text-zinc-100">{"self-end"}</span>
                        <span class="self-flex-start px-3 py-1 rounded-md bg-zinc-800 text-zinc-100">{"self-flex-start"}</span>
                        <span class="self-flex-end px-3 py-1 rounded-md bg-zinc-800 text-zinc-100">{"self-flex-end"}</span>
                        <span class="self-baseline px-3 py-1 rounded-md bg-zinc-800 text-zinc-100">{"self-baseline"}</span>
                    </div>

                    <div
                        class={if self.expanded {
                            "flex flex-col gap-2 self-stretch text-emerald-300"
                        } else {
                            "flex flex-col gap-2 self-center text-rose-300"
                        }}
                    >
                        <span class={dynamic_class}>{"dynamic class fast path"}</span>
                        <span whenClass={(self.expanded, "flex-grow-1 self-flex-end text-cyan-300")}>
                            {"whenClass"}
                        </span>
                    </div>

                    <div class="overflow-scroll h-16 p-3 rounded-md bg-zinc-950 border border-zinc-800">
                        <div class="aspect-square size-8 rounded-md bg-fuchsia-500" />
                        <p class="text-sm text-zinc-400">
                            {"Static overflow-scroll exercises stateful class auto-id handling."}
                        </p>
                    </div>

                    <div
                        class="flex gap-3 p-3 rounded-md bg-zinc-950 border border-zinc-800"
                        group={"api-surface"}
                    >
                        <span
                            class="px-3 py-1 rounded-md bg-zinc-800 text-zinc-100"
                            groupHover={("api-surface", |style| style.bg(rgb(0x334155)))}
                            groupActive={("api-surface", |style| style.opacity(0.85))}
                        >
                            {"group state"}
                        </span>
                    </div>

                    <button
                        class="px-4 py-2 rounded-md bg-sky-500 text-white cursor-pointer border border-sky-700"
                        flexGrow
                        flexShrink
                        role={Role::Button}
                        ariaLabel={"Toggle API surface state"}
                        ariaSelected={self.expanded}
                        hoverClass="bg-sky-600"
                        focusClass="border-sky-300"
                        focusVisible={|style| style.border_color(rgb(0xbae6fd))}
                        activeClass="opacity-75"
                        tooltip={|window, cx| Tooltip::new("Toggle the demo state").build(window, cx)}
                        tooltipShowDelay={Duration::from_millis(150)}
                        onClick={cx.listener(|view, _, _window, cx| {
                            view.expanded = !view.expanded;
                            cx.notify();
                        })}
                        onAuxClick={|_, _, _| {}}
                        onA11yAction={(AccessibleAction::Click, |_extra, _window, _cx| {})}
                    >
                        {"Toggle"}
                    </button>
                </section>
            </div>
        }
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_| ApiSurfaceView::default())
        })
        .expect("failed to open API surface window");
        cx.activate(true);
    });
}
