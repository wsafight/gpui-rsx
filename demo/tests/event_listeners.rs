use gpui::test::{TestSupportExt, TestWindowExt};
use gpui::{
    AppContext, Context, MouseButton, MouseDownEvent, Render, TestAppContext, Window, div,
    prelude::*, px,
};
use gpui_rsx::{rsx, rsx_listener, rsx_update};
use std::{cell::Cell, rc::Rc};

#[derive(Default)]
struct Counter {
    native: usize,
    assisted: usize,
    mouse: usize,
    quiet: usize,
    batched: usize,
    label: String,
}

impl Render for Counter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let increment = 3;
        rsx! {
            <div flex flex_col>
                <div id="native" class="w-40 h-20" test_support
                    onClick={cx.listener(|view, _, _, cx| {
                        view.native += 1;
                        cx.notify();
                    })} />
                <div id="assisted" class="w-40 h-20" test_support
                    onClick={rsx_listener!(cx, notify, move |view, _, _, _| {
                        if view.assisted > 0 {
                            return;
                        }
                        view.assisted += increment;
                    })} />
                <div id="mouse" class="w-40 h-20" test_support
                    onMouseDown={(MouseButton::Right, rsx_listener!(cx, notify, |view, event: &MouseDownEvent, _, _| {
                        assert_eq!(event.button, MouseButton::Right);
                        view.mouse += 1;
                    }))} />
                <div id="quiet" class="w-40 h-20" test_support
                    onClick={rsx_listener!(cx, |view, _, _, _| view.quiet += 1)} />
                <div id="batched" class="w-40 h-20" test_support
                    onClick={rsx_listener!(cx, |view, _, _, cx| {
                        let result = rsx_update!(view, cx, |view, _| {
                            view.batched += 2;
                            view.label = String::from("clicked");
                            view.batched
                        });
                        assert_eq!(result, 2);
                    })} />
            </div>
        }
    }
}

#[gpui::test]
fn optional_notify_helper_preserves_events_and_exposes_notification_tradeoffs(
    cx: &mut TestAppContext,
) {
    let handle = cx.add_window(|_, _| Counter::default());
    let entity = handle.root(cx).unwrap();
    let notifications = Rc::new(Cell::new(0));
    let observed = notifications.clone();
    let _subscription =
        cx.update(|cx| cx.observe(&entity, move |_, _| observed.set(observed.get() + 1)));

    for (id, expected_notifications) in [
        ("native", 1),
        ("assisted", 2),
        ("assisted", 3),
        ("quiet", 3),
        ("batched", 4),
    ] {
        cx.update_window(handle.into(), |_, window, cx| {
            window.render_frame(cx);
            window.click(id, cx);
        })
        .unwrap();
        assert_eq!(notifications.get(), expected_notifications, "{id}");
    }

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.right_click("mouse", cx);
    })
    .unwrap();
    assert_eq!(notifications.get(), 5);
    handle
        .read_with(cx, |view, _| {
            assert_eq!(view.native, 1);
            assert_eq!(view.assisted, 3);
            assert_eq!(view.mouse, 1);
            assert_eq!(view.quiet, 1);
            assert_eq!(view.batched, 2);
            assert_eq!(view.label, "clicked");
        })
        .unwrap();
}
