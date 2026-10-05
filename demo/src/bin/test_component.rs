use zopra_gpui_view::rsx_expand;

fn main() {
    let s = rsx_expand! {
        <div ref={my_ref} />
    };
    println!("{}", s);
}
