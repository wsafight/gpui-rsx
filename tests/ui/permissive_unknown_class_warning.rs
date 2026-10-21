// Permissive mode keeps compiling but warns at the class literal; `deny` makes it checkable.
#![deny(deprecated)]

#[path = "../common/mod.rs"]
mod common;

use common::*;
use gpui_rsx::rsx;

fn main() {
    let _el = rsx! { <div class="flex md:flex-row hovr:flex" /> };
}
