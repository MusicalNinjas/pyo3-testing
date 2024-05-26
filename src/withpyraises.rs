/// The implementation of `with_py_raises`[1], all logic is here using `TokenStream2` to allow
/// for unit testing and easier refactoring.
///
/// [1]: https://docs.pytest.org/en/latest/getting-started.html#assert-that-a-certain-exception-is-raised
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse2, parse_quote,
    token::Comma,
    Block, Ident, Stmt,
};

/// Parses the macro invocation contents as a with raises statement and then returns the
/// required code segment to check that the expected error is raised.
pub fn impl_with_py_raises(input: TokenStream2) -> TokenStream2 {
    let withraisesstmt: WithRaisesStmt = match parse2(input) {
        Ok(withraisesstmt) => withraisesstmt,
        Err(e) => return e.into_compile_error(),
    };
    expand(withraisesstmt)
}

/// Represents a well-formed `with pytest.raises`-like statement.
///
/// In order to be correctly parsed this should be in the form of
/// `Error Type` `Comma: [,]` `{block in braces}`  
#[derive(Debug, PartialEq)]
struct WithRaisesStmt {
    /// The error type, this must be the ident of a valid rust error type which is already in scope.
    /// The error type must implement `std::from::From<E> for PyErr`, which all pyo3 errors do.
    /// See [pyo3 - Error handling][1] for details on implementing this for custom error types.
    ///
    /// [1]: https://pyo3.rs/v0.21.2/function/error-handling#custom-rust-error-types
    err: Ident,
    /// A valid rust code block which returns a `PyResult` and is expected to result in the
    /// specified `err`. The same guidance applies here as it does to with `pytest.raises` blocks
    /// in python: keep this as short as possible to be sure you are checking the right thing.
    block: Block,
}

/// See Doc Comment above for correct format...
impl Parse for WithRaisesStmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let error_example =
        "\nCorrect format for with_py_raises is: `Error Type` `Comma: [,]` `{block in braces}`\n\
        E.g.: `with_py_raises!(PyTypeError, { addone.call1((\"4\",)) })`";
        let err: Ident = input.parse()?;
        let _comma: Comma = match input.parse() {
            Ok(comma) => comma,
            Err(_) => {
                return Err(syn::Error::new(
                    err.span(),
                    "Expected a comma (`,`) after this:".to_string() + error_example,
                ))
            }
        };
        let block: Block = match input.parse() {
            Ok(block) => block,
            Err(error) => {
                return Err(syn::Error::new(
                    error.span(),
                    "Expected a code block with braces (`{ ... }`) here:".to_string()
                        + error_example,
                ))
            }
        };
        Ok(WithRaisesStmt { err, block })
    }
}

/// Take a WithRaisesStmt and return a TokenStream2 which panics if the expected error is raised
/// and returns () otherwise.
fn expand(withraisesstmt: WithRaisesStmt) -> TokenStream2 {
    let err = withraisesstmt.err;
    let block = withraisesstmt.block;
    let expanded: Stmt = parse_quote! {
        match #block {
            Ok(_) => panic!("No Error"),
            Err(error) if error.is_instance_of::<#err>(py) => return (),
            Err(_) => panic!("Wrong Error"),
        }
    };
    expanded.into_token_stream()
}

#[cfg(test)]
mod test {
    use super::*;
    use quote::quote;

    #[test]
    fn test_expansion() {
        let codeblock = parse_quote! {
            {addone.call1("4",)}
        };
        let errortype = parse_quote! {
            PyTypeError
        };
        let invocation = WithRaisesStmt {
            err: errortype,
            block: codeblock,
        };
        let expected: TokenStream2 = quote! {
            match  {
                addone.call1("4",)
            }  {
                Ok(_) => panic!("No Error"),
                Err(error) if error.is_instance_of::<PyTypeError>(py) => return (),
                Err(_) => panic!("Wrong Error"),
            }
        };
        assert_eq!(expand(invocation).to_string(), expected.to_string())
    }

    #[test]
    fn test_parse_input() {
        let input: WithRaisesStmt = parse_quote! {
            PyTypeError, {
                addone.call1("4",)
            }
        };
        let codeblock = parse_quote! {
            {addone.call1("4",)}
        };
        let errortype = parse_quote! {
            PyTypeError
        };
        let expected = WithRaisesStmt {
            err: errortype,
            block: codeblock,
        };
        assert_eq!(input, expected);
    }
}
