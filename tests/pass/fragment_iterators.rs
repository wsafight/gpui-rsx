use gpui::prelude::*;
use gpui::*;
use gpui_rsx::rsx;

fn main() {
    let items = vec![1, 2, 3];
    let rest: Vec<AnyElement> = Vec::new();

    let _loop_only: Vec<AnyElement> = rsx! {
        <>{for item in items.clone() {<div>{item.to_string()}</div>}}</>
    };

    let _mixed: Vec<AnyElement> = rsx! {
        <>
            <div />
            <>{for item in items {<span>{item.to_string()}</span>}}</>
            {...rest}
        </>
    };
}
