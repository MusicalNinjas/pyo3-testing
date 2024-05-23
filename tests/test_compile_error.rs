#[test]
fn test_compile_errors_pyo3testing() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/invalid_pyo3imports.rs");
}
