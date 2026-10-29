//! Compare generated classes with real GPUI builder chains, including existing styles.
use gpui::{div, prelude::*, px, rgb, svg};
use gpui_rsx::{rsx, rsx_strict, rsx_tw};

fn style(el: &mut impl Styled) -> String {
    format!("{:?}", el.style())
}

#[test]
fn dynamic_styles_match_gpui_and_preserve_existing_refinements() {
    let classes = "flex items-center gap-4 p-2.5 bg-blue-500 w-[123px] opacity-75";
    let mut actual = rsx! { <div class="border-2 rounded-lg" class={classes} /> };
    let mut expected = div()
        .border_2()
        .rounded_lg()
        .flex()
        .items_center()
        .gap(px(4.0))
        .p(px(2.5))
        .bg(rgb(0x3b82f6))
        .w(px(123.0))
        .opacity(0.75);
    assert_eq!(style(&mut actual), style(&mut expected));
}

#[test]
fn tailwind_and_multiple_element_types_share_the_runtime() {
    let classes = "gap-4 p-2.5 w-8";
    let mut actual = rsx_tw! { <div class={classes} /> };
    let mut expected = div().gap(px(16.0)).p(px(10.0)).w(px(32.0));
    assert_eq!(style(&mut actual), style(&mut expected));

    let classes = "size-8 text-red-500";
    let mut svg = rsx_tw! { <svg class={classes} /> };
    let mut expected = gpui::svg().size(px(32.0)).text_color(rgb(0xef4444));
    assert_eq!(style(&mut svg), style(&mut expected));
}

#[test]
fn callbacks_stay_at_each_call_site_and_empty_values_preserve_styles() {
    let classes = "flex unknown bg-brand hover:flex";
    let mut unknown = Vec::new();
    let mut actual = rsx_strict! {
        <div class={classes} onUnknownClass={|class: &str| unknown.push(class.to_owned())} />
    };
    assert_eq!(unknown, ["unknown", "bg-brand", "hover:flex"]);
    assert_eq!(style(&mut actual), style(&mut div().flex()));

    for classes in ["", " \t\n"] {
        let mut actual = rsx! { <div class="border-2" class={classes} /> };
        assert_eq!(style(&mut actual), style(&mut div().border_2()));
    }
}

#[test]
fn new_dynamic_tailwind_values_match_static_builder_calls() {
    let classes = "text-[14px] rounded-[4px] leading-6 bg-red-500/50";
    let mut actual = rsx_tw! { <div class={classes} /> };
    let mut expected = div()
        .text_size(px(14.0))
        .rounded(px(4.0))
        .line_height(px(24.0))
        .bg(gpui::rgba(0xef444480));
    assert_eq!(style(&mut actual), style(&mut expected));
}

#[test]
fn format_templates_preserve_evaluation_and_style_order() {
    use std::cell::Cell;
    struct Value<'a>(&'a Cell<usize>);
    impl std::fmt::Display for Value<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.set(self.0.get() + 1);
            f.write_str("8 bg-blue-500")
        }
    }
    let calls = Cell::new(0);
    let mut actual = rsx! { <div class={format!("p-2 p-{} bg-red-500", Value(&calls))} /> };
    assert_eq!(calls.get(), 1);
    assert_eq!(
        style(&mut actual),
        style(&mut div().p(px(8.0)).bg(rgb(0xef4444)))
    );
    let n = 4;
    let width = 2;
    let mut actual = rsx! { <div class={format!("p-{n:0width$} bg-red-500")} /> };
    assert_eq!(
        style(&mut actual),
        style(&mut div().p(px(4.0)).bg(rgb(0xef4444)))
    );
}

#[test]
fn format_templates_keep_unknown_edge_classes_at_runtime() {
    let mut unknown = Vec::new();
    let n = 4;
    let mut actual = rsx_strict! { <div class={format!("{{}} p-{} bg-red-500", n)} onUnknownClass={|c: &str| unknown.push(c.to_owned())} /> };
    assert_eq!(unknown, ["{}"]);
    assert_eq!(
        style(&mut actual),
        style(&mut div().p(px(4.0)).bg(rgb(0xef4444)))
    );
}
