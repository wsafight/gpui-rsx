use gpui_rsx::rsx;

fn main() {
    rsx! {
        <div when={true}>"Content"</div>
    };
}
