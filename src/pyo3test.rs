//! All the implementation logic for `#[pyo3test]`.
//!
//! Separated out into this module, using TokenStream2 to allow for unit testing and easier
//! refactoring.

use std::fmt::Debug;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse2, parse_quote,
    token::Colon,
    Attribute, Ident, ItemFn, Signature, Stmt, Token,
};

/// The function which is called by the proc macro `pyo3test`.
/// Takes a TokenStream2 input, parses it as a Pyo3TestCase and returns a wrapped
/// function with the requested imports, run in Python::with_gil.
///
/// The parsing is fallible as the testcase or attributes may be incorrectly constructed. In case of
/// a parsing error this will be converted to a compile error and returned.
pub fn impl_pyo3test(_attr: TokenStream2, input: TokenStream2) -> TokenStream2 {
    let testcase: Pyo3TestCase = match parse2::<ItemFn>(input).and_then(|itemfn| itemfn.try_into())
    {
        Ok(testcase) => testcase,
        Err(e) => return e.into_compile_error(),
    };
    wrap_testcase(testcase)
}

/// A pyo3 test case consisting of zero or more imports and an ItemFn which should be wrapped to
/// execute in Python::with_gil. Don't construct this directly but use .try_into() on a suitable ItemFn

// #[derive(Debug, PartialEq)] - Signature, Stmt, Attribute don't allow either Debug or PartialEq currently.
struct Pyo3TestCase {
    pyo3imports: Vec<Pyo3Import>,
    signature: Signature,
    statements: Vec<Stmt>,
    otherattributes: Vec<Attribute>,
}

/// Attempt to convert an ItemFn into a Pyo3TestCase. This is a fallible conversion as the arguments
/// provided to a Pyo3Import Attribute may be empty.
impl TryFrom<ItemFn> for Pyo3TestCase {
    type Error = syn::Error;

    fn try_from(testcase: ItemFn) -> syn::Result<Pyo3TestCase> {
        let mut pyo3imports = Vec::<Pyo3Import>::new();
        let mut otherattributes = Vec::<Attribute>::new();
        for attr in testcase.attrs {
            if attr.path().is_ident("pyo3import") {
                pyo3imports.push(attr.parse_args()?);
            } else {
                otherattributes.push(attr);
            };
        }

        Ok(Pyo3TestCase {
            pyo3imports,
            signature: testcase.sig,
            statements: testcase.block.stmts,
            otherattributes,
        })
    }
}

/// A python `import` statement for a pyo3-wrapped function.
#[derive(Debug, PartialEq)]
struct Pyo3Import {
    /// The *rust* `ident` of the wrapped module
    o3_moduleident: Ident,
    /// The *python* module name
    py_modulename: String,
    /// The *python* function name
    py_functionname: Option<String>,
}

impl Parse for Pyo3Import {
    /// Attributes parsing to Pyo3Imports should have the format:
    /// `moduleidentifier: from modulename import functionname`
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        // Written by a rust newbie, if there is a better option than all these assignments; please
        // feel free to change this code...
        let o3_moduleident;
        if input.peek2(Token![:]) {
            o3_moduleident = input.parse()?;
            let _: Colon = input.parse()?;
        } else {
            return Err(input.error("invalid import statement: expected a colon (':') after this"));
        }
        let firstkeyword: PythonImportKeyword = input.parse()?;
        let py_modulename = input.parse::<Ident>()?.to_string();
        let py_functionname = match firstkeyword {
            PythonImportKeyword::from => {
                let _import: PythonImportKeyword = input.parse()?;
                Some(input.parse::<Ident>()?.to_string())
            }
            PythonImportKeyword::import => None,
        };

        Ok(Pyo3Import {
            o3_moduleident,
            py_modulename,
            py_functionname,
        })
    }
}

/// Only the keywords `from` and `import` are valid for a python import statement, which has to take
/// the form: `from x import y` or `import x`.
/// Note we do not accept the additional keyword `as` by design: this is a simple testing framework
/// to validate correct binding, type conversion and errorhandling.
#[allow(non_camel_case_types)] // represent actual keywords in python which are lower case
#[derive(Debug, PartialEq)]
enum PythonImportKeyword {
    from,
    import,
}

