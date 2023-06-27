use crate::Parse;
use ry_ast::{token::RawToken, Literal, Token};
use ry_diagnostics::{parser::ParseDiagnostic, BuildDiagnostic};

pub(crate) struct LiteralParser;

impl Parse for LiteralParser {
    type Output = Option<Literal>;

    fn parse(self, state: &mut crate::ParseState<'_>) -> Self::Output {
        match state.next_token.raw {
            RawToken::IntegerLiteral => {
                state.advance();
                if let Ok(value) = state.resolve_current().replace('_', "").parse::<u64>() {
                    Some(Literal::Integer {
                        value,
                        span: state.current_token.span,
                    })
                } else {
                    state.diagnostics.push(
                        ParseDiagnostic::IntegerOverflowError {
                            span: state.current_token.span,
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
                        span: state.current_token.span,
                    })
                } else {
                    state.diagnostics.push(
                        ParseDiagnostic::FloatOverflowError {
                            span: state.current_token.span,
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
                    span: state.current_token.span,
                })
            }
            RawToken::CharLiteral => {
                state.advance();
                Some(Literal::Character {
                    value: state.lexer.scanned_char(),
                    span: state.current_token.span,
                })
            }
            Token![true] => {
                state.advance();
                Some(Literal::Boolean {
                    value: true,
                    span: state.current_token.span,
                })
            }
            Token![false] => {
                state.advance();
                Some(Literal::Boolean {
                    value: false,
                    span: state.current_token.span,
                })
            }
            _ => {
                unreachable!()
            }
        }
    }
}
