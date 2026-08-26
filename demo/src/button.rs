use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx};

#[component]
pub fn FunctionalButton(label: String, is_active: bool) -> impl IntoElement {
    let bg_color = if is_active { gpui::rgb(0x2563eb) } else { gpui::rgb(0x475569) };
    
    rsx! {
        <div class="px-4 py-2" bg={bg_color}>
            {label}
        </div>
    }
}
