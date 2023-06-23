use crate::Parse;
use ry_ast::{token::RawToken, Literal, Token};
use ry_diagnostics::{parser::ParseDiagnostic, Report};

pub(crate) struct LiteralParser;

impl Parse for LiteralParser {
    type Output = Option<Literal>;

    fn parse_using(self, iterator: &mut crate::TokenIterator<'_>) -> Self::Output {
        match iterator.next.raw {
            RawToken::IntegerLiteral => {
                iterator.next_token();
                if let Ok(value) = iterator.resolve_current().replace('_', "").parse::<u64>() {
                    Some(Literal::Integer {
                        value,
                        span: iterator.current.span,
                    })
                } else {
                    iterator.diagnostics.push(
                        ParseDiagnostic::IntegerOverflowError {
                            span: iterator.current.span,
                        }
                        .build(),
                    );
                    None
                }
            }
            RawToken::FloatLiteral => {
                iterator.next_token();
                if let Ok(value) = iterator.resolve_current().replace('_', "").parse::<f64>() {
                    Some(Literal::Float {
                        value,
                        span: iterator.current.span,
                    })
                } else {
                    iterator.diagnostics.push(
                        ParseDiagnostic::FloatOverflowError {
                            span: iterator.current.span,
                        }
                        .build(),
                    );
                    None
                }
            }
            RawToken::StringLiteral => {
                iterator.next_token();
                Some(Literal::String {
                    value: iterator.lexer.scanned_string(),
                    span: iterator.current.span,
                })
            }
            RawToken::CharLiteral => {
                iterator.next_token();
                Some(Literal::Character {
                    value: iterator.lexer.scanned_char(),
                    span: iterator.current.span,
                })
            }
            Token![true] => {
                iterator.next_token();
                Some(Literal::Boolean {
                    value: true,
                    span: iterator.current.span,
                })
            }
            Token![false] => {
                iterator.next_token();
                Some(Literal::Boolean {
                    value: false,
                    span: iterator.current.span,
                })
            }
            _ => {
                unreachable!()
            }
        }
    }
}
