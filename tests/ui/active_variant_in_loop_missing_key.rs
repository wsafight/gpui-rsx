#[path = "../common/mod.rs"]
mod common;

use common::*;
use gpui_rsx::rsx;

fn main() {
    let items = vec![1, 2];
    let _el = rsx! {
        <div>
            {for item in items {
                <div class="active:opacity-75">{item}</div>
            }}
        </div>
    };
}
