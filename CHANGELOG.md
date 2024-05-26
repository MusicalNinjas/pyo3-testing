# pyo3-testing Changelog

## v0.3.2 Extend readme to include details on with_py_raises

## v0.3.1 Improve error messages for with_py_raises

- Added custom compiler errors for misformed with_py_raises!() invocations
- Fixed documented examples

## v0.3.0 Add with_py_raises

- Added `with_py_raises!()`to emulate pytest's `with pytest.raises` context manager from python.
- Polished the documentation
- Moved the implementation of `#[pyo3test]`into a dedicated module to tidy up `lib.rs` and make the code easier to read and maintain

## v0.2.0 Add macros to call the wrapped functions
