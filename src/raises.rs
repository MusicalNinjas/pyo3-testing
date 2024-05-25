use proc_macro2::TokenStream as TokenStream2;
use pyo3::{PyAny, PyErr};
use syn::{parse_quote, Block, Expr, Ident, ItemStruct, Stmt};

struct Pyo3Raises {
    err: Ident,
    block: Block,
}

fn expand(invocation: Pyo3Raises) -> Expr {
    let err = invocation.err;
    let block = invocation.block;
    parse_quote! {
        match #block {
            Ok(_) => panic!("No Error"),
            Err(error) if error.is_instance_of::<#err>(py) => return (),
            Err(_) => panic!("Wrong Error"),
        }
    }
}

#[cfg(test)]
mod test {
    use pyo3::exceptions::PyTypeError;

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
        let expected: Expr = parse_quote! {
            match  {
                addone.call1("4",)
            };  {
                Ok(_) => panic!("No Error"),
                Err(PyTypeError) => return (),
                Err(_) => panic! ("Wrong Error")
            }
        };
        assert_eq!(expand(invocation), expected)
    }
}