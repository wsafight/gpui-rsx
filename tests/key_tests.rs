use gpui::{SharedString, div, prelude::*};
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

#[test]
fn borrowed_and_shared_string_keys_compile_without_static_lifetimes() {
    fn view(key: &str) -> impl IntoElement {
        rsx! { <div key={key} onClick={|_, _, _| {}} /> }
    }

    let owned = String::from("row");
    let shared = SharedString::from("shared");
    let _ = view(&owned);
    let _ = rsx! { <div key={shared} focusable /> };
    let _ = rsx! { <div key={&owned} focusable /> };
    assert_eq!(owned, "row");
    assert_eq!(shared, SharedString::from("shared"));
}

#[test]
fn display_fallback_evaluates_and_formats_the_key_once() {
    struct Key<'a>(&'a std::cell::Cell<usize>);

    impl std::fmt::Display for Key<'_> {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.set(self.0.get() + 1);
            formatter.write_str("row")
        }
    }

    fn view(key: impl std::fmt::Display) -> impl IntoElement {
        rsx! { <div key={key} focusable /> }
    }

    let evaluations = std::cell::Cell::new(0);
    let formats = std::cell::Cell::new(0);
    let key = Key(&formats);
    let _ = rsx! { <div key={{ evaluations.set(evaluations.get() + 1); &key }} focusable /> };
    assert_eq!(evaluations.get(), 1);
    assert_eq!(formats.get(), 1);
    let _ = view(&key);
    assert_eq!(formats.get(), 2);
    let _ = view(u128::MAX);
    let _ = view(i128::MIN);
}
