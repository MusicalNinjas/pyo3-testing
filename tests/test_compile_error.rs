#![cfg(feature = "macros")]

#[cfg(not(target_arch = "wasm32"))] // Not possible to invoke compiler from wasm
#[test]
fn test_compile_errors_pyo3testing() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/invalid_pyo3imports.rs");
}
