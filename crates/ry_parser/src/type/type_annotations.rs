use super::TypeParser;
use crate::{error::ParseResult, macros::parse_list, OptionalParser, Parser, ParserState};
use ry_ast::{r#type::TypeAnnotations, Token};

pub(crate) struct TypeAnnotationsParser;

impl OptionalParser for TypeAnnotationsParser {
    type Output = TypeAnnotations;

    fn optionally_parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if state.next.inner != Token!['['] {
            return Ok(vec![]);
        }

        self.parse_with(state)
    }
}

impl Parser for TypeAnnotationsParser {
    type Output = TypeAnnotations;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let result = parse_list!(state, "type annotations", Token![']'], || {
            TypeParser.parse_with(state)
        });

        state.next_token();

        Ok(result)
    }
}
