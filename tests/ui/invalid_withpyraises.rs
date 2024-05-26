use pyo3_testing::with_py_raises;

fn test_raises() {
    with_py_raises!(PyTypeError { () });
}

fn main() {}
