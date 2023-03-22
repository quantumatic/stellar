use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{location::Span, token::RawToken::*, *};

impl<'c> Parser<'c> {
    pub(crate) fn parse_struct_declaration(&mut self, public: Option<Span>) -> ParserResult<Item> {
        self.advance()?;

        let name = consume_ident!(self, "struct name in struct declaration");

        let generic_annotations = self.parse_generic_annotations()?;

        let r#where = self.parse_where_clause()?;

        self.advance_with_comments()?; // `{`

        let members = self.parse_struct_members()?;

        consume!(self, CloseBrace, "struct declaration");

        Ok(Item::StructDecl(StructDecl {
            generic_annotations,
            public,
            name,
            members,
            r#where,
        }))
    }

    fn parse_struct_member(&mut self) -> ParserResult<StructMemberDef> {
        let mut public = None;
        let mut r#mut = None;

        if self.next.value.is(Mut) {
            self.advance()?;
            r#mut = Some(self.current.span);
        }

        if self.next.value.is(Pub) {
            self.advance()?;
            public = Some(self.current.span);
        }

        if self.next.value.is(Mut) {
            self.advance()?;
            r#mut = Some(self.current.span);
        }

        let name = consume_ident!(self, "struct member name in struct definition");

        let r#type = self.parse_type()?;

        consume!(self, Semicolon, "struct member definition");

        Ok(StructMemberDef {
            public,
            r#mut,
            name,
            r#type,
        })
    }

    fn parse_struct_members(&mut self) -> ParserResult<Vec<(Docstring, StructMemberDef)>> {
        let mut members = vec![];

        while !self.next.value.is(CloseBrace) {
            members.push((self.consume_local_docstring()?, self.parse_struct_member()?));
        }

        Ok(members)
    }
}

#[cfg(test)]
mod struct_tests {
    use crate::{macros::parser_test, Parser};
    use string_interner::StringInterner;

    parser_test!(empty_struct, "struct test {}");
    parser_test!(
        r#struct,
        "struct test[T, M] { pub mut a i32; mut pub b T; pub c T; d M; }"
    );
}
