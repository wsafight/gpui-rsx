gpui_rsx::rsx_runtime!(ui);

pub fn render(classes: &str) -> impl ui::IntoElement {
    use ui::{prelude::*, *};
    gpui_rsx::rsx! { <div class="bg-brand" class={classes}><svg class={classes}/></div> }
}

#[cfg(test)]
mod tests {
    #[test]
    fn library_unit_tests_use_the_local_helper() {
        let _ = super::render("gap-4 flex");
    }
}
