use gpui::prelude::*;
use gpui::*;
use gpui_platform::application;
use gpui_rsx::rsx;

struct Swatch {
    name: &'static str,
    class_name: &'static str,
    value: &'static str,
}

struct PaletteView {
    swatches: Vec<Swatch>,
}

impl Default for PaletteView {
    fn default() -> Self {
        Self {
            swatches: vec![
                Swatch {
                    name: "Blue",
                    class_name: "bg-blue-500",
                    value: "#3b82f6",
                },
                Swatch {
                    name: "Emerald",
                    class_name: "bg-emerald-500",
                    value: "#10b981",
                },
                Swatch {
                    name: "Rose",
                    class_name: "bg-rose-500",
                    value: "#f43f5e",
                },
                Swatch {
                    name: "Amber",
                    class_name: "bg-amber-400",
                    value: "#fbbf24",
                },
            ],
        }
    }
}

impl Render for PaletteView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="size-full flex flex-col gap-4 p-6 bg-zinc-950">
                <header class="flex flex-col gap-2">
                    <h1 class="text-3xl font-bold text-white">
                        {"Palette"}
                    </h1>
                    <p class="text-sm text-zinc-400">
                        {"Static and dynamic class paths can live in the same view."}
                    </p>
                </header>

                <section class="flex flex-col gap-3">
                    {for swatch in self.swatches.iter() {
                        <div class="flex items-center gap-4 p-4 rounded-lg bg-zinc-900 border border-zinc-700">
                            <div class={format!("size-8 rounded-md {}", swatch.class_name)} />
                            <div class="flex flex-col gap-1">
                                <span class="text-base text-white">{swatch.name}</span>
                                <span class="text-sm text-zinc-400">{swatch.value}</span>
                            </div>
                        </div>
                    }}
                </section>
            </div>
        }
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let _ = cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_| PaletteView::default())
        });
        cx.activate(true);
    });
}
