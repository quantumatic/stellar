use crate::{
    r#type::{GenericsParser, TypeParser, WhereClauseParser},
    OptionalParser, ParseResult, Parser, ParserState,
};
use ry_ast::{
    declaration::{
        Documented, Item, StructDeclarationItem, StructMemberDeclaration, WithDocComment,
    },
    token::{
        Keyword::Pub,
        Punctuator::{CloseBrace, Colon, Semicolon},
        RawToken::{Keyword, Punctuator},
    },
    Visibility,
};

pub(crate) struct StructMemberParser;

impl Parser for StructMemberParser {
    type Output = StructMemberDeclaration;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut visibility = Visibility::private();

        if state.next.inner == Keyword(Pub) {
            state.next_token();
            visibility = Visibility::public(state.current.span);
        }

        let name = state.consume_identifier("struct member name in struct definition")?;

        state.consume(Punctuator(Colon), "struct member definition")?;

        let r#type = TypeParser.parse_with(state)?;

        state.consume(Punctuator(Semicolon), "struct member definition")?;

        Ok(StructMemberDeclaration {
            visibility,
            name,
            r#type,
        })
    }
}

pub(crate) struct StructMembersParser;

impl Parser for StructMembersParser {
    type Output = Vec<Documented<StructMemberDeclaration>>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut members = vec![];

        while state.next.inner != Punctuator(CloseBrace) {
            let docstring = state.consume_docstring()?;

            members.push(
                StructMemberParser
                    .parse_with(state)?
                    .with_doc_comment(docstring),
            );
        }

        Ok(members)
    }
}

#[derive(Default)]
pub(crate) struct StructDeclarationParser {
    pub(crate) visibility: Visibility,
}

impl Parser for StructDeclarationParser {
    type Output = Item;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let name = state.consume_identifier("struct name in struct declaration")?;

        let generics = GenericsParser.optionally_parse_with(state)?;

        let r#where = WhereClauseParser.optionally_parse_with(state)?;

        state.next_token();

        let members = StructMembersParser.parse_with(state)?;

        state.consume(Punctuator(CloseBrace), "struct declaration")?;

        Ok(StructDeclarationItem {
            visibility: self.visibility,
            name,
            generics,
            r#where,
            members,
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::parser_test;

    parser_test!(StructDeclarationParser, empty_struct, "struct test {}");
    parser_test!(
        StructDeclarationParser,
        r#struct1,
        "struct Point[T: Numeric] { pub x: T; pub y: T; }"
    );
    parser_test!(
        StructDeclarationParser,
        r#struct2,
        "struct Lexer[S] where S: Iterator[char] { contents: S; }"
    );
}
