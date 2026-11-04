use gpui::Context;
use gpui_rsx::rsx_update;

fn main() {
    let mut view = String::new();
    let mut context = Context::<usize>::new();
    rsx_update!(&mut view, &mut context, |_, _| {});
}
