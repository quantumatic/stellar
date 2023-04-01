use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{
    declaration::{
        docstring::{WithDocstring, WithDocstringable},
        function::FunctionDeclaration,
        r#impl::ImplItem,
        Item,
    },
    token::RawToken::*,
    Visibility,
};

impl<'c> Parser<'c> {
    pub(crate) fn parse_impl(&mut self, visibility: Visibility) -> ParserResult<Item> {
        self.advance()?;

        let generics = self.parse_generics()?;

        let mut r#type = self.parse_type()?;
        let mut r#trait = None;

        if self.next.unwrap().is(For) {
            self.advance()?; // `for`

            r#trait = Some(r#type);
            r#type = self.parse_type()?;
        }

        let r#where = self.parse_where_clause()?;

        self.advance()?; // '{'

        let implementations = self.parse_associated_functions_implementations()?;

        consume!(with_docstring self, CloseBrace, "type implementation");

        Ok(ImplItem::new(
            visibility,
            generics,
            r#type,
            r#trait,
            r#where,
            implementations,
        )
        .into())
    }

    pub(crate) fn parse_associated_functions_implementations(
        &mut self,
    ) -> ParserResult<Vec<WithDocstring<FunctionDeclaration>>> {
        let mut associated_functions = vec![];

        while !self.next.unwrap().is(CloseBrace) {
            let docstring = self.consume_non_module_docstring()?;

            let mut visibility = None;

            if self.next.unwrap().is(Pub) {
                visibility = Some(self.next.span())
            }

            associated_functions.push(
                self.parse_function_declaration(visibility)?
                    .with_docstring(docstring),
            );
        }

        Ok(associated_functions)
    }
}

#[cfg(test)]
mod impl_tests {
    use crate::{macros::parser_test, Parser};
    use string_interner::StringInterner;

    parser_test!(impl1, "impl[T] NotOption for T {}");
    parser_test!(
        impl2,
        "impl[T] Into[M?] for Tuple[T, M] where M: Into[T] {}"
    );
}
