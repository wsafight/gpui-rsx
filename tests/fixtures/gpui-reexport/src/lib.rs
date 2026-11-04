pub use engine as ui;
use ui::{prelude::*, *};

gpui_rsx::rsx_runtime!();

#[gpui_rsx::component]
pub struct Card {
    #[children]
    children: Vec<AnyElement>,
}

impl RenderOnce for Card {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        gpui_rsx::rsx! { <div>{...self.children}</div> }
    }
}

pub fn render(classes: &str) -> impl IntoElement {
    gpui_rsx::rsx! { <Card><div class={classes} onClick={|_, _, _| {}} /></Card> }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_markup_modes_follow_the_crate_reexport() {
        let _ = super::render("p-4 flex").into_any_element();
        let _ = gpui_rsx::rsx_tw! { <div onClick={|_, _, _| {}} /> };
        let _ = gpui_rsx::rsx_strict! { <div onClick={|_, _, _| {}} /> };
        let _ = gpui_rsx::rsx_permissive! { <div onClick={|_, _, _| {}} /> };
        let parts = gpui_rsx::rsx_fragment! { <><div /><div id="stateful" /></> };
        assert_eq!(parts.len(), 2);
        let empty = gpui_rsx::rsx! { <>{for _ in 0..0 {}}</> };
        assert!(empty.is_empty());
        let key = String::from("dynamic");
        let _ = gpui_rsx::rsx! { <Custom base={div()} key={key} onClick={|_, _, _| {}} /> };
    }

    #[test]
    fn helpers_resolve_the_reexport_from_nested_modules() {
        let mut view = 0usize;
        let mut cx = Context::new();
        let count = gpui_rsx::rsx_update!(&mut view, &mut cx, |view, _| {
            *view += 1;
            *view
        });
        assert_eq!(count, 1);
        let _ = gpui_rsx::rsx_listener!(cx, notify, |_, _: &(), _, _| {});
        let _ = gpui_rsx::rsx_listener!(cx, |_, _: &(), _, _| {});
    }
}
