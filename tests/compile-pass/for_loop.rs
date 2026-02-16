use gpui_rsx::rsx;

fn main() {
    let items = vec![1, 2, 3];
    let _ = rsx! {
        <div>
            {for item in &items {
                <div>{*item}</div>
            }}
        </div>
    };
}
