#![deny(elided_lifetimes_in_paths)]

use gpui::Context;
use gpui_rsx::rsx_update;

fn replace<Value: 'static>(view: &mut Value, cx: &mut Context<'_, Value>, next: Value) -> Value {
    rsx_update!(view, cx, move |view, _| std::mem::replace(view, next))
}

fn main() {
    let mut view = String::from("old");
    let mut context = Context::new();
    assert_eq!(replace(&mut view, &mut context, String::from("new")), "old");
    let mut observed = Vec::new();
    let result: Result<(), &str> = rsx_update!(&mut view, &mut context, |view, cx| {
        let _ = cx;
        observed.push(view.len());
        Err("unchanged")?;
        Ok(())
    },);
    assert_eq!(result, Err("unchanged"));
    assert_eq!(observed, [3]);
}
