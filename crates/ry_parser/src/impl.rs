use crate::{error::*, macros::*, Parser};
use ry_ast::{
    declaration::{FunctionDeclaration, ImplItem, Item, WithDocstring, WithDocstringable},
    token::{Keyword::*, Punctuator::*, RawToken::*},
    Visibility,
};

impl Parser<'_> {
    pub(crate) fn parse_impl(&mut self, visibility: Visibility) -> ParseResult<Item> {
        self.advance()?;

        let generics = self.optionally_parse_generics()?;

        let mut r#type = self.parse_type()?;
        let mut r#trait = None;

        if let Keyword(For) = self.next.unwrap() {
            self.advance()?; // `for`

            r#trait = Some(r#type);
            r#type = self.parse_type()?;
        }

        let r#where = self.optionally_parse_where_clause()?;

        self.advance()?; // '{'

        let implementations = self.parse_associated_functions_implementations()?;

        consume!(with_docstring self, Punctuator(CloseBrace), "type implementation");

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
    ) -> ParseResult<Vec<WithDocstring<FunctionDeclaration>>> {
        let mut associated_functions = vec![];

        loop {
            if let Punctuator(CloseBrace) = self.next.unwrap() {
                break;
            }

            let docstring = self.consume_non_module_docstring()?;

            let mut visibility = Visibility::private();

            if let Keyword(Pub) = self.next.unwrap() {
                visibility = Visibility::public(self.next.span())
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
    use ry_interner::Interner;

    parser_test!(impl1, "impl[T] NotOption for T {}");
    parser_test!(
        impl2,
        "impl[T] Into[Option[M]] for Tuple[T, M] where M: Into[T] {}"
    );
}
