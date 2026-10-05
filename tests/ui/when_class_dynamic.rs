use zopra_gpui_view::rsx;

fn main() {
    let active = true;
    let classes = "text-white";
    let _el = rsx! { <div whenClass={(active, classes)} /> };
}
