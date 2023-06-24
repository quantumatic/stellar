use crate::{Parse, TokenIterator};
use ry_ast::{Path, Token};
use ry_source_file::span::Span;

pub(crate) struct PathParser;

impl Parse for PathParser {
    type Output = Option<Path>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let mut identifiers = vec![];
        let first_identifier = iterator.consume_identifier("path")?;
        identifiers.push(first_identifier);

        let (start, mut end) = (first_identifier.span.start(), first_identifier.span.end());

        while iterator.next_token.raw == Token![.] {
            iterator.advance();
            identifiers.push(iterator.consume_identifier("path")?);
            end = iterator.current_token.span.end();
        }

        Some(Path {
            span: Span::new(start, end, iterator.file_id),
            identifiers,
        })
    }
}
