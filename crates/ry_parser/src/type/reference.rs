use super::TypeParser;
use crate::{error::ParseResult, Parser, ParserState};
use ry_ast::{
    r#type::{RawType, ReferenceType, Type},
    span::{At, Span},
};

pub(crate) struct ReferenceTypeParser;

impl Parser for ReferenceTypeParser {
    type Output = Type;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let start = state.current.span().start();

        let inner = TypeParser.parse_with(state)?;

        Ok(RawType::from(ReferenceType {
            inner: Box::new(inner),
        })
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}
