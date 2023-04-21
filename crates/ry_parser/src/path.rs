use crate::{error::ParseResult, Parser, ParserState};
use ry_ast::{name::Path, span::At, Token};

pub(crate) struct PathParser;

impl Parser for PathParser {
    type Output = Path;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut path = vec![];
        let first_identifier = state.consume_identifier("path")?;
        path.push(first_identifier.inner);

        let (start, mut end) = (first_identifier.span.start, first_identifier.span.end);

        while state.next.inner == Token![.] {
            state.next_token();
            path.push(state.consume_identifier("path")?.inner);
            end = state.current.span.end;
        }

        Ok(path.at(start..end))
    }
}
