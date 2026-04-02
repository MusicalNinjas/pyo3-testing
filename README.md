# pyo3-testing: A crate to make testing pyo3 wrapped functions easy in rust

Pyo3-testing is designed to save the need to continually build and install your wrapped extension modules in order to run integration tests in python.

It provides a test attribute `#[pyo3test]` which allows you to shorten your tests to:

```rust
#[pyo3test]
#[pyo3import(py_adders: from adders import addone)]
fn test_pyo3test_simple_case() {
    let result: isize = addone!(1);
    assert_eq!(result, 2);
}
```

Without `pyo3-testing` this test can run to over 20 lines of code and randomly fail due to issues with python interpreter pre-initialisation.

It also provides a `with_py_raises!` macro modelled on pytest's `with raises` context manager to test for expected Exceptions:

```rust
# use pyo3_testing::{pyo3test, with_py_raises};
#[pyo3test]
#[allow(unused_macros)]
#[pyo3import(py_adders: from adders import addone)]
fn test_raises() {
    with_py_raises!(PyTypeError, { addone.call1(("4",)) });
}
```

For a walk-through guide to using the crate along with lots of other tips on developing rust extensions for python see: [Combining rust & python - a worked example](https://musicalninjadad.github.io/FizzBuzz)

Technical documentation for the crate is available at [docs.rs](https://docs.rs/pyo3-testing)

## Supported pyo3 version(s)

As of v0.28.0 pyo3_testing version numbers will shadow pyo3's left-most non-zero major/minor version. This is to simplify things so that you and cargo can always find the right match.

pyo3 has recently made a series of breaking changes to the underlying functionality needed for this crate. Cargo _should_ use the correct version of this crate to match the version of pyo3, but in case you run into issues compatibility is as follows:

| pyo3 version    | pyo3_testing version |
|-----------------|----------------------|
| >= 0.28.2       | 0.28                 |
| 0.27.1..0.27.2  | 0.4                  |
| (0.23..0.27.0)* | (0.3.5)*             |
| 0.22            | 0.3.5                |

* technically compatible but refused by [cargo's treatment of 0.x sermver](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#default-requirements)

## Recognition

This crate wouldn't be possible or necessary without the amazing work done by [pyo3](https://www.github.com/pyo3/pyo3)

## Feedback, ideas and contributions

... are very welcome via [MusicalNinjas/pyo3-testing](https://github.com/MusicalNinjas/pyo3-testing)
