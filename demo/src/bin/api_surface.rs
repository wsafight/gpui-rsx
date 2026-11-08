mod common;

use gpui::component::{ActiveTheme as _, tooltip::Tooltip};
use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{rsx, rsx_listener};
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
        let dynamic_class = "flex flex-grow-1 flex-shrink-1 self-center text-primary";

        rsx! {
            <div class="size-full overflow-y-scroll flex flex-col gap-[24px] p-[32px] bg-background text-foreground">
                <header class="flex flex-col gap-[8px]">
                    <h1 class="text-2xl font-semibold">
                        {"API surface"}
                    </h1>
                    <p class="text-sm text-muted-foreground">
                        {"Compare flex alignment and group states. Toggle changes the conditional styles."}
                    </p>
                </header>

                <section class="flex flex-col gap-[20px] p-[24px] rounded-lg bg-muted border border-border">
                    <div class="flex items-stretch content-stretch justify-evenly gap-[12px]">
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

                    <div class="flex flex-wrap gap-[12px]">
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

                    <div class="overflow-scroll h-[96px] p-[12px] rounded-md bg-background border border-border">
                        <div class="aspect-square size-[48px] rounded-md bg-emerald-500" />
                        <p class="text-sm text-muted-foreground">
                            {"Scroll this area to inspect its element state."}
                        </p>
                    </div>

                    <div
                        class="flex gap-[12px] p-[12px] rounded-md bg-background border border-border"
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
                        onClick={rsx_listener!(cx, notify, |view, _, _window, _cx| {
                            view.expanded = !view.expanded;
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
    common::run(
        |cx| common::window(cx, "API surface · GPUI-RSX", 960.0, 760.0),
        |_| ApiSurfaceView::default(),
    );
}
