#[path = "../common/mod.rs"]
mod common;

use common::*;
use zopra_gpui_view::rsx_strict;

fn main() {
    let _el = rsx_strict! { <div class="font-heavy" /> };
}
