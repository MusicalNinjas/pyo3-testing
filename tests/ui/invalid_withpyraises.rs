use pyo3_testing::with_py_raises;

fn test_missing_comma() {
    with_py_raises!(PyTypeError { () });
}

fn test_missing_braces() {
    with_py_raises!(PyTypeError, Ok(()));
}

fn main() {}
