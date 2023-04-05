use crate::{error::*, Parser};
use ry_ast::{
    declaration::{Documented, FunctionDeclaration, ImplItem, Item, WithDocstring},
    token::{Keyword::*, Punctuator::*, RawToken::*},
    Visibility,
};

impl Parser<'_> {
    pub(crate) fn parse_impl(&mut self, visibility: Visibility) -> ParseResult<Item> {
        self.advance();

        let generics = self.optionally_parse_generics()?;

        let mut r#type = self.parse_type()?;
        let mut r#trait = None;

        if let Keyword(For) = self.next.inner {
            self.advance();

            r#trait = Some(r#type);
            r#type = self.parse_type()?;
        }

        let r#where = self.optionally_parse_where_clause()?;

        self.advance();

        let implementations = self.parse_associated_functions_implementations()?;

        self.consume_with_docstring(Punctuator(CloseBrace), "type implementation")?;

        Ok(ImplItem {
            visibility,
            generics,
            r#type,
            r#trait,
            r#where,
            implementations,
        }
        .into())
    }

    pub(crate) fn parse_associated_functions_implementations(
        &mut self,
    ) -> ParseResult<Vec<Documented<FunctionDeclaration>>> {
        let mut associated_functions = vec![];

        loop {
            if self.next.inner == Punctuator(CloseBrace) {
                break;
            }

            let docstring = self.consume_non_module_docstring()?;

            let mut visibility = Visibility::private();

            if let Keyword(Pub) = self.next.inner {
                visibility = Visibility::public(self.next.span)
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
