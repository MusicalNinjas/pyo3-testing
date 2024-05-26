//! Simplifies testing of `#[pyo3function]`s by enabling tests to be condensed to:
//!
//! ```ignore # expands to include #[test] so gets ignored anyway
//! #[pyo3test]
//! #[pyo3import(py_adders: from adders import addone)]
//! fn test_pyo3test_simple_case() {
//!     let result: isize = addone!(1);
//!     assert_eq!(result, 2);
//! }
//! ```

mod pyo3test;
mod withpyraises;

use pyo3test::impl_pyo3test;
use withpyraises::impl_with_py_raises;

use proc_macro::TokenStream as TokenStream1;

/// A proc macro to decorate tests, which removes boilerplate code required for testing pyO3-wrapped
/// functions within rust.
///
///   1. takes a function (the "testcase") designed to test either a `#[pyo3module]`
///      or a `#[pyo3function]`,
///   2. imports the `pyo3module` and `pyo3function` so they are accessible to a python interpreter embedded in rust,
///   3. creates a macro_rules! to easily call the `pyo3function`,
///   4. executes the body of the testcase using an embedded python interpreter.
///
///
/// ## Specifying the function or module to test with `#[pyo3import(...)]`
///
/// Add the attribute `#[pyo3import(...)]` between `#[pyo3test]` and the testcase using the
/// following format:
///
///   - `#[pyo3import(module_rustfn: from python_module import python_function)]` OR
///   - `#[pyo3import(module_rustfn: import python_module)]`
///
/// where:
///   - `module_rustfn` is the rust function identifier of the `#[pymodule]`
///   - `python_module` is the module name exposed to python
///   - `python_function` is the function name exposed to python
///
/// You can then directly call `python_function!(...)` or use `python_module` and `python_function`
/// within the testcase as described in [pyo3: Calling Python functions][1]
///
/// [1]: https://pyo3.rs/latest/python-from-rust/function-calls.html#calling-python-functions
///
/// ## Note:
///
/// 1. Multiple imports are possible
/// 2. The macro_rules! will accept positional arguments as in the example below OR a tuple
/// in the form of `python_function!(*args)` - the `*` is important, just as in python.
/// 3. The macro_rules! cannot currently cope with keyword arguments or a few positional arguments
/// followed by *args.
///
/// ## Example usage:
///
/// ```ignore # expands to include #[test] so gets ignored anyway
/// use pyo3::prelude::*;
/// use pyo3_testing::pyo3test;
/// #[pyfunction]
/// #[pyo3(name = "addone")]
/// fn py_addone(num: isize) -> isize {
///     num + 1
/// }
///
/// #[pymodule]
/// #[pyo3(name = "adders")]
/// fn py_adders(module: &Bound<'_, PyModule>) -> PyResult<()> {
///     module.add_function(wrap_pyfunction!(py_addone, module)?)?;
///     Ok(())
/// }
///
/// #[pyo3test]
/// #[pyo3import(py_adders: from adders import addone)]
/// fn test_pyo3test_simple_case() {
///     let result = addone!(1_isize);
///     assert_eq!(result, expected_result);
/// }
///
/// #[pyo3test]
/// #[pyo3import(py_adders: import adders)]
/// fn test_pyo3test_import_module_only() {
///     let result: isize = adders
///         .getattr("addone")
///         .unwrap()
///         .call1((1_isize,))
///         .unwrap()
///         .extract()
///         .unwrap();
///     let expected_result = 2_isize;
///     assert_eq!(result, expected_result);
/// }
/// ```
#[proc_macro_attribute]
pub fn pyo3test(attr: TokenStream1, input: TokenStream1) -> TokenStream1 {
    impl_pyo3test(attr.into(), input.into()).into()
}

/// A proc macro to implement the equivalent of pytests `with raises`[1] context manager.
/// 
/// [1]: https://docs.pytest.org/en/latest/getting-started.html#assert-that-a-certain-exception-is-raised
///
/// ## Note:
/// 
/// The code inside the block must be valid rust which returns a `PyResult<T>`.
///
/// ## Example usage:
///
/// ```ignore # expands to include #[test] so gets ignored anyway
/// #[pyo3test]
/// #[allow(unused_macros)]
/// #[pyo3import(py_adders: from adders import addone)]
/// fn test_raises() {
///     with_py_raises!(PyTypeError, { addone.call1(("4",)) }); //can't use `let result =` here
/// }
/// ```

#[proc_macro]
pub fn with_py_raises(input: TokenStream1) -> TokenStream1 {
    impl_with_py_raises(input.into()).into()
}