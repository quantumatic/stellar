use crate::Parse;
use ry_ast::{token::RawToken, Literal, Token};
use ry_diagnostics::{parser::ParseDiagnostic, Report};

pub(crate) struct LiteralParser;

impl Parse for LiteralParser {
    type Output = Option<Literal>;

    fn parse_with(self, cursor: &mut crate::Cursor<'_>) -> Self::Output {
        match cursor.next.raw {
            RawToken::IntegerLiteral => {
                cursor.next_token();
                if let Ok(value) = cursor.resolve_current().replace('_', "").parse::<u64>() {
                    Some(Literal::Integer {
                        value,
                        span: cursor.current.span,
                    })
                } else {
                    cursor.diagnostics.push(
                        ParseDiagnostic::IntegerOverflowError {
                            span: cursor.current.span,
                        }
                        .build(),
                    );
                    None
                }
            }
            RawToken::FloatLiteral => {
                cursor.next_token();
                if let Ok(value) = cursor.resolve_current().replace('_', "").parse::<f64>() {
                    Some(Literal::Float {
                        value,
                        span: cursor.current.span,
                    })
                } else {
                    cursor.diagnostics.push(
                        ParseDiagnostic::FloatOverflowError {
                            span: cursor.current.span,
                        }
                        .build(),
                    );
                    None
                }
            }
            RawToken::StringLiteral => {
                cursor.next_token();
                Some(Literal::String {
                    value: cursor.lexer.scanned_string(),
                    span: cursor.current.span,
                })
            }
            RawToken::CharLiteral => {
                cursor.next_token();
                Some(Literal::Character {
                    value: cursor.lexer.scanned_char(),
                    span: cursor.current.span,
                })
            }
            Token![true] => {
                cursor.next_token();
                Some(Literal::Boolean {
                    value: true,
                    span: cursor.current.span,
                })
            }
            Token![false] => {
                cursor.next_token();
                Some(Literal::Boolean {
                    value: false,
                    span: cursor.current.span,
                })
            }
            _ => {
                unreachable!()
            }
        }
    }
}
