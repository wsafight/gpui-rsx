#![deny(elided_lifetimes_in_paths)]

use gpui::prelude::*;
use gpui::{Context, MouseButton, div};
use gpui_rsx::{rsx, rsx_listener};

struct View;

fn render(cx: &mut Context<'_, View>) -> impl IntoElement {
    let label = String::from("click");
    rsx! {
        <div
            onClick={rsx_listener!(cx, |_, _, _, _| {})}
            onMouseDown={(MouseButton::Left, rsx_listener!(cx, notify, move |_, _, _, _| {
                let _ = &label;
            }))}
        />
    }
}

fn main() {
    let _ = render(&mut Context::new());
}
