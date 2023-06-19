use crate::{Cursor, Parse};
use ry_ast::{Path, Token};
use ry_source_file::span::Span;

pub(crate) struct PathParser;

impl Parse for PathParser {
    type Output = Option<Path>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let mut symbols = vec![];
        let first_identifier = cursor.consume_identifier("path")?;
        symbols.push(first_identifier);

        let (start, mut end) = (first_identifier.span.start(), first_identifier.span.end());

        while cursor.next.raw == Token![.] {
            cursor.next_token();
            symbols.push(cursor.consume_identifier("path")?);
            end = cursor.current.span.end();
        }

        Some(Path {
            span: Span::new(start, end, cursor.file_id),
            symbols,
        })
    }
}
