use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{
    declaration::{
        Item, StructDeclarationItem, StructMemberDeclaration, WithDocstring, WithDocstringable,
    },
    span::{Span, WithSpan},
    token::RawToken::*,
};

impl<'c> Parser<'c> {
    pub(crate) fn parse_struct_declaration(
        &mut self,
        visiblity: Option<Span>,
    ) -> ParserResult<Item> {
        self.advance()?;

        let name = consume_ident!(self, "struct name in struct declaration");

        let generics = self.parse_generics()?;

        let r#where = self.parse_where_clause()?;

        self.advance_with_docstring()?; // `{`

        let members = self.parse_struct_members()?;

        consume!(with_docstring self, CloseBrace, "struct declaration");

        Ok(StructDeclarationItem::new(visiblity, name, generics, r#where, members).into())
    }

    fn parse_struct_member(&mut self) -> ParserResult<StructMemberDeclaration> {
        let mut visibility = None;
        let mut mutability = None;

        if self.next.unwrap().is(Mut) {
            self.advance()?;
            mutability = Some(self.current.span());
        }

        if self.next.unwrap().is(Pub) {
            self.advance()?;
            visibility = Some(self.current.span());
        }

        if self.next.unwrap().is(Mut) {
            self.advance()?;
            mutability = Some(self.current.span());
        }

        let name = consume_ident!(self, "struct member name in struct definition");

        consume!(self, Colon, "struct member definition");

        let r#type = self.parse_type()?;

        consume!(self, Semicolon, "struct member definition");

        Ok(StructMemberDeclaration::new(
            visibility, mutability, name, r#type,
        ))
    }

    fn parse_struct_members(
        &mut self,
    ) -> ParserResult<Vec<WithDocstring<StructMemberDeclaration>>> {
        let mut members = vec![];

        while !self.next.unwrap().is(CloseBrace) {
            let docstring = self.consume_non_module_docstring()?;
            let member = self.parse_struct_member()?;

            members.push(member.with_docstring(docstring));
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
        "struct test[T, M] { pub mut a: i32; mut pub b: T; pub c: T; d: M; }"
    );
}
