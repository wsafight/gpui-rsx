use gpui_rsx::rsx;

fn main() {
    let count = 42;
    let _ = rsx! {
        <div>{format!("Count: {}", count)}</div>
    };
}
