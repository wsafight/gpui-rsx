//! Every negatable length family against GPUI's real layout engine.
//!
//! The documentation promises exactly what these tests pin: negative margin, padding
//! and gap shift or overlap content, negative sizes are clamped to zero by layout, and
//! negative `min-*` has no effect. Two 20x20 children `a` and `b` sit in a 100x100 flex
//! container; each case lists their expected `[x, y, width, height]`.
use gpui::test::{TestSupportExt, TestWindowExt};
use gpui::{
    AnyElement, AnyWindowHandle, App, AppContext, Bounds, Context, Pixels, Render, TestAppContext,
    Window, WindowHandle, div, prelude::*, px,
};
use gpui_rsx::rsx;

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

type Rect = [f32; 4];

fn rect(bounds: Bounds<Pixels>) -> Rect {
    [
        f32::from(bounds.origin.x),
        f32::from(bounds.origin.y),
        f32::from(bounds.size.width),
        f32::from(bounds.size.height),
    ]
}

fn render(
    cx: &mut TestAppContext,
    build: impl FnMut(&mut Window, &mut App) -> AnyElement + 'static,
) -> (Rect, Rect) {
    let handle = mount(cx, build);
    let mut result = None;
    cx.update_window(AnyWindowHandle::from(handle), |_, window, cx| {
        window.render_frame(cx);
        result = Some((
            rect(window.find("a").bounds()),
            rect(window.find("b").bounds()),
        ));
    })
    .unwrap();
    result.unwrap()
}

/// Lays out `a` and `b` (both with `child` classes) inside `flex {dir} w-100 h-100 {root}`
/// using dynamic class strings, which exercises the runtime matcher.
fn layout(cx: &mut TestAppContext, dir: &str, root: &str, child: &str) -> (Rect, Rect) {
    let (root, child) = (format!("flex {dir} w-100 h-100 {root}"), child.to_string());
    render(cx, move |_, _| {
        let (a_class, b_class, root_class) = (child.clone(), child.clone(), root.clone());
        let a = rsx! { <div id="a" class={a_class} /> }.test_support();
        let b = rsx! { <div id="b" class={b_class} /> }.test_support();
        rsx! { <div id="root" class={root_class}>{a}{b}</div> }
            .test_support()
            .into_any_element()
    })
}

struct Case {
    class: &'static str,
    dir: &'static str,
    a: Rect,
    b: Rect,
}

const COL: &str = "flex-col";
const ROW: &str = "flex-row";
const BOX: &str = "w-20 h-20";

fn check(cx: &mut TestAppContext, cases: &[Case], root_of: impl Fn(&str) -> String) {
    for case in cases {
        let (root, child) = match root_of(case.class) {
            root if root.is_empty() => (String::new(), format!("{BOX} {}", case.class)),
            root => (root, BOX.to_string()),
        };
        let (a, b) = layout(cx, case.dir, &root, &child);
        assert_eq!(a, case.a, "{} in {}: a", case.class, case.dir);
        assert_eq!(b, case.b, "{} in {}: b", case.class, case.dir);
    }
}

#[gpui::test]
fn negative_margins_overlap_neighbours(cx: &mut TestAppContext) {
    // The class is on both children, so each one's own margin applies as well.
    #[rustfmt::skip]
    let cases = [
        Case { class: "-m-5",  dir: COL, a: [-5., -5., 20., 20.], b: [-5., 5., 20., 20.] },
        Case { class: "-mx-5", dir: COL, a: [-5., 0., 20., 20.],  b: [-5., 20., 20., 20.] },
        Case { class: "-my-5", dir: COL, a: [0., -5., 20., 20.],  b: [0., 5., 20., 20.] },
        Case { class: "-mt-5", dir: COL, a: [0., -5., 20., 20.],  b: [0., 10., 20., 20.] },
        Case { class: "-mb-5", dir: COL, a: [0., 0., 20., 20.],   b: [0., 15., 20., 20.] },
        Case { class: "-ml-5", dir: COL, a: [-5., 0., 20., 20.],  b: [-5., 20., 20., 20.] },
        // A right margin does not move anything on the cross axis of a column.
        Case { class: "-mr-5", dir: COL, a: [0., 0., 20., 20.],   b: [0., 20., 20., 20.] },
        Case { class: "-m-5",  dir: ROW, a: [-5., -5., 20., 20.], b: [5., -5., 20., 20.] },
        Case { class: "-ml-5", dir: ROW, a: [-5., 0., 20., 20.],  b: [10., 0., 20., 20.] },
        Case { class: "-mr-5", dir: ROW, a: [0., 0., 20., 20.],   b: [15., 0., 20., 20.] },
        Case { class: "-mt-5", dir: ROW, a: [0., -5., 20., 20.],  b: [20., -5., 20., 20.] },
        Case { class: "-mb-5", dir: ROW, a: [0., 0., 20., 20.],   b: [20., 0., 20., 20.] },
    ];
    check(cx, &cases, |_| String::new());
}

