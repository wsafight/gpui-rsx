#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}

#[test]
fn compile_pass_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile-pass/*.rs");
}
