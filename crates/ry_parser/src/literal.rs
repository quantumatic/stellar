use crate::Parse;
use ry_ast::{token::RawToken, Literal, Token};
use ry_diagnostics::{parser::ParseDiagnostic, Report};
use ry_source_file::span::{At, SpanIndex, Spanned};

pub(crate) struct LiteralParser;

impl Parse for LiteralParser {
    type Output = Option<Spanned<Literal>>;

    fn parse_with(self, cursor: &mut crate::Cursor<'_>) -> Self::Output {
        match cursor.next.unwrap() {
            RawToken::IntegerLiteral => {
                cursor.next_token();
                if let Ok(integer) = cursor
                    .source
                    .index(cursor.current.span())
                    .replace('_', "")
                    .parse::<u64>()
                {
                    Some(Literal::Integer(integer).at(cursor.current.span()))
                } else {
                    cursor.diagnostics.push(
                        ParseDiagnostic::IntegerOverflowError {
                            at: cursor.current.span(),
                        }
                        .build(),
                    );
                    None
                }
            }
            RawToken::FloatLiteral => {
                cursor.next_token();
                if let Ok(float) = cursor
                    .source
                    .index(cursor.current.span())
                    .replace('_', "")
                    .parse::<f64>()
                {
                    Some(Literal::Float(float).at(cursor.current.span()))
                } else {
                    cursor.diagnostics.push(
                        ParseDiagnostic::FloatOverflowError {
                            at: cursor.current.span(),
                        }
                        .build(),
                    );
                    None
                }
            }
            RawToken::StringLiteral => {
                cursor.next_token();
                Some(
                    Literal::String(cursor.source.index(cursor.current.span()).to_owned())
                        .at(cursor.current.span()),
                )
            }
            RawToken::CharLiteral => {
                cursor.next_token();
                Some(
                    Literal::Character(cursor.source.index(cursor.current.span()).to_owned())
                        .at(cursor.current.span()),
                )
            }
            Token![true] => {
                cursor.next_token();
                Some(Literal::Boolean(true).at(cursor.current.span()))
            }
            Token![false] => {
                cursor.next_token();
                Some(Literal::Boolean(false).at(cursor.current.span()))
            }
            _ => {
                unreachable!()
            }
        }
    }
}
