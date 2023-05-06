use crate::{error::ParseResult, Parser, ParserState};
use ry_ast::{
    name::Path,
    span::{At, Span},
    Token,
};

pub(crate) struct PathParser;

impl Parser for PathParser {
    type Output = Path;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut path = vec![];
        let first_identifier = state.consume_identifier("path")?;
        path.push((*first_identifier.unwrap()).at(state.current.span()));

        let (start, mut end) = (
            first_identifier.span().start(),
            first_identifier.span().end(),
        );

        while *state.next.unwrap() == Token![.] {
            state.next_token();
            path.push((*state.consume_identifier("path")?.unwrap()).at(state.current.span()));
            end = state.current.span().end();
        }

        Ok(path.at(Span::new(start, end, state.file_id)))
    }
}
