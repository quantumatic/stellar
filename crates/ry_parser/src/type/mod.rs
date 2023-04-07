pub(crate) mod array;
pub(crate) mod generics;
pub(crate) mod primary;
pub(crate) mod reference;
pub(crate) mod type_annotations;
pub(crate) mod where_clause;

use self::{array::ArrayTypeParser, primary::PrimaryTypeParser, reference::ReferenceTypeParser};
use crate::{
    error::{expected, ParseError, ParseResult},
    Parser, ParserState,
};
use ry_ast::{
    r#type::Type,
    token::{Punctuator::*, RawToken::*},
};

pub(crate) struct TypeParser;

impl Parser for TypeParser {
    type Output = Type;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let r#type = match state.next.inner {
            Identifier(..) => PrimaryTypeParser.parse_with(state)?,
            Punctuator(And) => ReferenceTypeParser.parse_with(state)?,
            Punctuator(OpenBracket) => ArrayTypeParser.parse_with(state)?,
            _ => {
                return Err(ParseError::unexpected_token(
                    state.next.clone(),
                    expected!("identifier", Punctuator(And), Punctuator(OpenBracket)),
                    "type",
                ));
            }
        };

        Ok(r#type)
    }
}

// #[cfg(test)]
// mod type_tests {
//     use crate::{macros::parser_test, Parser};
//     use ry_interner::Interner;

//     parser_test!(primary_type1, "pub fun test(): i32 {}");
//     parser_test!(
//         primary_type2,
//         "pub fun div[T](a: T, b: T): Result[T, DivisionError] {}"
//     );
//     parser_test!(array_type, "pub fun test(a: [i32]) {}");
//     parser_test!(reference_type, "pub fun test(a: &mut i32): i32 {}");
//     parser_test!(negative_trait_type, "pub fun test(a: Into[string]) {}");
// }
