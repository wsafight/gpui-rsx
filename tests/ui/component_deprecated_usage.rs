#![deny(deprecated)]

use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx};

#[component]
#[deprecated(note = "use NewCard")]
fn OldCard(title: String) -> impl IntoElement {
    div().child(title)
}

fn main() {
    let _ = rsx! { <OldCard title="Inbox" /> };
}
