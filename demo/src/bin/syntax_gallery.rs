mod common;

use gpui_rsx_demo::syntax::SyntaxGallery;

fn main() {
    common::run(
        |cx| common::window(cx, "Syntax gallery · GPUI-RSX", 960.0, 760.0),
        |_| SyntaxGallery::default(),
    );
}
