mod array;
mod generics;
mod primary;
mod reference;
mod type_annotations;
mod where_clause;

pub(crate) use self::{
    array::ArrayTypeParser, generics::*, primary::PrimaryTypeParser,
    type_annotations::TypeAnnotationsParser, where_clause::WhereClauseParser,
};
use crate::{
    error::{expected, ParseError, ParseResult},
    Parser, ParserState,
};
use ry_ast::{r#type::Type, token::RawToken, Token};

#[derive(Default)]
pub(crate) struct TypeParser;

impl Parser for TypeParser {
    type Output = Type;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let r#type = match state.next.unwrap() {
            RawToken::Identifier(..) => PrimaryTypeParser.parse_with(state)?,
            Token!['['] => ArrayTypeParser.parse_with(state)?,
            _ => {
                return Err(ParseError::unexpected_token(
                    state.next.clone(),
                    expected!("identifier", Token![&], Token!['[']),
                    "type",
                ));
            }
        };

        Ok(r#type)
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::parser_test;

    parser_test!(TypeParser, primary1, "i32");
    parser_test!(TypeParser, primary, "Result[T, DivisionError]");
    parser_test!(TypeParser, array, "[i32]");
}
