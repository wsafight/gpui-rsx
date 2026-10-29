//! Runnable examples of the RSX grammar, shared with the rendering tests.
mod components;
mod rejected;

use components::{Badge, Panel};
use gpui::component::{ActiveTheme as _, button::Button};
use gpui::{
    App, Bounds, BoxShadow, Context, FontWeight, Image, ImageFormat, MouseButton, Pixels, Render,
    SharedString, TestSupportExt as _, Window, canvas, div, hsla, img, prelude::*, px, rgb, rgba,
    svg,
};
use gpui_rsx::{rsx, rsx_permissive, rsx_strict, rsx_tw};
use std::sync::Arc;

/// Toggle conditions, trigger events, and reorder keyed rows in a single window.
pub struct SyntaxGallery {
    pub selected: bool,
    pub clicks: usize,
    pub rows: Vec<u64>,
}

impl Default for SyntaxGallery {
    fn default() -> Self {
        Self {
            selected: false,
            clicks: 0,
            rows: vec![3, 1, 2],
        }
    }
}

mod ui {
    pub fn tile() -> gpui::Div {
        gpui::div()
    }
}

/// A local, embedded image keeps the special-tag example independent of the network.
pub fn sample_image() -> Arc<Image> {
    Arc::new(Image::from_bytes(
        ImageFormat::Svg,
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><rect width="24" height="24" rx="5" fill="#3b82f6"/></svg>"##.to_vec(),
    ))
}

impl Render for SyntaxGallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dynamic: SharedString = "rounded-md bg-blue-500/50".into();
        let width = self.selected.then_some(180.0);
        let extra = rsx! { <><span>{"spread A"}</span><span>{"spread B"}</span></> };
        rsx! {
            <main id="syntax-gallery" class="size-full overflow-scroll p-16 bg-background text-foreground">
                <div class="flex flex-col gap-12">
                    <header>
                        <h1 styled>"RSX Syntax Gallery"</h1>
                        <p>"Toggle, click and reverse rows to try the syntax."</p>
                    </header>
                    <Panel title="Elements, attributes and events" compactIf={self.selected}>
                        <button id="syntax-toggle" class="w-180 h-36 rounded-md bg-blue-500 text-white"
                            hoverClass="bg-blue-600" activeClass="opacity-75"
                            focusClass="border-2" focusVisible={|style| style.underline()}
                            role={gpui::Role::Button} ariaLabel="Toggle conditions"
                            onClick={cx.listener(|view, _, _, cx| {
                                view.selected = !view.selected;
                                view.clicks += 1;
                                cx.notify();
                            })} test_support>
                            {format!("Toggle ({})", self.clicks)}
                        </button>
                        <div flex itemsCenter gap={px(8.)}
                            backgroundColor={rgb(0x27272a)} textColor={rgb(0xffffff)}
                            onMouseDown={(MouseButton::Right, |_, _, _| {})}>
                            <ui::tile class="p-8">"path constructor"</ui::tile>
                            <Badge label="Function component" highlightedIf={self.selected} />
                            <Button base={Button::new("syntax-kit")} label="Kit builder" />
                        </div>
                    </Panel>
                    <Panel title="Conditions and class expressions">
                        <div id="syntax-conditional" class={if self.selected { "p-8 bg-emerald-500" } else { "p-4 bg-zinc-700" }}
                            when={(self.selected, |el| el.border_2())}
                            whenSome={(width, |el, width| el.w(px(width)))}
                            whenClass={(self.selected, "rounded-lg")}
                            visible={true}>
                            {if self.selected { rsx! { <span>"selected"</span> } } else { rsx! { <span>"idle"</span> } }}
                        </div>
                        <div class={match self.clicks % 2 { 0 => "text-blue-500", _ => "text-emerald-500" }}>"match class"</div>
                        <div class={dynamic}>"SharedString dynamic class"</div>
                        <div class={format!("p-8 {} border-2", "rounded-md")}>"format! class"</div>
                        <div class="group/menu">
                            <span class="hover:underline focus:italic focus-visible:underline active:opacity-75 group-hover/menu:text-blue-500 group-active/menu:underline">"State variants"</span>
                        </div>
                        {rsx_strict! { <div class="text-[14px] leading-20 rounded-[4px] bg-[#123456]/50">"strict / arbitrary values"</div> }}
                        {rsx_permissive! { <div class="p-4">"permissive macro"</div> }}
                        {rsx_tw! { <div class="p-2 gap-2">"Tailwind: p-2 = 8px"</div> }}
                        <div class="text-size-body leading-body opacity-muted shadow-card">"rsx.toml tokens"</div>
                    </Panel>
                    <Panel title="Fragments, spread and keyed loops">
                        <><span>"nested fragment"</span><></><span>"without a wrapper"</span></>
                        {...extra}
                        <button id="syntax-reverse" class="w-180 h-32 bg-zinc-700"
                            onClick={cx.listener(|view, _, _, cx| { view.rows.reverse(); cx.notify(); })} test_support>
                            "Reverse rows"
                        </button>
                        {for (index, key) in self.rows.iter().enumerate() {
                            <>
                                <button key={*key} class="h-24" onClick={|_, _, _| {}}>{format!("Row {key} at {index}")}</button>
                                {for detail in 0..1 { <span>{format!("detail {detail}")}</span> }}
                            </>
                        }}
                    </Panel>
                    <Panel title="SVG, image and canvas">
                        <div class="flex gap-12">
                            <svg src="icons/check.svg" class="size-24 text-blue-500" />
                            <img src={sample_image()} grayscale class="size-24" />
                            <img source={sample_image()} class="size-24" />
                            <canvas class="w-80 h-24"
                                prepaint={|bounds: Bounds<Pixels>, _, _| bounds}
                                paint={|_, bounds, window, cx| {
                                    window.paint_quad(gpui::fill(bounds, rgb(0x10b981)));
                                    let _: &mut App = cx;
                                }} />
                        </div>
                    </Panel>
                </div>
            </main>
        }
    }
}
