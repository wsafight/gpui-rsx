#[path = "../common/mod.rs"]
mod common;

use common::*;
use gpui_rsx::rsx;

fn main() {
    let _el = rsx! { <div class="opacity-250" /> };
    let _decimal = rsx! { <div class="opacity-100.1" /> };
    let _nan = rsx! { <div class="opacity-NaN" /> };
    let _infinite = rsx! { <div class="opacity-inf" /> };
    let _negative = rsx! { <div class="opacity--0.1" /> };
}
