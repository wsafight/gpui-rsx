use zopra_gpui_view::rsx;

fn main() {
    let _el = rsx! {
        <div groupDragOver={("items", |style| style.opacity(0.75))} />
    };
}
