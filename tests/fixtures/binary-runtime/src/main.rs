use gpui::{prelude::*, *};

gpui_rsx::rsx_runtime!();

fn main() {
    let classes = "flex gap-4";
    let _ = gpui_rsx::rsx! { <div class={classes} /> };
}

#[test]
fn binary_uses_its_own_runtime() {
    main();
}