impl Parse for PythonImportKeyword {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let keyword = input.parse::<Ident>()?;
        match keyword.to_string().as_str() {
            "from" => Ok(PythonImportKeyword::from),
            "import" => Ok(PythonImportKeyword::import),
            _ => Err(syn::Error::new(
                keyword.span(),
                "invalid import statement: expect 'from' or 'import' here",
            )),
        }
    }
}

/// Takes a code block which should be executed using Python::with_gil and adds the required
/// pyo3 equivalent `import` and `with_gil` statements.
///
/// Technically this is the equivalent to the python statements:
/// ```python
/// import module
/// function = module.function
/// ```
/// and not `from module import function`
#[allow(non_snake_case)] // follow python exception naming for error messages
fn wrap_testcase(mut testcase: Pyo3TestCase) -> TokenStream2 {
    //The quote crate cannot interpolate fields within structs so we need to separate out all
    //import statements into Vecs of the individual fields. To make the final `quote` more readable,
    //we also construct the longer strings and the Idents in advance.
    //
    //This is safe as the order of a Vec is guaranteed, so we will not mismatch fields from different
    //imports (but note the two different Vecs `py_moduleidents` and `py_moduleswithfnsidents`).
    let mut o3_moduleidents = Vec::<Ident>::new(); // idents of the initial rust fns representing modules
    let mut o3_pymoduleidents = Vec::<Ident>::new(); // interim idents representing the modules after initial binding to the GIL token
    let mut py_moduleidents = Vec::<Ident>::new(); // final idents representing the imported modules
    let mut py_modulenames = Vec::<String>::new(); // The module names
    let mut py_ModuleNotFoundErrormsgs = Vec::<String>::new(); // The error messages to give if the module is invalid
    let mut py_functionidents = Vec::<Ident>::new(); // idents representing the imported functions
    let mut py_macroidents = Vec::<Ident>::new(); // idents representing the macro_rules! used to call the functions
    let mut py_moduleswithfnsidents = Vec::<Ident>::new(); // final idents representing the imported modules (only those with named function imports)
    let mut py_functionnames = Vec::<String>::new(); // The function names
    let mut py_AttributeErrormsgs = Vec::<String>::new(); // The error messages to give if the function is invalid

    for pyo3import in testcase.pyo3imports {
        // statements ordered to allow multiple borrows of module and functionname before moving to Vec
        let py_modulename = pyo3import.py_modulename;
        if let Some(py_functionname) = pyo3import.py_functionname {
            py_AttributeErrormsgs
                .push("Failed to get ".to_string() + &py_functionname + " function");
            py_functionidents.push(Ident::new(&py_functionname, Span::call_site()));
            py_macroidents.push(Ident::new(&py_functionname, Span::call_site()));
            py_moduleswithfnsidents.push(Ident::new(&py_modulename, Span::call_site()));
            py_functionnames.push(py_functionname);
        };
        py_ModuleNotFoundErrormsgs.push("Failed to import ".to_string() + &py_modulename);
        py_moduleidents.push(Ident::new(&py_modulename, Span::call_site()));
        py_modulenames.push(py_modulename);
        o3_pymoduleidents.push(format_ident!("{}_pymodule", pyo3import.o3_moduleident));
        o3_moduleidents.push(pyo3import.o3_moduleident);
    }

    let testfn_signature = testcase.signature;
    let testfn_statements = testcase.statements;

    let mut testfn: ItemFn = parse_quote!(
        #[test]
        #testfn_signature {
            pyo3::prepare_freethreaded_python();
            Python::with_gil(|py| {

                // from sys import modules as sys_modules
                let sys = PyModule::import_bound(py, "sys").unwrap();
                let sys_modules: Bound<'_, PyDict> =
                    sys.getattr("modules").unwrap().downcast_into().unwrap();

                #( // for each module to import

                    // create the PyModule and bind it to our GIL token `py`
                    let #o3_pymoduleidents = unsafe { Bound::from_owned_ptr(py, #o3_moduleidents::__pyo3_init()) };

                    // insert module into sys_modules
                    sys_modules
                        .set_item(#py_modulenames, #o3_pymoduleidents)
                        .expect(#py_ModuleNotFoundErrormsgs);

                    // and get it back - cannot fail as we just put it there
                    let #py_moduleidents = sys_modules.get_item(#py_modulenames).unwrap().unwrap();
                )*

                #( // for each function to import

                    // assign each wrapped function to a rust Ident of the same name
                    let #py_functionidents = #py_moduleswithfnsidents
                        .getattr(#py_functionnames)
                        .expect(#py_AttributeErrormsgs);

                    // create call macros last, so they have access to the py_functionidents we create
                    macro_rules! #py_macroidents {
                        ($($arg:tt),+) => {
                            #py_functionidents
                            .call1(($($arg,)+))
                            .unwrap()
                            .extract()
                            .unwrap()
                        };
                        (*$args:ident) => {
                            #py_functionidents
                            .call1($args)
                            .unwrap()
                            .extract()
                            .unwrap()
                        };
                        () => {
                            #py_functionidents
                            .call0()
                            .unwrap()
                            .extract()
                            .unwrap()
                        };
                    };
                )*

                #(#testfn_statements)*
            });
        }
    );

    testfn.attrs.append(&mut testcase.otherattributes);

    testfn.into_token_stream()
}

