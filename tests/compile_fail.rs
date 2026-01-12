#[rustversion::nightly(2026-01-09)]
#[test]
fn compile_test() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}

#[rustversion::not(nightly(2026-01-09))]
#[test]
fn invalid_rust_version() {
    // error messages may vary across compiler versions
    panic!("not the expected version of rust compiler");
}
