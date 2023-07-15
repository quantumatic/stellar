use ry_ast::{token::RawToken, Literal, Token};
use ry_diagnostics::BuildDiagnostic;

use crate::{diagnostics::ParseDiagnostic, Parse};

pub(crate) struct LiteralParser;

impl Parse for LiteralParser {
    type Output = Option<Literal>;

    fn parse(self, state: &mut crate::ParseState<'_, '_, '_>) -> Self::Output {
        match state.next_token.raw {
            RawToken::IntegerLiteral => {
                state.advance();
                if let Ok(value) = state.resolve_current().replace('_', "").parse::<u64>() {
                    Some(Literal::Integer {
                        value,
                        location: state.current_token.location,
                    })
                } else {
                    state.diagnostics.push(
                        ParseDiagnostic::IntegerOverflowError {
                            location: state.current_token.location,
                        }
                        .build(),
                    );
                    None
                }
            }
            RawToken::FloatLiteral => {
                state.advance();
                if let Ok(value) = state.resolve_current().replace('_', "").parse::<f64>() {
                    Some(Literal::Float {
                        value,
                        location: state.current_token.location,
                    })
                } else {
                    state.diagnostics.push(
                        ParseDiagnostic::FloatOverflowError {
                            location: state.current_token.location,
                        }
                        .build(),
                    );
                    None
                }
            }
            RawToken::StringLiteral => {
                state.advance();
                Some(Literal::String {
                    value: state.lexer.scanned_string(),
                    location: state.current_token.location,
                })
            }
            RawToken::CharLiteral => {
                state.advance();
                Some(Literal::Character {
                    value: state.lexer.scanned_char,
                    location: state.current_token.location,
                })
            }
            Token![true] => {
                state.advance();
                Some(Literal::Boolean {
                    value: true,
                    location: state.current_token.location,
                })
            }
            Token![false] => {
                state.advance();
                Some(Literal::Boolean {
                    value: false,
                    location: state.current_token.location,
                })
            }
            _ => {
                unreachable!()
            }
        }
    }
}
