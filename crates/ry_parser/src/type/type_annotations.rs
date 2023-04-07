use super::TypeParser;
use crate::{error::ParseResult, macros::parse_list, OptionalParser, Parser, ParserState};
use ry_ast::{
    r#type::TypeAnnotations,
    token::{
        Punctuator::{CloseBracket, OpenBracket},
        RawToken::Punctuator,
    },
};

pub(crate) struct TypeAnnotationsParser;

impl OptionalParser for TypeAnnotationsParser {
    type Output = TypeAnnotations;

    fn optionally_parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if state.next.inner != Punctuator(OpenBracket) {
            return Ok(vec![]);
        }

        state.advance();

        let result = parse_list!(state, "generics", Punctuator(CloseBracket), || {
            TypeParser.parse_with(state)
        });

        state.advance();

        Ok(result)
    }
}
