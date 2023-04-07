use super::type_annotations::TypeAnnotationsParser;
use crate::{error::ParseResult, path::PathParser, OptionalParser, Parser, ParserState};
use ry_ast::{
    r#type::{PrimaryType, RawType, Type},
    span::At,
};

pub(crate) struct PrimaryTypeParser;

impl Parser for PrimaryTypeParser {
    type Output = Type;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = state.next.span.start;
        let path = PathParser.parse_with(state)?;
        let type_annotations = TypeAnnotationsParser.optionally_parse_with(state)?;

        Ok(RawType::from(PrimaryType {
            path,
            type_annotations,
        })
        .at(start..state.current.span.end))
    }
}
