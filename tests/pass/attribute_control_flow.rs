#[path = "../common/mod.rs"]
mod common;

use common::*;
use gpui_rsx::rsx;

fn visible_question(value: Option<bool>) -> Option<MockElement> {
    Some(rsx! { <div visible={value?} /> })
}

fn static_if_question(value: Option<bool>) -> Option<MockElement> {
    Some(rsx! {
        <div class={if value? { "flex" } else { "grid" }} />
    })
}

fn static_match_question(value: Option<u8>) -> Option<MockElement> {
    Some(rsx! {
        <div class={match value? {
            0 => "flex",
            _ => "grid",
        }} />
    })
}

fn static_match_guard_question(value: Option<u8>) -> Option<MockElement> {
    Some(rsx! {
        <div class={match value {
            Some(n) if Some(n > 0)? => "flex",
            _ => "grid",
        }} />
    })
}

fn dynamic_class_question(value: Option<String>) -> Option<MockElement> {
    Some(rsx! { <div class={value?} /> })
}

fn dynamic_handler_question(class: &str, handler: Option<fn(&str)>) -> Option<MockElement> {
    Some(rsx! { <div class={class} onUnknownClass={handler?} /> })
}

async fn await_attributes() -> MockElement {
    let _visible = rsx! { <div visible={async { true }.await} /> };
    let _static_if = rsx! {
        <div class={if async { true }.await { "flex" } else { "grid" }} />
    };
    let _static_match = rsx! {
        <div class={match async { 0 }.await {
            0 => "flex",
            _ => "grid",
        }} />
    };
    rsx! { <div class={async { "flex" }.await} /> }
}

fn ignore_class(_: &str) {}

fn generated_names_do_not_capture_user_variables() {
    let __el = true;
    let __visible = true;
    let __class_branch = true;
    let __class_expr = "flex";
    let __class_str = "grid";
    let __class = "block";
    let __style = "hidden";
    let __on_unknown = "flex";
    let __handler: fn(&str) = ignore_class;

    let _visible = rsx! { <div visible={__el && __visible} /> };
    let _static_if = rsx! {
        <div class={if __el && __class_branch { "flex" } else { "grid" }} />
    };
    let _static_match = rsx! {
        <div class={match 0 {
            n if __el && n == 0 => "flex",
            _ => "grid",
        }} />
    };
    let _dynamic = rsx! {
        <div class={if __el { __class_expr } else { __class_str }} />
    };
    let _more_dynamic_names = rsx! {
        <div class={if __el { __class } else { __style }} />
    };
    let _dynamic_handler = rsx! {
        <div
            class={__on_unknown}
            onUnknownClass={|class| {
                let _ = (__el, class);
                __handler(class);
            }}
        />
    };
}

fn main() {}
