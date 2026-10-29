use ui::{prelude::*, *};

fn main() {
    let classes = "p-2 flex";
    let _ = gpui_rsx::rsx! { <div class={classes} /> };
}
