use crate::{
    r#type::{GenericsParser, TypeParser, WhereClauseParser},
    OptionalParser, ParseResult, Parser, ParserState,
};
use ry_ast::{
    declaration::{Documented, StructDeclarationItem, StructMemberDeclaration, WithDocstring},
    token::{Keyword::*, Punctuator::*, RawToken::*},
    Mutability, Visibility,
};

pub(crate) struct StructMemberParser;

impl Parser for StructMemberParser {
    type Output = StructMemberDeclaration;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut visibility = Visibility::private();
        let mut mutability = Mutability::immutable();

        if state.next.inner == Keyword(Pub) {
            state.advance();
            visibility = Visibility::public(state.current.span);
        }

        if state.next.inner == Keyword(Mut) {
            state.advance();
            mutability = Mutability::mutable(state.current.span);
        }

        let name = state.consume_identifier("struct member name in struct definition")?;

        state.consume(Punctuator(Colon), "struct member definition")?;

        let r#type = TypeParser.parse_with(state)?;

        state.consume(Punctuator(Semicolon), "struct member definition")?;

        Ok(StructMemberDeclaration {
            visibility,
            mutability,
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
                    .with_docstring(docstring),
            );
        }

        Ok(members)
    }
}

pub(crate) struct StructDeclarationParser {
    pub(crate) visibility: Visibility,
}

impl Parser for StructDeclarationParser {
    type Output = StructDeclarationItem;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let name = state.consume_identifier("struct name in struct declaration")?;

        let generics = GenericsParser.optionally_parse_with(state)?;

        let r#where = WhereClauseParser.optionally_parse_with(state)?;

        state.advance();

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

//     fn parse_struct_member(&mut self) -> ParseResult<StructMemberDeclaration> {
//         let mut visibility = Visibility::private();
//         let (mut invalid_mutability, mut mutability) =
//             (Mutability::immutable(), Mutability::immutable());

//         if let Keyword(Mut) = self.next.inner {
//             self.advance();
//             mutability = Mutability::mutable(self.current.span);
//         }

//         if let Keyword(Pub) = self.next.inner {
//             self.advance();
//             visibility = Visibility::public(self.current.span);
//         }

//         if let Keyword(Mut) = self.next.inner {
//             self.advance();
//             invalid_mutability = Mutability::mutable(self.current.span);
//         }

//         let name = self.consume_identifier("struct member name in struct definition")?;

//         self.consume(Punctuator(Colon), "struct member definition")?;

//         let r#type = self.parse_type()?;

//         self.consume(Punctuator(Semicolon), "struct member definition")?;

//         Ok(StructMemberDeclaration {
//             visibility,
//             mutability,
//             invalid_mutability,
//             name,
//             r#type,
//         })
//     }

//     fn parse_struct_members(&mut self) -> ParseResult<Vec<Documented<StructMemberDeclaration>>> {
//         let mut members = vec![];

//         loop {
//             if self.next.inner == Punctuator(CloseBrace) {
//                 break;
//             }

//             let docstring = self.consume_non_module_docstring()?;
//             let member = self.parse_struct_member()?;

//             members.push(member.with_docstring(docstring));
//         }

//         Ok(members)
//     }
// }

// #[cfg(test)]
// mod struct_tests {
//     use crate::{macros::parser_test, Parser};
//     use ry_interner::Interner;

//     parser_test!(empty_struct, "struct test {}");
//     parser_test!(
//         r#struct,
//         "struct test[T, M] { pub mut a: i32; mut pub b: T; pub c: T; d: M; }"
//     );
// }
