use crate::{error::ParseError, macros::*, ParseResult, Parser};
use ry_ast::{
    declaration::{
        Item, StructDeclarationItem, StructMemberDeclaration, WithDocstring, WithDocstringable,
    },
    span::WithSpan,
    token::{Keyword::*, Punctuator::*, RawToken::*},
    Mutability, Visibility,
};

impl Parser<'_> {
    pub(crate) fn parse_struct_declaration(&mut self, visiblity: Visibility) -> ParseResult<Item> {
        self.advance()?;

        let name = consume_ident!(self, "struct name in struct declaration");

        let generics = self.optionally_parse_generics()?;

        let r#where = self.optionally_parse_where_clause()?;

        self.advance_with_docstring()?; // `{`

        let members = self.parse_struct_members()?;

        consume!(with_docstring self, Punctuator(CloseBrace), "struct declaration");

        Ok(StructDeclarationItem::new(visiblity, name, generics, r#where, members).into())
    }

    fn parse_struct_member(&mut self) -> ParseResult<StructMemberDeclaration> {
        let mut visibility = Visibility::private();
        let mut mutability = Mutability::immutable();

        if let Keyword(Mut) = self.next.unwrap() {
            self.advance()?;
            mutability = Mutability::mutable(self.current.span());
        }

        if let Keyword(Pub) = self.next.unwrap() {
            self.advance()?;
            visibility = Visibility::public(self.current.span());
        }

        if let Keyword(Mut) = self.next.unwrap() {
            self.advance()?;
            mutability = Mutability::mutable(self.current.span());
        }

        let name = consume_ident!(self, "struct member name in struct definition");

        consume!(self, Punctuator(Colon), "struct member definition");

        let r#type = self.parse_type()?;

        consume!(self, Punctuator(Semicolon), "struct member definition");

        Ok(StructMemberDeclaration::new(
            visibility, mutability, name, r#type,
        ))
    }

    fn parse_struct_members(&mut self) -> ParseResult<Vec<WithDocstring<StructMemberDeclaration>>> {
        let mut members = vec![];

        loop {
            if let Punctuator(CloseBrace) = self.next.unwrap() {
                break;
            }

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
    use ry_interner::Interner;

    parser_test!(empty_struct, "struct test {}");
    parser_test!(
        r#struct,
        "struct test[T, M] { pub mut a: i32; mut pub b: T; pub c: T; d: M; }"
    );
}
