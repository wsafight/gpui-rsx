use gpui_rsx::rsx;

fn main() {
    let _ = rsx! {
        <div>{for _ in 0..2 {
            <div onClick={|_, _, _| {}} />
            <div onClick={|_, _, _| {}} />
        }}</div>
    };
}
