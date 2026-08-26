use gpui_rsx::rsx_expand;

fn main() {
    let s = rsx_expand! {
        <div ref={my_ref} />
    };
    println!("{}", s);
}
