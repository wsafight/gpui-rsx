use gpui_rsx::rsx;

mod ui {
    #[allow(non_snake_case)]
    pub fn TaskCard() {}

    #[allow(non_snake_case)]
    pub fn OtherCard() {}
}

fn main() {
    let _el = rsx! {
        <ui::TaskCard>
            {"content"}
        </ui::OtherCard>
    };
}
