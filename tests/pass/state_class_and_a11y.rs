#[path = "../common/mod.rs"]
mod common;

use common::*;
use gpui_rsx::rsx;

fn main() {
    let aux = |_: ()| {};
    let a11y = |_: (), _: (), _: ()| {};

    let _el = rsx! {
        <button
            class="px-4 py-2 rounded-md bg-blue-500 text-white"
            hoverClass="bg-blue-600"
            focusClass="border-blue-500"
            activeClass="opacity-75"
            role={"button"}
            ariaLabel={"Save changes"}
            ariaSelected={false}
            onAuxClick={aux}
            onA11yAction={("press", a11y)}
        />
    };
}
