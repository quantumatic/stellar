use crate::{expression::ExpressionParser, Cursor, Parse, ParseResult};
use ry_ast::{Statement, StatementsBlock, Token};

struct StatementParser;

pub(crate) struct StatementsBlockParser;

struct DeferStatementParser;

struct ReturnStatementParser;

impl Parse for StatementParser {
    type Output = (Statement, bool);

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let statement = match cursor.next.unwrap() {
            Token![return] => ReturnStatementParser.parse_with(cursor)?,
            Token![defer] => DeferStatementParser.parse_with(cursor)?,
            _ => {
                let expression = ExpressionParser::default().parse_with(cursor)?;

                must_have_semicolon_at_the_end = !expression.unwrap().with_block();

                match cursor.next.unwrap() {
                    Token![;] => {}
                    _ => {
                        if must_have_semicolon_at_the_end {
                            last_statement_in_block = true;
                        }
                    }
                }

                if last_statement_in_block || !must_have_semicolon_at_the_end {
                    Statement::Expression {
                        has_semicolon: false,
                        expression,
                    }
                } else {
                    Statement::Expression {
                        has_semicolon: true,
                        expression,
                    }
                }
            }
        };

        if !last_statement_in_block && must_have_semicolon_at_the_end {
            cursor.consume(Token![;], "end of the statement")?;
        }

        Ok((statement, last_statement_in_block))
    }
}

impl Parse for StatementsBlockParser {
    type Output = StatementsBlock;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.consume(Token!['{'], "statements block")?;

        let mut block = vec![];

        while *cursor.next.unwrap() != Token!['}'] {
            let (statement, last) = StatementParser.parse_with(cursor)?;
            block.push(statement);

            if last {
                break;
            }
        }

        cursor.consume(Token!['}'], "end of the statements block")?;

        Ok(block)
    }
}

impl Parse for DeferStatementParser {
    type Output = Statement;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token();

        Ok(Statement::Defer {
            call: ExpressionParser::default().parse_with(cursor)?,
        })
    }
}

impl Parse for ReturnStatementParser {
    type Output = Statement;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token();

        Ok(Statement::Return {
            expression: ExpressionParser::default().parse_with(cursor)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::StatementParser;
    use crate::macros::parse_test;

    parse_test!(StatementParser, r#return, "return a?.b.unwrap_or(0);");
    parse_test!(StatementParser, defer, "defer call();");
}
