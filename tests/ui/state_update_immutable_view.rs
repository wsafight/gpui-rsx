use gpui::Context;
use gpui_rsx::rsx_update;

fn main() {
    let view = 0usize;
    let mut context = Context::<usize>::new();
    rsx_update!(&view, &mut context, |_, _| {});
}
