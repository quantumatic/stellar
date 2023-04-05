use crate::{error::ParseError, macros::*, ParseResult, Parser};
use ry_ast::{
    declaration::{Documented, Function, Item, TraitDeclarationItem, WithDocstring},
    span::At,
    token::{Keyword::*, Punctuator::*, RawToken::*},
    Visibility,
};

impl Parser<'_> {
    pub(crate) fn parse_trait_declaration(&mut self, visibility: Visibility) -> ParseResult<Item> {
        self.advance()?;

        let name = consume_ident!(self, "trait name in trait declaration");

        let generics = self.optionally_parse_generics()?;

        let r#where = self.optionally_parse_where_clause()?;

        consume!(with_docstring self, Punctuator(OpenBrace), "trait declaration");

        let methods = self.parse_trait_associated_functions()?;

        consume!(self, Punctuator(CloseBrace), "trait declaration");

        Ok(TraitDeclarationItem {
            visibility,
            name,
            generics,
            r#where,
            methods,
        }
        .into())
    }

    pub(crate) fn parse_trait_associated_functions(
        &mut self,
    ) -> ParseResult<Vec<Documented<Function>>> {
        let mut associated_functions = vec![];

        loop {
            if let Punctuator(CloseBrace) = self.next.unwrap() {
                break;
            }

            let docstring = self.consume_non_module_docstring()?;

            let mut visibility = Visibility::private();

            if let Keyword(Pub) = self.next.unwrap() {
                visibility = Visibility::public(self.next.span());
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
