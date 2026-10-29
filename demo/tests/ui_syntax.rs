//! Render RSX in GPUI's headless platform and dispatch real input events.
use gpui::test::{TestSupportExt, TestWindowExt};
use gpui::{
    AnyElement, AnyWindowHandle, App, AppContext, Bounds, Context, Element, ElementId, Pixels,
    Render, SharedString, TestAppContext, Window, WindowHandle, canvas, div, prelude::*, px,
};
use gpui_rsx::{component, rsx};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

type Build = Box<dyn FnMut(&mut Window, &mut App) -> AnyElement>;
struct Fixture(Build);
impl Render for Fixture {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        (self.0)(window, cx)
    }
}
fn mount(
    cx: &mut TestAppContext,
    build: impl FnMut(&mut Window, &mut App) -> AnyElement + 'static,
) -> WindowHandle<Fixture> {
    cx.add_window(|_, _| Fixture(Box::new(build)))
}
fn frame(
    cx: &mut TestAppContext,
    handle: impl Into<AnyWindowHandle>,
    run: impl FnOnce(&mut Window, &mut App),
) {
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        run(window, cx);
    })
    .unwrap();
}

struct ClickView {
    clicks: usize,
}

#[gpui::test]
fn auto_id_events_work_and_keep_their_identity_across_frames(cx: &mut TestAppContext) {
    let clicks = Rc::new(Cell::new(0));
    let ids = Rc::new(RefCell::new(Vec::new()));
    let (sink, captured) = (clicks.clone(), ids.clone());
    let handle = mount(cx, move |_, _| {
        let sink = sink.clone();
        let element = rsx! { <Custom base={div()} class="w-100 h-40" onClick={move |_, _, _| sink.set(sink.get() + 1)} /> };
        captured.borrow_mut().push(Element::id(&element).unwrap());
        element.test_support().into_any_element()
    });
    frame(cx, handle, |window, cx| {
        let id = ids.borrow()[0].clone();
        window.click(id.clone(), cx);
        window.click(id, cx);
    });
    assert_eq!(clicks.get(), 2);
    assert!(ids.borrow().len() >= 2);
    assert!(ids.borrow().windows(2).all(|ids| ids[0] == ids[1]));
}

#[gpui::test]
fn conditional_attributes_change_the_rendered_bounds(cx: &mut TestAppContext) {
    let selected = Rc::new(Cell::new(false));
    let condition = selected.clone();
    let handle = mount(cx, move |_, _| {
        let selected = condition.get();
        rsx! { <div id="sized" class="h-20 w-10"
        when={(selected, |el| el.w(px(20.)))}
        whenSome={(selected.then_some(30.), |el, n| el.w(px(n)))}
        whenClass={(selected, "w-40")} /> }
        .test_support()
        .into_any_element()
    });
    frame(cx, handle, |window, _| {
        assert_eq!(window.find("sized").bounds().size.width, px(10.))
    });
    selected.set(true);
    frame(cx, handle, |window, _| {
        assert_eq!(window.find("sized").bounds().size.width, px(40.))
    });
}

#[component]
fn Marker(name: SharedString, order: Rc<RefCell<Vec<SharedString>>>) -> impl IntoElement {
    order.borrow_mut().push(name.clone());
    rsx! { <div id={name} class="h-10" /> }.test_support()
}

#[gpui::test]
fn nested_fragments_spread_and_loops_render_in_source_order(cx: &mut TestAppContext) {
    let order = Rc::new(RefCell::new(Vec::new()));
    let recorded = order.clone();
    let handle = mount(cx, move |_, _| {
        let order = recorded.clone();
        let spread = rsx! { <><Marker name="spread" order={order.clone()} /></> };
        let roots: Vec<AnyElement> = rsx! {
            <>
                <Marker name="first" order={order.clone()} />
                <><></><Marker name="nested" order={order.clone()} /></>
                {...spread}
                {for (i, _) in [0, 1].iter().enumerate() {
                    <>
                        <Marker name={format!("row-{i}")} order={order.clone()} />
                        {for j in 0..1 { <Marker name={format!("detail-{i}-{j}")} order={order.clone()} /> }}
                    </>
                }}
                <Marker name="last" order={order.clone()} />
            </>
        };
        div().flex().flex_col().children(roots).into_any_element()
    });
    order.borrow_mut().clear();
    frame(cx, handle, |window, _| {
        assert!(window.find("nested").bounds().origin.y < window.find("spread").bounds().origin.y);
    });
    let expected = [
        "first",
        "nested",
        "spread",
        "row-0",
        "detail-0-0",
        "row-1",
        "detail-1-0",
        "last",
    ];
    assert_eq!(
        order
            .borrow()
            .iter()
            .map(|name| name.as_ref())
            .collect::<Vec<&str>>(),
        expected
    );
}

#[gpui::test]
fn keyed_loop_ids_follow_values_when_rows_reorder(cx: &mut TestAppContext) {
    let rows = Rc::new(RefCell::new(vec![3u64, 1, 2]));
    let data = rows.clone();
    let ids = Rc::new(RefCell::new(Vec::<ElementId>::new()));
    let seen = ids.clone();
    let handle = mount(cx, move |_, _| {
        seen.borrow_mut().clear();
        let elements = rsx! { <>{for key in data.borrow().iter().copied() {
            {{
                let element = rsx! { <div key={key} class="h-10" onClick={|_, _, _| {}} /> };
                seen.borrow_mut().push(Element::id(&element).unwrap());
                element.test_support()
            }}
        }}</> };
        div()
            .flex()
            .flex_col()
            .children(elements)
            .into_any_element()
    });
    frame(cx, handle, |_, _| {});
    let before = ids.borrow().clone();
    rows.borrow_mut().reverse();
    frame(cx, handle, |window, _| {
        assert!(
            window.find(before[2].clone()).bounds().origin.y
                < window.find(before[0].clone()).bounds().origin.y
        );
    });
    let mut reversed = before;
    reversed.reverse();
    assert_eq!(*ids.borrow(), reversed);
}

