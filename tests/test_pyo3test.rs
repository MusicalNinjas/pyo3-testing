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

#[pyfunction]
#[pyo3(name = "double")]
fn py_double(num: isize) -> isize {
    num * 2
}

#[pyfunction]
#[pyo3(name = "add")]
fn py_add(left: isize, right: isize) -> isize {
    left + right
}

#[pyfunction]
#[pyo3(name = "zero")]
fn py_zero() -> isize {
    0
}

#[pymodule]
#[pyo3(name = "adders")]
fn py_adders(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(py_addone, module)?)?;
    module.add_function(wrap_pyfunction!(py_double, module)?)?;
    module.add_function(wrap_pyfunction!(py_add, module)?)?;
    module.add_function(wrap_pyfunction!(py_zero, module)?)?;
    Ok(())
}

// This is how the test would be written WITHOUT using the pyo3test macro. This validates that
// adders.addone is correctly constructed.
#[test]
fn test_without_macro() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let sys = PyModule::import_bound(py, "sys").unwrap();
        let py_modules: Bound<'_, PyDict> =
            sys.getattr("modules").unwrap().downcast_into().unwrap();
        let py_adders_pymodule = unsafe { Bound::from_owned_ptr(py, py_adders::__pyo3_init()) };
        py_modules
            .set_item("adders", py_adders_pymodule)
            .expect("Failed to import adders");
        let adders = py_modules.get_item("adders").unwrap().unwrap();
        let addone = adders
            .getattr("addone")
            .expect("Failed to get addone function");
        let result: PyResult<isize> = match addone.call1((1_isize,)) {
            Ok(r) => r.extract(),
            Err(e) => Err(e),
        };
        let result = result.unwrap();
        let expected_result = 2_isize;
        assert_eq!(result, expected_result);
    });
}

// ... and this is how the test can be written using the pyo3test macro and pyo3import attribute
#[pyo3test]
#[pyo3import(py_adders: from adders import addone)]
fn test_simple_case() {
    let result: isize = addone!(1_isize);
    let expected_result = 2_isize;
    assert_eq!(result, expected_result);
}

#[pyo3test]
#[pyo3import(py_adders: from adders import double)]
#[pyo3import(py_adders: from adders import addone)]
fn test_multiple_imports() {
    let result: isize = addone!(1);
    let result: isize = double!(result);
    assert_eq!(result, 4_isize);
}

#[pyo3test]
#[pyo3import(py_adders: import adders)]
fn test_import_module_only() {
    let result: isize = adders
        .getattr("addone")
        .unwrap()
        .call1((1_isize,))
        .unwrap()
        .extract()
        .unwrap();
    let expected_result = 2_isize;
    assert_eq!(result, expected_result);
}

#[pyo3test]
#[pyo3import(py_adders: import adders)]
#[pyo3import(py_adders: from adders import double)]
fn test_mixed_import_types() {
    let result: isize = adders
        .getattr("addone")
        .unwrap()
        .call1((1_isize,))
        .unwrap()
        .extract()
        .unwrap();
    let result: isize = double!(result);
    assert_eq!(result, 4_isize);
}

#[pyo3test]
fn test_no_imports() {
    let fun: Py<PyAny> = PyModule::from_code_bound(
        py,
        "def two():
            return 2
        ",
        "",
        "",
    )
    .unwrap()
    .getattr("two")
    .unwrap()
    .into();
    let result: isize = fun.call0(py).unwrap().extract(py).unwrap();
    assert_eq!(result, 2_isize)
}

#[pyo3test]
#[pyo3import(py_adders: from adders import add)]
fn test_multiple_args() {
    let result: isize = add!(1, 2);
    assert_eq!(result, 3)
}

#[pyo3test]
#[pyo3import(py_adders: from adders import zero)]
fn test_no_args() {
    let result: isize = zero!();
    assert_eq!(result, 0)
}

#[pyo3test]
#[pyo3import(py_adders: from adders import add)]
fn test_star_args() {
    let args = (1, 2);
    let result: isize = add!(*args);
    assert_eq!(result, 3)
}

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
