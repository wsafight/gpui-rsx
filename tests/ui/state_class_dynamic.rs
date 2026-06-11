use gpui_rsx::rsx;

fn main() {
    let classes = "bg-blue-500";
    let _el = rsx! { <div hoverClass={classes} /> };
}