#[component]
fn ContextProbe(
    calls: Rc<Cell<usize>>,
    #[context] window: &mut Window,
    #[context] cx: &mut App,
) -> impl IntoElement {
    assert!(window.viewport_size().width > px(0.));
    let _: &mut App = cx;
    calls.set(calls.get() + 1);
    div()
}

#[gpui::test]
fn function_components_run_at_render_time_with_injected_context(cx: &mut TestAppContext) {
    let calls = Rc::new(Cell::new(0));
    let element = rsx! { <ContextProbe calls={calls.clone()} /> };
    assert_eq!(calls.get(), 0);
    let mut element = Some(element);
    let handle = mount(cx, move |_, _| {
        div()
            .when_some(element.take(), |el, child| el.child(child))
            .into_any_element()
    });
    frame(cx, handle, |_, _| {});
    assert_eq!(calls.get(), 1);
}

#[gpui::test]
fn canvas_passes_prepaint_state_and_bounds_to_paint(cx: &mut TestAppContext) {
    let events = Rc::new(RefCell::new(Vec::new()));
    let captured = events.clone();
    let handle = mount(cx, move |_, _| {
        let prepaint = captured.clone();
        let paint = captured.clone();
        rsx! {
            <canvas class="w-80 h-24"
                prepaint={move |bounds: Bounds<Pixels>, _, _| { prepaint.borrow_mut().push("prepaint"); bounds.size }}
                paint={move |bounds, size, _, _| {
                    assert_eq!(bounds.size, size);
                    assert_eq!(size.width, px(80.));
                    assert_eq!(size.height, px(24.));
                    paint.borrow_mut().push("paint");
                }} />
        }.into_any_element()
    });
    events.borrow_mut().clear();
    frame(cx, handle, |_, _| {});
    assert_eq!(*events.borrow(), ["prepaint", "paint"]);
}

#[gpui::test]
fn gallery_renders_the_actual_components_and_switches_conditions(cx: &mut TestAppContext) {
    use gpui_rsx_demo::syntax::SyntaxGallery;
    cx.update(gpui::init);
    let (handle, view) = cx.update(|cx| {
        gpui::open_window(Default::default(), cx, |_, cx| {
            cx.new(|_| SyntaxGallery::default())
        })
        .unwrap()
    });
    frame(cx, handle, |window, cx| window.click("syntax-toggle", cx));
    view.read_with(cx, |view, _| {
        assert!(view.selected);
        assert_eq!(view.clicks, 1);
    });
    frame(cx, handle, |window, cx| window.click("syntax-toggle", cx));
    view.read_with(cx, |view, _| {
        assert!(!view.selected);
        assert_eq!(view.clicks, 2);
    });
}

#[gpui::test]
fn tuple_mouse_events_and_focused_key_events_dispatch(cx: &mut TestAppContext) {
    let mouse = Rc::new(Cell::new(0));
    let keys = Rc::new(Cell::new(0));
    let focus = cx.update(|cx| cx.focus_handle());
    let (mouse_sink, key_sink, tracked) = (mouse.clone(), keys.clone(), focus.clone());
    let handle = mount(cx, move |_, _| {
        let (mouse, keys) = (mouse_sink.clone(), key_sink.clone());
        rsx! { <div id="events" class="w-100 h-40" test_support trackFocus={&tracked}
        onMouseDown={(gpui::MouseButton::Right, move |_, _, _| mouse.set(mouse.get() + 1))}
        onKeyDown={move |_, _, _| keys.set(keys.get() + 1)} /> }
        .into_any_element()
    });
    frame(cx, handle, |window, cx| {
        window.right_click("events", cx);
        window.focus(&focus, cx);
        window.press("enter", cx);
    });
    assert_eq!(mouse.get(), 1);
    assert_eq!(keys.get(), 1);
}

#[gpui::test]
fn hover_class_changes_layout_through_gpui_state_dispatch(cx: &mut TestAppContext) {
    let handle = mount(cx, |_, _| {
        rsx! {
            <div id="hovered" class="w-40 h-20" hoverClass="w-60" />
        }
        .test_support()
        .into_any_element()
    });
    frame(cx, handle, |window, cx| {
        assert_eq!(window.find("hovered").bounds().size.width, px(40.));
        window.hover("hovered", cx);
        assert_eq!(window.find("hovered").bounds().size.width, px(60.));
    });
}

impl Render for ClickView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <button id="click" class="w-100 h-40" onClick={cx.listener(|view, _, _, cx| {
                view.clicks += 1;
                cx.notify();
            })} />
        }
        .test_support()
    }
}

#[gpui::test]
fn click_alias_dispatches_to_the_view(cx: &mut TestAppContext) {
    let handle = cx.add_window(|_, _| ClickView { clicks: 0 });
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(window.find("click").bounds().size.width, gpui::px(100.));
        window.click("click", cx);
    })
    .unwrap();
    handle
        .read_with(cx, |view, _| assert_eq!(view.clicks, 1))
        .unwrap();
}
