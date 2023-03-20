use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{token::RawToken::*, Item};

impl<'c> Parser<'c> {
    pub(crate) fn parse_impl(&mut self) -> ParserResult<Item> {
        let mut public = None;

        if self.current.value.is(Pub) {
            public = Some(self.current.span);
            self.advance(false)?; // `pub`
        }

        self.advance(false)?; // `impl`

        let generic_annotations = self.parse_generic_annotations()?;

        let mut r#type = self.parse_type()?;
        let mut r#trait = None;

        if self.current.value.is(For) {
            self.advance(false)?; // `for`

            r#trait = Some(r#type);
            r#type = self.parse_type()?;
        }

        check_token!(self, OpenBrace, "type implementation")?;

        self.advance(false)?; // '{'

        let methods = self.parse_trait_methods()?;

        check_token!(self, CloseBrace, "type implementation")?;

        self.advance(true)?; // '}'

        Ok(Item::Impl(ry_ast::Impl {
            public,
            global_generic_annotations: generic_annotations,
            r#type,
            r#trait,
            methods,
        }))
    }
}

#[cfg(test)]
mod impl_tests {
    use crate::{macros::parser_test, Parser};
    use string_interner::StringInterner;

    parser_test!(impl1, "impl[T] NotOption for T {}");
    parser_test!(impl2, "impl[T] !NotOption for T? {}");
    parser_test!(impl3, "impl[T, M Into[T]] Into[M?] for Tuple[T, M] {}");
}
