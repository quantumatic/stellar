use ry_ast::{ImportPath, Path, Token};

use crate::{Parse, ParseState};

pub(crate) struct PathParser;

impl Parse for PathParser {
    type Output = Option<Path>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut identifiers = vec![];

        let first_identifier = state.consume_identifier("path")?;
        identifiers.push(first_identifier);

        let start = first_identifier.location.start;

        while state.next_token.raw == Token![.] {
            state.advance();
            identifiers.push(state.consume_identifier("path")?);
        }

        Some(Path {
            location: state.location_from(start),
            identifiers,
        })
    }
}

pub(crate) struct ImportPathParser;

impl Parse for ImportPathParser {
    type Output = Option<ImportPath>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let path = PathParser.parse(state)?;

        let r#as = if state.next_token.raw == Token![as] {
            state.advance();

            Some(state.consume_identifier("import path")?)
        } else {
            None
        };

        Some(ImportPath { path, r#as })
    }
}
