use super::TypeParser;
use crate::{error::ParseResult, macros::parse_list, OptionalParser, Parser, ParserState};
use ry_ast::{
    r#type::{Generic, Generics},
    Token,
};

pub(crate) struct GenericsParser;

impl OptionalParser for GenericsParser {
    type Output = Generics;

    fn optionally_parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if state.next.inner != Token!['['] {
            return Ok(vec![]);
        }

        state.next_token();

        let result = parse_list!(
            state,
            "generics",
            Token![']'],
            || -> ParseResult<Generic> {
                let name = state.consume_identifier("generic name")?;

                let constraint = if state.next.inner == Token![:] {
                    state.next_token();
                    Some(TypeParser.parse_with(state)?)
                } else {
                    None
                };

                Ok(Generic { name, constraint })
            }
        );

        state.next_token();

        Ok(result)
    }
}
