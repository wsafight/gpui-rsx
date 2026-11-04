pub use gpui_reexport_fixture::{rsx_runtime, ui};
use ui::{prelude::*, *};

fn main() {
    let classes = String::from("p-2");
    let _ = gpui_rsx::rsx! { <div class={classes} /> };
}
