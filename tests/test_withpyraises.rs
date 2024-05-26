use pyo3::{exceptions::PyTypeError, prelude::*, types::PyDict};
use pyo3_testing::{pyo3test, with_py_raises};

// The example from the Guide ...
fn o3_addone(num: isize) -> isize {
    num + 1
}

#[pyfunction]
#[pyo3(name = "addone")]
fn py_addone(num: isize) -> isize {
    o3_addone(num)
}

#[pymodule]
#[pyo3(name = "adders")]
fn py_adders(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(py_addone, module)?)?;
    Ok(())
}

/// This is how the test would be written without the macro
#[pyo3test]
#[allow(unused_macros)]
#[pyo3import(py_adders: from adders import addone)]
fn test_raises_validate_approach() {
    match { addone.call1(("4",)) } {
        Ok(_) => panic!("No Error"),
        Err(error) if error.is_instance_of::<PyTypeError>(py) => return (),
        Err(_) => panic!("Wrong Error"),
    };
}

#[pyo3test]
#[allow(unused_macros)]
#[pyo3import(py_adders: from adders import addone)]
fn test_raises() {
    with_py_raises!(PyTypeError, { addone.call1(("4",)) }); //can't use `let result =` here
}

#[test]
fn test_compile_errors_pyo3testing() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/invalid_withpyraises.rs");
}
