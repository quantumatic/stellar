use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{
    declaration::{Function, Item, TraitDeclarationItem, WithDocstring, WithDocstringable},
    span::{Span, WithSpan},
    token::RawToken::*,
};

impl<'c> Parser<'c> {
    pub(crate) fn parse_trait_declaration(
        &mut self,
        visibility: Option<Span>,
    ) -> ParserResult<Item> {
        self.advance()?;

        let name = consume_ident!(self, "trait name in trait declaration");

        let generics = self.parse_generics()?;

        let r#where = self.parse_where_clause()?;

        consume!(with_docstring self, OpenBrace, "trait declaration");

        let methods = self.parse_trait_associated_functions()?;

        consume!(self, CloseBrace, "trait declaration");

        Ok(TraitDeclarationItem::new(visibility, name, generics, r#where, methods).into())
    }

    pub(crate) fn parse_trait_associated_functions(
        &mut self,
    ) -> ParserResult<Vec<WithDocstring<Function>>> {
        let mut associated_functions = vec![];

        while !self.next.unwrap().is(CloseBrace) {
            let docstring = self.consume_non_module_docstring()?;

            let mut visibility = None;

            if self.next.unwrap().is(Pub) {
                visibility = Some(self.next.span());
                self.advance()?;
            }

            associated_functions.push(self.parse_function(visibility)?.with_docstring(docstring));
        }

        Ok(associated_functions)
    }
}

#[cfg(test)]
mod trait_tests {
    use crate::{macros::parser_test, Parser};
    use ry_interner::Interner;

    parser_test!(empty_trait, "trait test {}");
    parser_test!(r#trait, "trait test { fun f(); }");
    parser_test!(
        r#trait_with_generics,
        "trait Into[T] { fun into(self: &Self): T; }"
    );
    parser_test!(
        unnecessary_visibility_qualifier,
        "trait Into[T] { pub fun into(self: &Self): T; }"
    );
}
