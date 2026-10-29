mod common;

use gpui::WindowOptions;
use gpui_rsx_demo::syntax::SyntaxGallery;

fn main() {
    common::run(|_| WindowOptions::default(), |_| SyntaxGallery::default());
}
