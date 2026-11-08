mod common;

use gpui::component::ActiveTheme as _;
use gpui::prelude::*;
use gpui::*;
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="size-full overflow-y-scroll flex flex-col gap-[24px] p-[32px] bg-background text-foreground">
                <header class="flex flex-col gap-[8px]">
                    <h1 class="text-2xl font-semibold">
                        {"Palette"}
                    </h1>
                    <p class="text-sm text-muted-foreground">
                        {"Each swatch uses a dynamic background class. The layout uses static classes."}
                    </p>
                </header>

                <section class="flex flex-col">
                    {for swatch in self.swatches.iter() {
                        <div class="flex items-center gap-[16px] py-[16px] border-b border-border">
                            <div class={format!("size-[48px] rounded-md {}", swatch.class_name)} />
                            <div class="flex-1 flex flex-col gap-[4px]">
                                <span class="text-base font-medium">{swatch.name}</span>
                                <span class="text-sm text-muted-foreground">{swatch.class_name}</span>
                            </div>
                            <span class="text-sm text-muted-foreground">{swatch.value}</span>
                        </div>
                    }}
                </section>
            </div>
        }
    }
}

fn main() {
    common::run(
        |cx| common::window(cx, "Palette · GPUI-RSX", 720.0, 600.0),
        |_| PaletteView::default(),
    );
}
