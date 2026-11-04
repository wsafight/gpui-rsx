use gpui::{AppContext, TestAppContext};
use gpui_rsx::rsx_update;
use std::{cell::Cell, rc::Rc};

#[derive(Default)]
struct Counter {
    count: usize,
    label: String,
}

#[gpui::test]
fn explicit_updates_batch_changes_return_values_and_keep_native_assignments(
    cx: &mut TestAppContext,
) {
    let entity = cx.new(|_| Counter::default());
    let notifications = Rc::new(Cell::new(0));
    let observed = notifications.clone();
    let _subscription =
        cx.update(|cx| cx.observe(&entity, move |_, _| observed.set(observed.get() + 1)));

    let label = String::from("batched");
    let result = entity.update(cx, |view, cx| {
        rsx_update!(view, cx, move |view, _| {
            view.count += 1;
            view.label = label;
            view.count
        })
    });
    assert_eq!(result, 1);
    assert_eq!(notifications.get(), 1);
    entity.update(cx, |view, _| view.count += 1);
    assert_eq!(notifications.get(), 1);
    let result = entity.update(cx, |view, cx| {
        rsx_update!(view, cx, |view, _| {
            if view.count == 2 {
                return false;
            }
            view.count += 1;
            true
        })
    });
    assert!(!result);
    assert_eq!(notifications.get(), 2);
    entity.read_with(cx, |view, _| {
        assert_eq!(view.count, 2);
        assert_eq!(view.label, "batched");
    });
}

#[gpui::test]
fn fallible_updates_notify_on_error_and_borrow_local_values(cx: &mut TestAppContext) {
    let entity = cx.new(|_| Counter::default());
    let notifications = Rc::new(Cell::new(0));
    let observed = notifications.clone();
    let _subscription =
        cx.update(|cx| cx.observe(&entity, move |_, _| observed.set(observed.get() + 1)));
    let mut labels = Vec::new();
    let result: Result<(), &str> = entity.update(cx, |view, cx| {
        rsx_update!(view, cx, |view, _| {
            view.count += 1;
            labels.push(view.count);
            Err("partial update")?;
            Ok(())
        })
    });
    assert_eq!(result, Err("partial update"));
    assert_eq!(labels, [1]);
    assert_eq!(notifications.get(), 1);
    entity.read_with(cx, |view, _| assert_eq!(view.count, 1));
}

#[gpui::test]
fn panicking_updates_keep_mutations_without_notifying_and_can_recover(cx: &mut TestAppContext) {
    let entity = cx.new(|_| Counter::default());
    let notifications = Rc::new(Cell::new(0));
    let observed = notifications.clone();
    let _subscription =
        cx.update(|cx| cx.observe(&entity, move |_, _| observed.set(observed.get() + 1)));
    entity.update(cx, |view, cx| {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rsx_update!(view, cx, |view, _| {
                view.count = 7;
                panic!("update interrupted");
            });
        }));
        assert!(result.is_err());
    });
    assert_eq!(notifications.get(), 0);
    entity.read_with(cx, |view, _| assert_eq!(view.count, 7));
    entity.update(cx, |view, cx| {
        rsx_update!(view, cx, |view, _| view.label = String::from("recovered"));
    });
    assert_eq!(notifications.get(), 1);
    entity.read_with(cx, |view, _| assert_eq!(view.label, "recovered"));
}

#[gpui::test]
fn explicit_updates_notify_the_matching_entity(cx: &mut TestAppContext) {
    let first = cx.new(|_| Counter::default());
    let second = cx.new(|_| Counter::default());
    let first_notifications = Rc::new(Cell::new(0));
    let second_notifications = Rc::new(Cell::new(0));
    let observed = first_notifications.clone();
    let _first_subscription =
        cx.update(|cx| cx.observe(&first, move |_, _| observed.set(observed.get() + 1)));
    let observed = second_notifications.clone();
    let _second_subscription =
        cx.update(|cx| cx.observe(&second, move |_, _| observed.set(observed.get() + 1)));

    first.update(cx, |view, cx| {
        rsx_update!(view, cx, |view, _| view.count = 1)
    });
    assert_eq!(
        (first_notifications.get(), second_notifications.get()),
        (1, 0)
    );
    second.update(cx, |view, cx| {
        rsx_update!(view, cx, |view, _| view.count = 2)
    });
    assert_eq!(
        (first_notifications.get(), second_notifications.get()),
        (1, 1)
    );
    first.read_with(cx, |view, _| assert_eq!(view.count, 1));
    second.read_with(cx, |view, _| assert_eq!(view.count, 2));
}
