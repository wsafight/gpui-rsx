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
    let _el = rsx! {
        <div>
            {for item in items {
                <div restrictScrollToAxis>{item}</div>
            }}
        </div>
    };
}
