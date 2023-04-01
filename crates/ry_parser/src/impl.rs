use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{span::Span, token::RawToken::*, declaration::Item};

impl<'c> Parser<'c> {
    pub(crate) fn parse_impl(&mut self, public: Option<Span>) -> ParserResult<Item> {
        self.advance()?;

        let generic_annotations = self.parse_generic_annotations()?;

        let mut r#type = self.parse_type()?;
        let mut r#trait = None;

        if self.next.unwrap().is(For) {
            self.advance()?; // `for`

            r#trait = Some(r#type);
            r#type = self.parse_type()?;
        }

        let r#where = self.parse_where_clause()?;

        self.advance()?; // '{'

        let methods = self.parse_trait_methods()?;

        consume!(with_docstring self, CloseBrace, "type implementation");

        Ok(Item::Impl(ry_ast::Impl {
            public,
            global_generic_annotations: generic_annotations,
            r#type,
            r#trait,
            methods,
            r#where,
        }))
    }
}

#[cfg(test)]
mod impl_tests {
    use crate::{macros::parser_test, Parser};
    use string_interner::StringInterner;

    parser_test!(impl1, "impl[T] NotOption for T {}");
    parser_test!(impl2, "impl[T] !NotOption for T? {}");
    parser_test!(
        impl3,
        "impl[T] Into[M?] for Tuple[T, M] where M of Into[T] {}"
    );
}
