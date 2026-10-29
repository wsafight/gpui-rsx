use ui::{prelude::*, *};

fn main() {
    let classes = "gap-4 bg-red-500";
    let _ = gpui_rsx::rsx_tw! { <div class={classes} /> };
}
