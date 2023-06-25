use crate::Parse;
use ry_ast::{token::RawToken, Literal, Token};
use ry_diagnostics::{parser::ParseDiagnostic, BuildDiagnostic};

pub(crate) struct LiteralParser;

impl Parse for LiteralParser {
    type Output = Option<Literal>;

    fn parse_using(self, iterator: &mut crate::TokenIterator<'_>) -> Self::Output {
        match iterator.next_token.raw {
            RawToken::IntegerLiteral => {
                iterator.advance();
                if let Ok(value) = iterator.resolve_current().replace('_', "").parse::<u64>() {
                    Some(Literal::Integer {
                        value,
                        span: iterator.current_token.span,
                    })
                } else {
                    iterator.diagnostics.push(
                        ParseDiagnostic::IntegerOverflowError {
                            span: iterator.current_token.span,
                        }
                        .build(),
                    );
                    None
                }
            }
            RawToken::FloatLiteral => {
                iterator.advance();
                if let Ok(value) = iterator.resolve_current().replace('_', "").parse::<f64>() {
                    Some(Literal::Float {
                        value,
                        span: iterator.current_token.span,
                    })
                } else {
                    iterator.diagnostics.push(
                        ParseDiagnostic::FloatOverflowError {
                            span: iterator.current_token.span,
                        }
                        .build(),
                    );
                    None
                }
            }
            RawToken::StringLiteral => {
                iterator.advance();
                Some(Literal::String {
                    value: iterator.lexer.scanned_string(),
                    span: iterator.current_token.span,
                })
            }
            RawToken::CharLiteral => {
                iterator.advance();
                Some(Literal::Character {
                    value: iterator.lexer.scanned_char(),
                    span: iterator.current_token.span,
                })
            }
            Token![true] => {
                iterator.advance();
                Some(Literal::Boolean {
                    value: true,
                    span: iterator.current_token.span,
                })
            }
            Token![false] => {
                iterator.advance();
                Some(Literal::Boolean {
                    value: false,
                    span: iterator.current_token.span,
                })
            }
            _ => {
                unreachable!()
            }
        }
    }
}
