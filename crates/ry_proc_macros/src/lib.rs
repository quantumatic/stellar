use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Brace,
    Block, Ident, Token, Type,
};

struct VisitFnMacroInput {
    ident: Ident,
    ty: Type,
    walk_fn_content: Option<Block>,
}

impl Parse for VisitFnMacroInput {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let ident = input.parse()?;
        input.parse::<Token![for]>()?;
        let ty = input.parse()?;
        let mut block = None;

        if input.peek(Brace) {
            block = Some(input.parse()?);
        }

        Ok(VisitFnMacroInput {
            ident,
            ty,
            walk_fn_content: block,
        })
    }
}

#[proc_macro]
pub fn visit_fn(input: TokenStream) -> TokenStream {
    let VisitFnMacroInput {
        ident,
        ty,
        walk_fn_content,
    } = parse_macro_input!(input as VisitFnMacroInput);

    let node_name = ident.to_string();

    let node_name = if let Some(s) = node_name.strip_prefix("r#") {
        s
    } else {
        &node_name
    };

    let visit_fn_name = Ident::new(&format!("visit_{}", node_name), ident.span());
    let walk_fn_name = Ident::new(&format!("walk_{}", node_name), ident.span());

    let walk_fn_body = match walk_fn_content {
        Some(b) => quote! { #b },
        None => quote! { {} },
    };

    quote! {
        fn #visit_fn_name(&mut self, node: #ty) {
            self.#walk_fn_name(node);
        }

        fn #walk_fn_name(&mut self, node: #ty)  #walk_fn_body
    }
    .into()
}