#[gpui::test]
fn negative_padding_shifts_content_outward(cx: &mut TestAppContext) {
    // `-pb-*` and `-pr-*` only change the far edge, so no child moves.
    #[rustfmt::skip]
    let cases = [
        Case { class: "-p-5",  dir: COL, a: [-5., -5., 20., 20.], b: [-5., 15., 20., 20.] },
        Case { class: "-px-5", dir: COL, a: [-5., 0., 20., 20.],  b: [-5., 20., 20., 20.] },
        Case { class: "-py-5", dir: COL, a: [0., -5., 20., 20.],  b: [0., 15., 20., 20.] },
        Case { class: "-pt-5", dir: COL, a: [0., -5., 20., 20.],  b: [0., 15., 20., 20.] },
        Case { class: "-pl-5", dir: COL, a: [-5., 0., 20., 20.],  b: [-5., 20., 20., 20.] },
        Case { class: "-pb-5", dir: COL, a: [0., 0., 20., 20.],   b: [0., 20., 20., 20.] },
        Case { class: "-pr-5", dir: COL, a: [0., 0., 20., 20.],   b: [0., 20., 20., 20.] },
        Case { class: "-p-5",  dir: ROW, a: [-5., -5., 20., 20.], b: [15., -5., 20., 20.] },
        Case { class: "-px-5", dir: ROW, a: [-5., 0., 20., 20.],  b: [15., 0., 20., 20.] },
        Case { class: "-pl-5", dir: ROW, a: [-5., 0., 20., 20.],  b: [15., 0., 20., 20.] },
        Case { class: "-pt-5", dir: ROW, a: [0., -5., 20., 20.],  b: [20., -5., 20., 20.] },
        Case { class: "-pr-5", dir: ROW, a: [0., 0., 20., 20.],   b: [20., 0., 20., 20.] },
    ];
    check(cx, &cases, str::to_string);
}

#[gpui::test]
fn negative_gap_pulls_siblings_together_along_its_axis(cx: &mut TestAppContext) {
    // `gap-x` only acts along a row and `gap-y` only along a column.
    #[rustfmt::skip]
    let cases = [
        Case { class: "-gap-5",   dir: COL, a: [0., 0., 20., 20.], b: [0., 15., 20., 20.] },
        Case { class: "-gap-y-5", dir: COL, a: [0., 0., 20., 20.], b: [0., 15., 20., 20.] },
        Case { class: "-gap-x-5", dir: COL, a: [0., 0., 20., 20.], b: [0., 20., 20., 20.] },
        Case { class: "-gap-5",   dir: ROW, a: [0., 0., 20., 20.], b: [15., 0., 20., 20.] },
        Case { class: "-gap-x-5", dir: ROW, a: [0., 0., 20., 20.], b: [15., 0., 20., 20.] },
        Case { class: "-gap-y-5", dir: ROW, a: [0., 0., 20., 20.], b: [20., 0., 20., 20.] },
    ];
    check(cx, &cases, str::to_string);
}

#[gpui::test]
fn negative_sizes_are_clamped_to_zero_and_min_is_ignored(cx: &mut TestAppContext) {
    // Sizing classes replace the matching base dimension, so the child is spelled out.
    // Both children carry the class, in a column and in a row.
    for (class, child, a_size) in [
        ("-w-5", "-w-5 h-20", [0., 20.]),
        ("-w-1/2", "-w-1/2 h-20", [0., 20.]),
        ("-h-5", "-h-5 w-20", [20., 0.]),
        ("-size-5", "-size-5", [0., 0.]),
        ("-max-w-5", "w-20 h-20 -max-w-5", [0., 20.]),
        ("-max-h-5", "w-20 h-20 -max-h-5", [20., 0.]),
        ("-min-w-5", "w-20 h-20 -min-w-5", [20., 20.]),
        ("-min-h-5", "w-20 h-20 -min-h-5", [20., 20.]),
    ] {
        for dir in [COL, ROW] {
            let (a, b) = layout(cx, dir, "", child);
            assert_eq!(&a[2..], &a_size, "{class} in {dir}: a size");
            assert_eq!(&b[2..], &a_size, "{class} in {dir}: b size");
        }
    }
}

#[gpui::test]
fn presets_arbitrary_and_rem_values_match_gpui_units(cx: &mut TestAppContext) {
    // Static literals use GPUI's `m_neg_px()` / `m_neg_0p5()` preset methods.
    let (a, b) = render(cx, |_, _| {
        let a = rsx! { <div id="a" class="w-20 h-20 -mt-px" /> }.test_support();
        let b = rsx! { <div id="b" class="w-20 h-20 -mt-0p5" /> }.test_support();
        rsx! { <div id="root" class="flex flex-col w-100 h-100">{a}{b}</div> }
            .test_support()
            .into_any_element()
    });
    assert_eq!(a[1], -1.0, "-mt-px");
    assert_eq!(b[1], 17.0, "-mt-0p5 after a's bottom edge at 19");

    // Arbitrary pixel and rem values (1rem = 16px) keep their sign.
    let (a, _) = layout(cx, COL, "", "w-20 h-20 -mt-[7px]");
    assert_eq!(a[1], -7.0);
    let (a, _) = layout(cx, COL, "", "w-20 h-20 -mt-[2rem]");
    assert_eq!(a[1], -32.0);
}
