#[path = "../common/mod.rs"]
mod common;

use common::*;
use gpui_rsx::rsx_strict;

fn main() {
    let _el = rsx_strict! { <div class="font-heavy" /> };
}
