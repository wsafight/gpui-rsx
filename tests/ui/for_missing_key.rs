use gpui_rsx::rsx;

struct MockElement;

fn div() -> MockElement {
    MockElement
}

impl MockElement {
    fn children<I>(self, _: I) -> Self {
        self
    }
}

fn main() {
    let items = [1, 2, 3];
    let handler = |_: (), _: ()| {};

    let _el = rsx! {
        <ul>
            {for item in items {
                <li onClick={handler}>{item}</li>
            }}
        </ul>
    };
}
