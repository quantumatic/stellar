use super::TypeParser;
use crate::{error::ParseResult, macros::parse_list, OptionalParser, Parser, ParserState};
use ry_ast::{
    r#type::{Generic, Generics},
    token::{
        Punctuator::{CloseBracket, Colon, OpenBracket},
        RawToken::Punctuator,
    },
};

pub(crate) struct GenericsParser;

impl OptionalParser for GenericsParser {
    type Output = Generics;

    fn optionally_parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if state.next.inner != Punctuator(OpenBracket) {
            return Ok(vec![]);
        }

        state.advance();

        let result = parse_list!(
            state,
            "generics",
            Punctuator(CloseBracket),
            || -> ParseResult<Generic> {
                let name = state.consume_identifier("generic name")?;

                let mut constraint = None;

                if state.next.inner == Punctuator(Colon) {
                    state.advance();
                    constraint = Some(TypeParser.parse_with(state)?);
                }

                Ok(Generic { name, constraint })
            }
        );

        state.advance();

        Ok(result)
    }
}
