mod array;
mod generics;
mod primary;
mod reference;
mod type_annotations;
mod where_clause;

pub(crate) use self::{
    array::ArrayTypeParser, generics::*, primary::PrimaryTypeParser,
    reference::ReferenceTypeParser, type_annotations::TypeAnnotationsParser,
    where_clause::WhereClauseParser,
};
use crate::{
    error::{expected, ParseError, ParseResult},
    Parser, ParserState,
};
use ry_ast::{
    r#type::Type,
    token::{
        Punctuator::{And, OpenBracket},
        RawToken::{Identifier, Punctuator},
    },
};

#[derive(Default)]
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

#[cfg(test)]
mod tests {
    use super::TypeParser;
    use crate::{macros::parser_test, Parser, ParserState};
    use ry_interner::Interner;

    parser_test!(TypeParser, primary1, "i32");
    parser_test!(TypeParser, primary2, "Result[T, DivisionError]");
    parser_test!(TypeParser, array, "[i32]");
    parser_test!(TypeParser, imut_reference, "&i32");
    parser_test!(TypeParser, mut_reference, "&mut i32");
}
