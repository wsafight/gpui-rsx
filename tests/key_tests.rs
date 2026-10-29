use gpui::{div, prelude::*};
use gpui_rsx::rsx;

#[test]
fn integer_and_display_keys_compile_without_moving_the_key() {
    let n = 42u64;
    let label = String::from("row");
    let large = u128::MAX;
    let _ = rsx! { <div key={n} focusable /> };
    let _ = rsx! { <div key={label} focusable /> };
    let _ = rsx! { <div key={large} focusable /> };
    assert_eq!(label, "row");
    let mut calls = 0;
    let _ = rsx! { <div key={{ calls += 1; n }} focusable /> };
    assert_eq!(calls, 1);
}
