#[path = "../common/mod.rs"]
mod common;

use common::*;
use gpui_rsx::rsx;

fn main() {
    let _el = rsx! { <div class="opacity-250" /> };
}
