use gpui::{App, Context, Window};
use gpui_rsx::rsx_listener;
use std::cell::Cell;

struct View;

fn handler(_: &mut View, _: &(), _: &mut Window, _: &mut Context<View>) {}

#[test]
fn listener_expressions_are_evaluated_once_and_release_context_borrows() {
    let mut context = Context::<View>::new();
    let contexts = Cell::new(0);
    let handlers = Cell::new(0);
    let listener = rsx_listener!(
        {
            contexts.set(contexts.get() + 1);
            &context
        },
        notify,
        {
            handlers.set(handlers.get() + 1);
            handler
        },
    );
    context.notify();
    listener(&(), &mut Window, &mut App);
    assert_eq!(contexts.get(), 1);
    assert_eq!(handlers.get(), 1);
}

#[test]
fn handler_lookup_is_preserved_in_both_forms() {
    let context = Context::<View>::new();
    let __gpui_rsx_listener_with_notify = handler;
    let notify = handler;
    let plain = rsx_listener!(context, notify,);
    let wrapped = rsx_listener!(context, notify, __gpui_rsx_listener_with_notify);
    plain(&(), &mut Window, &mut App);
    wrapped(&(), &mut Window, &mut App);
}

#[test]
fn borrowed_and_unsized_event_types_keep_the_native_signature() {
    fn borrowed<'event>(
        context: &Context<View>,
    ) -> impl Fn(&&'event str, &mut Window, &mut App) + 'static {
        rsx_listener!(context, notify, |_, _: &&'event str, _, _| {})
    }
    let context = Context::<View>::new();
    let text = String::from("event");
    borrowed(&context)(&text.as_str(), &mut Window, &mut App);
    let unsized_listener = rsx_listener!(context, notify, |_, _: &str, _, _| {});
    unsized_listener(&text, &mut Window, &mut App);
}
