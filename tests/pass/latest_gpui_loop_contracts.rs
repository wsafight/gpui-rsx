#[path = "../common/mod.rs"]
mod common;

use common::*;
use gpui_rsx::rsx;

fn main() {
    let items = [1, 2, 3];

    let _stateful = rsx! {
        <div>
            {for item in items {
                <div key={item} ariaDescription={"description"} />
            }}
        </div>
    };

    let _flag = rsx! {
        <div>
            {for item in items {
                <div key={item} restrictScrollToAxis />
            }}
        </div>
    };

    let _styled_only = rsx! {
        <div>
            {for _item in items {
                <div scrollbarWidth={px(8.0)} />
            }}
        </div>
    };
}
