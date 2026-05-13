use gpui_rsx::rsx;

#[derive(Debug)]
struct MockElement;

#[derive(Clone, Copy, Debug)]
struct FontWeight(f32);

impl FontWeight {
    const THIN: FontWeight = FontWeight(100.0);
    const EXTRA_LIGHT: FontWeight = FontWeight(200.0);
    const LIGHT: FontWeight = FontWeight(300.0);
    const NORMAL: FontWeight = FontWeight(400.0);
    const MEDIUM: FontWeight = FontWeight(500.0);
    const SEMIBOLD: FontWeight = FontWeight(600.0);
    const BOLD: FontWeight = FontWeight(700.0);
    const EXTRA_BOLD: FontWeight = FontWeight(800.0);
    const BLACK: FontWeight = FontWeight(900.0);
}

fn div() -> MockElement {
    MockElement
}

impl MockElement {
    fn font_weight(self, _: FontWeight) -> Self {
        self
    }

    fn text_3xl(self) -> Self {
        self
    }

    fn child<T>(self, _: T) -> Self {
        self
    }
}

fn main() {
    let _class = rsx! {
        <div class="font-thin font-extralight font-light font-normal font-medium font-semibold font-bold font-extrabold font-black" />
    };
    let _preset = rsx! { <h1 styled>{"Title"}</h1> };
}
