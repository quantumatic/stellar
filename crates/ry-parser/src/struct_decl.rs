use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{location::Span, token::RawToken::*, *};
use std::string::String;

impl<'c> Parser<'c> {
    pub(crate) fn parse_struct_declaration(&mut self, public: Option<Span>) -> ParserResult<Item> {
        self.advance(false)?; // `struct`

        check_token0!(
            self,
            "identifier for struct name",
            Identifier(_),
            "struct declaration"
        )?;

        let name = self.current_ident_with_span();

        self.advance(false)?; // name

        let generic_annotations = self.parse_generic_annotations()?;

        check_token!(self, OpenBrace, "struct declaration")?;

        self.advance(true)?; // `{`

        let members = self.parse_struct_members()?;

        check_token!(self, CloseBrace, "struct declaration")?;

        self.advance(true)?; // `}`

        Ok(Item::StructDecl(StructDecl {
            generic_annotations,
            public,
            name,
            members,
        }))
    }

    fn parse_struct_member(&mut self) -> ParserResult<StructMemberDef> {
        let mut public = None;

        if self.current.value.is(Pub) {
            public = Some(self.current.span);
            self.advance(false)?;
        }

        check_token0!(
            self,
            "identifier for struct member name or '}'",
            Identifier(_),
            "struct definition"
        )?;

        let name = self.current_ident_with_span();

        self.advance(false)?;

        let r#type = self.parse_type()?;

        check_token!(self, Semicolon, "struct member definition")?;

        self.advance(true)?; // `;`

        Ok(StructMemberDef {
            public,
            name,
            r#type,
        })
    }

    fn parse_struct_members(&mut self) -> ParserResult<Vec<(String, StructMemberDef)>> {
        let mut members = vec![];

        while !self.current.value.is(CloseBrace) {
            members.push((self.consume_local_docstring()?, self.parse_struct_member()?));
        }

        Ok(members)
    }
}
