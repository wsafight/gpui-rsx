pub use gpui_reexport_fixture::{rsx_runtime, ui};
use ui::{prelude::*, *};

mod explicit_runtime {
    gpui_rsx::rsx_runtime!(::engine);
}

#[test]
fn integration_targets_provide_their_own_crate_reexport() {
    let classes = String::from("flex p-4 unknown");
    let mut unknown = Vec::new();
    let _ = gpui_rsx::rsx_strict! {
        <div class={classes} onUnknownClass={|class: &str| unknown.push(class.to_owned())} />
    };
    assert_eq!(unknown, ["unknown"]);
    let _ = gpui_reexport_fixture::render("flex");
    let _ =
        explicit_runtime::rsx_runtime::px(StyleRefinement, "flex", &mut |_| panic!("known class"));
}
