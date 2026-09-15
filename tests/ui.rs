#![cfg(feature = "derive")]

#[test]
fn rejects_invalid_derives_and_conversions() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}
