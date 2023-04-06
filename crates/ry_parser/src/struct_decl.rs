use crate::{ParseResult, Parser, ParserState};
use ry_ast::{
    declaration::{Item, StructDeclarationItem},
    token::{Punctuator::*, RawToken::*},
    Visibility,
};

pub(crate) struct StructDeclarationParser {
    visibility: Visibility,
}

impl Parser for StructDeclarationParser {
    type Output = Item;

    fn parse_with(self, parser: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        parser.advance();

        let name = parser.consume_identifier("struct name in struct declaration")?;

        let generics = parser.optionally_parse_generics()?;

        let r#where = parser.optionally_parse_where_clause()?;

        parser.advance_with_docstring();

        let members = parser.parse_struct_members()?;

        parser.consume_with_docstring(Punctuator(CloseBrace), "struct declaration")?;

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
