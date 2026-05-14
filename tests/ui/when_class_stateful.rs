use gpui_rsx::rsx;

fn main() {
    let active = true;
    let _el = rsx! { <div whenClass={(active, "overflow-scroll")} /> };
}
