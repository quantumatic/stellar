use crate::{expression::ExpressionParser, ParseResult, Parser, ParserState};
use ry_ast::{Statement, StatementsBlock, Token};

pub(crate) struct StatementParser;

impl Parser for StatementParser {
    type Output = (Statement, bool);

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let statement = match state.next.unwrap() {
            Token![return] => ReturnStatementParser.parse_with(state)?,
            Token![defer] => DeferStatementParser.parse_with(state)?,
            _ => {
                let expression = ExpressionParser::default().parse_with(state)?;

                must_have_semicolon_at_the_end = !expression.unwrap().with_block();

                match state.next.unwrap() {
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
            state.consume(Token![;], "end of the statement")?;
        }

        Ok((statement, last_statement_in_block))
    }
}

pub(crate) struct StatementsBlockParser;

impl Parser for StatementsBlockParser {
    type Output = StatementsBlock;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.consume(Token!['{'], "statements block")?;

        let mut block = vec![];

        while *state.next.unwrap() != Token!['}'] {
            let (statement, last) = StatementParser.parse_with(state)?;
            block.push(statement);

            if last {
                break;
            }
        }

        state.consume(Token!['}'], "end of the statements block")?;

        Ok(block)
    }
}

#[derive(Default)]
pub(crate) struct DeferStatementParser;

impl Parser for DeferStatementParser {
    type Output = Statement;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        Ok(Statement::Defer {
            call: ExpressionParser::default().parse_with(state)?,
        })
    }
}

#[derive(Default)]
pub(crate) struct ReturnStatementParser;

impl Parser for ReturnStatementParser {
    type Output = Statement;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        Ok(Statement::Return {
            expression: ExpressionParser::default().parse_with(state)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::parser_test;

    parser_test!(ReturnStatementParser, r#return, "return a?.b.unwrap_or(0);");
    parser_test!(DeferStatementParser, defer, "defer call();");
}
