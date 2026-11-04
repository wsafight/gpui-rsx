use gpui::Context;
use gpui_rsx::rsx_update;
use std::cell::Cell;

#[test]
fn update_arguments_run_once_and_allow_borrowed_captures() {
    let mut view = 0usize;
    let mut context = Context::new();
    let arguments = Cell::new(0);
    let mut label = String::new();
    let result = rsx_update!(
        {
            arguments.set(arguments.get() + 1);
            &mut view
        },
        {
            arguments.set(arguments.get() + 1);
            &mut context
        },
        {
            arguments.set(arguments.get() + 1);
            |view, _| {
                *view += 1;
                label.push_str("updated");
                *view
            }
        },
    );
    context.notify();
    assert_eq!(arguments.get(), 3);
    assert_eq!(result, 1);
    assert_eq!(label, "updated");
}

#[test]
fn updater_lookup_preserves_names_similar_to_the_generated_helper() {
    let mut view = 0usize;
    let mut context = Context::new();
    let __gpui_rsx_update = |view: &mut usize, _: &mut Context<usize>| {
        *view += 1;
        *view
    };
    assert_eq!(rsx_update!(&mut view, &mut context, __gpui_rsx_update), 1);
}
