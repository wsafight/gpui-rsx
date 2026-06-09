//! Compile-fail tests for macro diagnostics.

#[test]
fn compile_fail_diagnostics() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}

#[test]
fn compile_pass_gpui_class_mappings() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/*.rs");
}
