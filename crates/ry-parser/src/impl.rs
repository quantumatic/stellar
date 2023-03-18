use crate::{error::ParserError, macros::*, Parser, ParserResult};

use ry_ast::token::RawToken;
use ry_ast::*;

impl<'c> Parser<'c> {
    pub(crate) fn parse_impl(&mut self) -> ParserResult<TopLevelStatement> {
        let mut public = None;

        if self.current.value.is(RawToken::Pub) {
            public = Some(self.current.span);
            self.advance(false)?; // `pub`
        }

        self.advance(false)?; // `impl`

        let generic_annotations = self.parse_generic_annotations()?;

        let mut r#type = self.parse_type()?;
        let mut r#trait = None;

        if self.current.value.is(RawToken::For) {
            self.advance(false)?; // `for`

            r#trait = Some(r#type);
            r#type = self.parse_type()?;
        }

        check_token!(self, RawToken::OpenBrace, "type implementation")?;

        self.advance(false)?; // '{'

        let methods = self.parse_trait_methods()?;

        check_token!(self, RawToken::CloseBrace, "type implementation")?;

        self.advance(true)?; // '}'

        Ok(TopLevelStatement::Impl(Impl {
            public,
            global_generic_annotations: generic_annotations,
            r#type,
            r#trait,
            methods,
        }))
    }
}
