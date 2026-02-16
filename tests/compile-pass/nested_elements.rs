use gpui_rsx::rsx;

fn main() {
    let _ = rsx! {
        <div>
            <span>"Nested"</span>
            <div>"Content"</div>
        </div>
    };
}
