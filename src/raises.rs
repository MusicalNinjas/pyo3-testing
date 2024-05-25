use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse2, parse_quote,
    token::Comma,
    Block, Ident, Stmt,
};

pub fn impl_with_py_raises(input: TokenStream2) -> TokenStream2 {
    let pyraisesblock: PyRaisesBlock = match parse2(input) {
        Ok(pyraisesblock) => pyraisesblock,
        Err(e) => return e.into_compile_error(),
    };
    expand(pyraisesblock)
}

#[derive(Debug, PartialEq)]
struct PyRaisesBlock {
    err: Ident,
    block: Block,
}

impl Parse for PyRaisesBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let err: Ident = input.parse()?;
        let _comma: Comma = input.parse()?;
        let block: Block = input.parse()?;
        Ok(PyRaisesBlock { err, block })
    }
}

fn expand(pyraisesblock: PyRaisesBlock) -> TokenStream2 {
    let err = pyraisesblock.err;
    let block = pyraisesblock.block;
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
        let invocation = PyRaisesBlock {
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
        let input: PyRaisesBlock = parse_quote! {
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
        let expected = PyRaisesBlock {
            err: errortype,
            block: codeblock,
        };
        assert_eq!(input, expected);
    }
}
