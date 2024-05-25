use proc_macro2::TokenStream as TokenStream2;
use pyo3::{PyAny, PyErr};
use syn::{parse_quote, Block, Expr, Ident, ItemStruct, Stmt};
use quote::ToTokens;

struct Pyo3Raises {
    err: Ident,
    block: Block,
}

fn expand(invocation: Pyo3Raises) -> TokenStream2 {
    let err = invocation.err;
    let block = invocation.block;
    let with_block: Stmt = parse_quote! {
        match #block {
            Ok(_) => panic!("No Error"),
            Err(error) if error.is_instance_of::<#err>(py) => return (),
            Err(_) => panic!("Wrong Error"),
        }
    };
    with_block.into_token_stream()
}

#[cfg(test)]
mod test {
    use pyo3::exceptions::PyTypeError;
    use quote::quote;
    use super::*;

    #[test]
    fn test_expansion() {
        let codeblock = parse_quote! {
            {addone.call1("4",)}
        };
        let errortype = parse_quote! {
            PyTypeError
        };
        let invocation = Pyo3Raises {
            err: errortype,
            block: codeblock 
        };
        let expected: TokenStream2 = quote! {
            match  {
                addone.call1("4",)
            };  {
                Ok(_) => panic!("No Error"),
                Err(PyTypeError) => return (),
                Err(_) => panic! ("Wrong Error")
            }
        };
        assert_eq!(expand(invocation).to_string(), expected.to_string())
    }
}