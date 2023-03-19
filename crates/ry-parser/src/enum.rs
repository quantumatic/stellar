use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{location::*, token::RawToken::*, *};

impl<'c> Parser<'c> {
    pub(crate) fn parse_enum_declaration(&mut self, public: Option<Span>) -> ParserResult<Item> {
        self.advance(false)?; // `enum`

        check_token0!(
            self,
            "identifier for enum name",
            Identifier(_),
            "enum declaration"
        )?;

        let name = self.current_ident_with_span();

        self.advance(false)?; // ident

        check_token!(self, OpenBrace, "enum declaration")?;

        self.advance(true)?; // `{`

        let variants = parse_list!(
            self,
            "enum variant",
            CloseBrace,
            true, // top level
            || {
                let doc = self.consume_local_docstring()?;

                check_token0!(self, "identifier", Identifier(_), "enum variant")?;

                let variant = self.current_ident_with_span();

                self.advance(false)?; // ident

                Ok((doc, variant))
            }
        );

        Ok(Item::EnumDecl(EnumDecl {
            public,
            name,
            variants,
        }))
    }
}
