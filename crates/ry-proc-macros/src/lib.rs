extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Error, Ident, Token, Type,
};

struct VisitFnMacroInput {
    ident: Ident,
    ty: Type,
}

impl Parse for VisitFnMacroInput {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty = input.parse()?;
        Ok(VisitFnMacroInput { ident, ty })
    }
}

#[proc_macro]
pub fn visit_fn(input: TokenStream) -> TokenStream {
    let VisitFnMacroInput { ident, ty } = parse_macro_input!(input as VisitFnMacroInput);

    let name = ident.to_string();

    let name = if let Some(s) = name.strip_prefix("r#") {
        s
    } else {
        &name
    };

    let fn_ident = Ident::new(&format!("visit_{}", name), ident.span());
    let call_ident = Ident::new(&format!("walk_{}", name), ident.span());

    let output = quote! {
        fn #fn_ident(&mut self, node: #ty) {
            #call_ident(self, node);
        }
    };

    output.into()
}
