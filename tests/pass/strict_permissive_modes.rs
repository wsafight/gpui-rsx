#[path = "../common/mod.rs"]
mod common;

use common::*;
use gpui_rsx::{rsx_permissive, rsx_strict};

fn main() {
    let _strict = rsx_strict! { <div class="flex" /> };
    let _permissive = rsx_permissive! { <div class="hover:bg-blue-500 flex" /> };
    let _custom_font = rsx_permissive! { <div class="font-display" /> };

    let dynamic = "unsupported-at-runtime";
    let _dynamic_permissive = rsx_permissive! { <div class={dynamic} /> };
}
