#[path = "../common/mod.rs"]
mod common;

use common::*;
use gpui_rsx::rsx;

fn main() {
    let _static = rsx! { <div class="opacity-0 opacity-33.5 opacity-100" /> };

    let decimal = "opacity-33.5";
    let _dynamic_decimal = rsx! { <div class={decimal} /> };
    let lower = "opacity-0";
    let _dynamic_lower = rsx! { <div class={lower} /> };
    let upper = "opacity-100";
    let _dynamic_upper = rsx! { <div class={upper} /> };
}
