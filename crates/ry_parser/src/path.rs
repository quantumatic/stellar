use crate::{Cursor, Parse};
use ry_ast::{Path, Token};
use ry_span::{At, Span};

pub(crate) struct PathParser;

impl Parse for PathParser {
    type Output = Option<Path>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let mut path = vec![];
        let first_identifier = cursor.consume_identifier("path")?;
        path.push((*first_identifier.unwrap()).at(cursor.current.span()));

        let (start, mut end) = (
            first_identifier.span().start(),
            first_identifier.span().end(),
        );

        while *cursor.next.unwrap() == Token![.] {
            cursor.next_token();
            path.push((*cursor.consume_identifier("path")?.unwrap()).at(cursor.current.span()));
            end = cursor.current.span().end();
        }

        Some(path.at(Span::new(start, end, cursor.file_id)))
    }
}
