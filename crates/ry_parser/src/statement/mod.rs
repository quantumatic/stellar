pub(crate) mod defer;
pub(crate) mod r#return;
pub(crate) mod var;

use self::{defer::DeferStatementParser, r#return::ReturnStatementParser, var::VarStatementParser};
use crate::{expression::ExpressionParser, ParseResult, Parser, ParserState};
use ry_ast::{
    statement::{ExpressionStatement, Statement, StatementsBlock},
    token::{Keyword::*, Punctuator::*, RawToken::*},
};

pub(crate) struct StatementParser;

impl Parser for StatementParser {
    type Output = (Statement, bool);

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let statement = match state.next.inner {
            Keyword(Return) => ReturnStatementParser.parse_with(state)?,
            Keyword(Defer) => DeferStatementParser.parse_with(state)?,
            Keyword(Var) => VarStatementParser.parse_with(state)?,
            _ => {
                let expression = ExpressionParser::default().parse_with(state)?;

                must_have_semicolon_at_the_end = !expression.inner.with_block();

                match state.next.inner {
                    Punctuator(Semicolon) => {}
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
            state.consume(Punctuator(Semicolon), "end of the statement")?;
        }

        Ok((statement, last_statement_in_block))
    }
}

pub(crate) struct StatementsBlockParser;

impl Parser for StatementsBlockParser {
    type Output = StatementsBlock;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.consume(Punctuator(OpenBrace), "statements block")?;

        let mut block = vec![];

        while state.next.inner != Punctuator(CloseBrace) {
            let (statement, last) = StatementParser.parse_with(state)?;
            block.push(statement);

            if last {
                break;
            }
        }

        state.consume(Punctuator(CloseBrace), "end of the statements block")?;

        Ok(block)
    }
}
