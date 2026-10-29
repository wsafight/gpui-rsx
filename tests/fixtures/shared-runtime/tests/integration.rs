use ui::{prelude::*, *};

#[test]
fn integration_tests_use_the_library_helper_and_preserve_callbacks() {
    let classes = "flex gap-4 hover:bg-brand unknown";
    let mut unknown = Vec::new();
    let _ = gpui_rsx::rsx_strict! {
        <div class={classes} onUnknownClass={|class: &str| unknown.push(class.to_owned())} />
    };
    assert_eq!(unknown, ["hover:bg-brand", "unknown"]);
    let _ = runtime_fixture::render("p-2");
}

#[test]
#[should_panic(expected = "unsupported dynamic class")]
fn strict_unknown_class_still_panics() {
    let classes = "unknown";
    let _ = gpui_rsx::rsx_strict! { <div class={classes} /> };
}
