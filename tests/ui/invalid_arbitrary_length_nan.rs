#[path = "../common/mod.rs"]
mod common;

use common::*;
use zopra_gpui_view::rsx;

fn main() {
    let _el = rsx! { <div class="w-[NaNpx]" /> };
}
