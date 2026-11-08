use gpui::component::{
    button::Button,
    input::{Input, InputState},
};
use gpui::test::TestWindowExt;
use gpui::{
    AppContext, Context, Entity, Focusable, Render, TestAppContext, Window, WindowOptions, div,
    prelude::*, px,
};
use gpui_rsx::{rsx, rsx_listener};

struct NativeControls {
    input: Entity<InputState>,
    saves: usize,
}

impl Render for NativeControls {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        rsx! {
            <div class="size-full flex flex-col gap-12 p-20">
                <Input base={Input::new(&self.input)} />
                <Button
                    base={Button::new("native-save")}
                    label="Save"
                    onClick={rsx_listener!(cx, notify, |view, _, _, _| {
                        view.saves += 1;
                    })}
                />
            </div>
        }
    }
}

#[gpui::test]
fn kit_input_keeps_unicode_text_and_focus_across_rsx_updates(cx: &mut TestAppContext) {
    cx.update(gpui::init);
    let (handle, view) = cx.update(|cx| {
        gpui::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| NativeControls {
                input: cx.new(|cx| InputState::new(window, cx)),
                saves: 0,
            })
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        let input = view.read(cx).input.clone();
        input.focus_handle(cx).focus(window, cx);
        window.render_frame(cx);
        window.input("排查中😀", cx);
        window.press("backspace", cx);
        assert_eq!(input.read(cx).value(), "排查中");
        window.input("🙂", cx);
        view.update(cx, |_, cx| cx.notify());
        window.render_frame(cx);
        assert!(input.focus_handle(cx).is_focused(window));
        window.click("native-save", cx);
        assert_eq!(view.read(cx).saves, 1);
        assert_eq!(input.read(cx).value(), "排查中🙂");
    })
    .unwrap();
}

#[gpui::test]
fn kit_input_handles_combining_marks_and_unicode_selection_replacement(cx: &mut TestAppContext) {
    cx.update(gpui::init);
    let (handle, view) = cx.update(|cx| {
        gpui::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| NativeControls {
                input: cx.new(|cx| InputState::new(window, cx)),
                saves: 0,
            })
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        let input = view.read(cx).input.clone();
        input.focus_handle(cx).focus(window, cx);
        window.render_frame(cx);
        window.input("A👩🏽‍💻é中", cx);
        for expected in ["A👩🏽‍💻é", "A👩🏽‍💻e", "A👩🏽‍💻"] {
            window.press("backspace", cx);
            assert_eq!(input.read(cx).value(), expected);
        }
        window.input("中文😀", cx);
        window.press("secondary-a", cx);
        window.input("替换🙂", cx);
        assert_eq!(input.read(cx).value(), "替换🙂");
        view.update(cx, |_, cx| cx.notify());
        window.render_frame(cx);
        assert!(input.focus_handle(cx).is_focused(window));
        assert_eq!(input.read(cx).value(), "替换🙂");
    })
    .unwrap();
}

#[gpui::test]
fn repeated_rsx_renders_do_not_duplicate_native_button_callbacks(cx: &mut TestAppContext) {
    cx.update(gpui::init);
    let (handle, view) = cx.update(|cx| {
        gpui::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| NativeControls {
                input: cx.new(|cx| InputState::new(window, cx)),
                saves: 0,
            })
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        let input = view.read(cx).input.clone();
        input.focus_handle(cx).focus(window, cx);
        window.render_frame(cx);
        window.input("保留内容😀", cx);
        for expected in 1..=12 {
            view.update(cx, |_, cx| cx.notify());
            window.render_frame(cx);
            window.click("native-save", cx);
            assert_eq!(view.read(cx).saves, expected);
            assert_eq!(view.read(cx).input.entity_id(), input.entity_id());
            assert_eq!(input.read(cx).value(), "保留内容😀");
        }
    })
    .unwrap();
}
