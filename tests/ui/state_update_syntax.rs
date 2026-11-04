use gpui_rsx::rsx_update;

fn main() {
    rsx_update!(view, cx);
    rsx_update!(view, cx, |_, _| {}, extra);
}
