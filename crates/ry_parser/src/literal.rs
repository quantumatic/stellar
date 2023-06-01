use crate::Parse;
use ry_ast::{
    span::{At, SpanIndex, Spanned},
    token::RawToken,
    Literal, Token,
};
use ry_diagnostics::{parser::ParseDiagnostic, Report};

pub(crate) struct LiteralParser;

impl Parse for LiteralParser {
    type Output = Option<Spanned<Literal>>;

    fn parse_with(self, cursor: &mut crate::Cursor<'_>) -> Self::Output {
        match cursor.next.unwrap() {
            RawToken::IntegerLiteral => {
                cursor.next_token();
                match cursor
                    .contents
                    .index(cursor.current.span())
                    .replace('_', "")
                    .parse::<u64>()
                {
                    Ok(integer) => Some(Literal::Integer(integer).at(cursor.current.span())),
                    Err(..) => {
                        cursor.diagnostics.push(
                            ParseDiagnostic::IntegerOverflowError {
                                at: cursor.current.span(),
                            }
                            .build(),
                        );
                        None
                    }
                }
            }
            RawToken::FloatLiteral => {
                cursor.next_token();
                match cursor
                    .contents
                    .index(cursor.current.span())
                    .replace('_', "")
                    .parse::<f64>()
                {
                    Ok(float) => Some(Literal::Float(float).at(cursor.current.span())),
                    Err(..) => {
                        cursor.diagnostics.push(
                            ParseDiagnostic::FloatOverflowError {
                                at: cursor.current.span(),
                            }
                            .build(),
                        );
                        None
                    }
                }
            }
            RawToken::StringLiteral => {
                cursor.next_token();
                Some(
                    Literal::String(cursor.contents.index(cursor.current.span()).to_owned())
                        .at(cursor.current.span()),
                )
            }
            RawToken::CharLiteral => {
                cursor.next_token();
                Some(
                    Literal::String(cursor.contents.index(cursor.current.span()).to_owned())
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

#[cfg(test)]
mod tests {
    use super::LiteralParser;
    use crate::macros::parse_test;

    parse_test!(LiteralParser, integer, "12345");
    parse_test!(LiteralParser, float, "12.5e+3");
    parse_test!(LiteralParser, string, "\"hello world!\"");
    parse_test!(LiteralParser, char, "'a'");
    parse_test!(LiteralParser, bool, "true");
}
