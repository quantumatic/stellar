use crate::{error::ParserError, macros::*, Parser, ParserResult};

use ry_ast::*;
use ry_ast::{location::Span, token::RawToken};

impl<'c> Parser<'c> {
    pub(crate) fn parse_enum_declaration(
        &mut self,
        public: Option<Span>,
    ) -> ParserResult<TopLevelStatement> {
        self.advance(false)?; // 'enum'

        check_token0!(
            self,
            "identifier for enum name",
            RawToken::Identifier(_),
            "enum declaration"
        )?;

        let name = self.get_name();

        self.advance(false)?; // 'name'

        check_token!(self, RawToken::OpenBrace, "enum declaration")?;

        self.advance(true)?; // '{'

        let variants = parse_list!(
            self,
            "enum variant",
            RawToken::CloseBrace,
            true, // top level
            || {
                let doc = self.consume_local_docstring()?;

                check_token0!(self, "identifier", RawToken::Identifier(_), "enum variant")?;

                let variant = self.get_name();

                self.advance(false)?; // id

                Ok((doc, variant))
            }
        );

        Ok(TopLevelStatement::EnumDecl(EnumDecl {
            public,
            name,
            variants,
        }))
    }
}
