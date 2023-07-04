use crate::{Parse, ParseState};
use ry_ast::{token::RawToken, ImportPath, Path, Token};
use ry_diagnostics::{expected, parser::ParseDiagnostic, BuildDiagnostic};

pub(crate) struct PathParser;

impl Parse for PathParser {
    type Output = Option<Path>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut identifiers = vec![];
        let first_identifier = state.consume_identifier("path")?;
        identifiers.push(first_identifier);

        let start = first_identifier.span.start;

        while state.next_token.raw == Token![.] {
            state.advance();
            identifiers.push(state.consume_identifier("path")?);
        }

        Some(Path {
            span: state.span_to_current_from(start),
            identifiers,
        })
    }
}

pub(crate) struct ImportPathParser;

impl Parse for ImportPathParser {
    type Output = Option<ImportPath>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut star_span = None;
        let mut identifiers = vec![];
        let first_identifier = state.consume_identifier("import path")?;
        identifiers.push(first_identifier);

        let start = first_identifier.span.start;

        while state.next_token.raw == Token![.] {
            state.advance();

            match state.next_token.raw {
                RawToken::Identifier => identifiers.push(state.consume_identifier_or_panic()),
                Token![*] => {
                    star_span = Some(state.next_token.span);
                    state.advance();
                    break;
                }
                _ => {
                    state.diagnostics.push(
                        ParseDiagnostic::UnexpectedTokenError {
                            got: state.next_token,
                            expected: expected!("*", "identifier"),
                            node: "import path".to_owned(),
                        }
                        .build(),
                    );

                    return None;
                }
            }
        }

        let identifiers = Path {
            span: state.span_to_current_from(start),
            identifiers,
        };

        let r#as = if state.next_token.raw == Token![as] {
            state.advance();

            Some(state.consume_identifier("import path")?)
        } else {
            None
        };

        Some(ImportPath {
            left: identifiers,
            r#as,
            star_span,
        })
    }
}
