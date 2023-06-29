use crate::{Parse, ParseState};
use ry_ast::{Path, Token};
use ry_workspace::span::Span;

pub(crate) struct PathParser;

impl Parse for PathParser {
    type Output = Option<Path>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut identifiers = vec![];
        let first_identifier = state.consume_identifier("path")?;
        identifiers.push(first_identifier);

        let (start, mut end) = (first_identifier.span.start(), first_identifier.span.end());

        while state.next_token.raw == Token![.] {
            state.advance();
            identifiers.push(state.consume_identifier("path")?);
            end = state.current_token.span.end();
        }

        Some(Path {
            span: Span::new(start, end, state.file_id),
            identifiers,
        })
    }
}