#[allow(clippy::non_minimal_cfg)]
// need to regularly disable this test by ading an additional cfg item.
// It is highly coupled to the exact expansion, but I can't see a better way to test this right now.
#[cfg(all(test))]
mod tests {
    use quote::quote;

    use super::*;

    #[test]
    fn test_other_attribute() {
        let testcase: TokenStream2 = quote! {
            #[pyo3import(py_fizzbuzzo3: from fizzbuzzo3 import fizzbuzz)]
            #[anotherattribute]
            #[pyo3import(foo_o3: from pyfoo import pybar)]
            fn test_fizzbuzz() {
                assert!(true)
            }
        };

        let expected: TokenStream2 = quote! {
            #[test]
            #[anotherattribute]
            fn test_fizzbuzz() {
                pyo3::prepare_freethreaded_python();
                Python::with_gil(|py| {
                    let sys = PyModule::import_bound(py, "sys").unwrap();
                    let sys_modules: Bound<'_, PyDict> =
                        sys.getattr("modules").unwrap().downcast_into().unwrap();
                    let py_fizzbuzzo3_pymodule = unsafe { Bound::from_owned_ptr(py, py_fizzbuzzo3::__pyo3_init()) };
                    sys_modules
                        .set_item("fizzbuzzo3", py_fizzbuzzo3_pymodule)
                        .expect("Failed to import fizzbuzzo3");
                    let fizzbuzzo3 = sys_modules.get_item("fizzbuzzo3").unwrap().unwrap();
                    let foo_o3_pymodule = unsafe { Bound::from_owned_ptr(py, foo_o3::__pyo3_init()) };
                    sys_modules
                        .set_item("pyfoo", foo_o3_pymodule)
                        .expect("Failed to import pyfoo");
                    let pyfoo = sys_modules.get_item("pyfoo").unwrap().unwrap();
                    let fizzbuzz = fizzbuzzo3
                        .getattr("fizzbuzz")
                        .expect("Failed to get fizzbuzz function");
                    macro_rules! fizzbuzz {
                        ($($arg:tt),+) => {
                            fizzbuzz
                            .call1(($($arg,)+))
                            .unwrap()
                            .extract()
                            .unwrap()
                        };
                        (*$args:ident) => {
                            fizzbuzz
                            .call1($args)
                            .unwrap()
                            .extract()
                            .unwrap()
                        };
                        () => {
                            fizzbuzz
                            .call0()
                            .unwrap()
                            .extract()
                            .unwrap()
                        };
                    };
                    let pybar = pyfoo
                        .getattr("pybar")
                        .expect("Failed to get pybar function");
                    macro_rules! pybar {
                        ($($arg:tt),+) => {
                            pybar
                            .call1(($($arg,)+))
                            .unwrap()
                            .extract()
                            .unwrap()
                        };
                        (*$args:ident) => {
                            pybar
                            .call1($args)
                            .unwrap()
                            .extract()
                            .unwrap()
                        };
                        () => {
                            pybar
                            .call0()
                            .unwrap()
                            .extract()
                            .unwrap()
                        };
                    };
                    assert!(true)
                });
            }
        };

        let output: TokenStream2 = impl_pyo3test(quote! {}, testcase);

        assert_eq!(output.to_string(), expected.to_string());
    }
}
