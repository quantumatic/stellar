mod defer;
mod r#return;
mod var;

use self::{defer::DeferStatementParser, r#return::ReturnStatementParser, var::VarStatementParser};
use crate::{expression::ExpressionParser, ParseResult, Parser, ParserState};
use ry_ast::{
    statement::{ExpressionStatement, Statement, StatementsBlock},
    Token,
};

pub(crate) struct StatementParser;

impl Parser for StatementParser {
    type Output = (Statement, bool);

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let statement = match state.next.inner {
            Token![return] => ReturnStatementParser.parse_with(state)?,
            Token![defer] => DeferStatementParser.parse_with(state)?,
            Token![var] => VarStatementParser.parse_with(state)?,
            _ => {
                let expression = ExpressionParser::default().parse_with(state)?;

                must_have_semicolon_at_the_end = !expression.inner.with_block();

                match state.next.inner {
                    Token![;] => {}
                    _ => {
                        if must_have_semicolon_at_the_end {
                            last_statement_in_block = true;
                        }
                    }
                }

                if last_statement_in_block || !must_have_semicolon_at_the_end {
                    ExpressionStatement {
                        has_semicolon: false,
                        expression,
                    }
                    .into()
                } else {
                    ExpressionStatement {
                        has_semicolon: true,
                        expression,
                    }
                    .into()
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

        while state.next.inner != Token!['}'] {
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
