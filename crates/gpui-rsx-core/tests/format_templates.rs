use gpui_rsx_core::{ClassMode, expand_str};
#[test]
fn static_edges_expand_and_middle_tokens_preserve_dynamic_order() {
    let source = r#"<div class={format!("p-2 p-{} bg-red-500", n)} />"#;
    let tokens = expand_str(source, ClassMode::PERMISSIVE)
        .unwrap()
        .to_string();
    assert!(tokens.contains(". p (px (2f32))"), "{tokens}");
    assert!(tokens.contains(". bg (rgb (15680580u32))"), "{tokens}");
    assert_eq!(tokens.matches("format !").count(), 1);
    let source = r#"<div class={format!("p-{} active:bg-red-500", n)} />"#;
    let tokens = expand_str(source, ClassMode::PERMISSIVE)
        .unwrap()
        .to_string();
    assert!(tokens.contains(". id ("));
    assert!(tokens.contains(". active ("));
}
