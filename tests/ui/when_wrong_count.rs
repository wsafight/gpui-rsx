use gpui_rsx::rsx;

fn main() {
    let _el = rsx! {
        <div when={(true, |el| el, "extra")} />
    };
}
