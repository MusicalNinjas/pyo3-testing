# pyo3-testing Changelog

## v0.3.0 Add with_py_raises

- Added `with_py_raises!()`to emulate pytest's `with pytest.raises` context manager from python.
- Polished the documentation
- Moved the implementation of `#[pyo3test]`into a dedicated module to tidy up `lib.rs` and make the code easier to read and maintain

## v0.2.0 Add macros to call the wrapped functions
