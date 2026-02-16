use gpui_rsx::rsx;

fn main() {
    let _ = rsx! {
        <>
            <div>"First"</div>
            <div>"Second"</div>
        </>
    };
}
