use gpui::Context;
use gpui_rsx::rsx_update;

fn main() {
    let mut view = 0usize;
    let mut context = Context::<usize>::new();
    rsx_update!(&mut view, &mut context, async |view, _| {
        *view += 1;
    });
}
