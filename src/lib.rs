//! Simplifies testing of `#[pyo3function]`s by enabling tests to be condensed to:
//!
//! ```no_run # expands to include #[test] so gets ignored anyway
//! # use pyo3_testing::pyo3test;
//! #[pyo3test]
//! #[pyo3import(py_adders: from adders import addone)]
//! fn test_pyo3test_simple_case() {
//!     let result: isize = addone!(1);
//!     assert_eq!(result, 2);
//! }
//! ```
//!
//! and for checking that the correct type of python Exception is raised:
//!
//! ```no_run # expands to include #[test] so gets ignored anyway
//! # use pyo3_testing::{pyo3test, with_py_raises};
//! #[pyo3test]
//! #[allow(unused_macros)]
//! #[pyo3import(py_adders: from adders import addone)]
//! fn test_raises() {
//!     with_py_raises!(PyTypeError, { addone.call1(("4",)) });
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
///   3. creates a `"call macro"` for each `pyo3function` so you can easily call it,
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
/// ### Note:
///
/// 1. Multiple imports are possible
///
/// ## "Call macros"
///
/// `#[pyo3test]` will automatically generate a macro for each of the `python_function`s imported.
/// The macro will have the same name as the function name exposed to python and can be called
/// using `python_function!()`. This avoids the need to use the correct `.call()`, `.call1()` or
/// `.call2()` method and then `.unwrap().extract().unwrap()` the result.
///
/// ### Note:
/// 1. The `"call macros"` will accept positional arguments as in the example below OR a tuple
/// in the form of `python_function!(*args)` - the `*` is important, just as in python
/// 2. The "Call macros" cannot currently cope with keyword arguments or a mixture of some
///  positional arguments followed by *args
/// 3. The macros will `panic!` if an error occurs due to incorrect argument types, missing arguments
/// etc. - this is designed for use in tests, where panicing is the acceptable and required behaviour
///
/// ## Example usage:
///
/// ```no_run # expands to include #[test] so gets ignored anyway
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
///     assert_eq!(result, 2);
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
///     assert_eq!(result, 2);
/// }
/// ```
#[proc_macro_attribute]
pub fn pyo3test(attr: TokenStream1, input: TokenStream1) -> TokenStream1 {
    impl_pyo3test(attr.into(), input.into()).into()
}

/// A proc macro to implement the equivalent of [pytest's `with raises`][1] context manager.
///
/// Use like this: `with_py_raises(ExpectedErrType, {code block which should raise error })`
///
/// [1]: https://docs.pytest.org/en/latest/getting-started.html#assert-that-a-certain-exception-is-raised
///
/// ## Note:
///
/// 1. The `ExpectedErrType` must be _in scope_ when calling the macro and must implement
/// `std::from::From<E> for PyErr`
/// 1. The code inside the block must be valid rust which returns a `PyResult<T>`. Currently it is not
/// possible to use the autogenerated call macros provided by `#[pyo3test]`[macro@pyo3test].
/// If you would like to see that feature, please let me know via [github][2]
/// 1. Add `#[allow(unused_macros)]` to disable the warning that you have imported a python function
/// but not called the associated macro.
/// 1. The code will `panic!` if the incorrect error, or no error, is returned - this is designed for
/// use in tests, where panicing is the acceptable and required behaviour
///
/// [2]: https://github.com/MusicalNinjas/pyo3-testing/issues/3
///
/// ## Example usage:
///
/// ```no_run # expands to include #[test] so gets ignored anyway
/// use pyo3::exceptions::PyTypeError;
/// use pyo3_testing::{pyo3test, with_py_raises};
/// #[pyo3test]
/// #[allow(unused_macros)]
/// #[pyo3import(py_adders: from adders import addone)]
/// fn test_raises() {
///     //can't use `let result =` or `addone!()` here as they don't return a `Result`
///     with_py_raises!(PyTypeError, { addone.call1(("4",)) });
/// }
/// ```

#[proc_macro]
pub fn with_py_raises(input: TokenStream1) -> TokenStream1 {
    impl_with_py_raises(input.into()).into()
}
