use super::ExpressionParser;
use crate::{error::ParseResult, statement::StatementsBlockParser, Parser, ParserState};
use ry_ast::{
    expression::{Expression, RawExpression, WhileExpression},
    span::{At, Span},
};

pub(crate) struct WhileExpressionParser;

impl Parser for WhileExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();
        let start = state.current.span().start();

        let condition = ExpressionParser::default().parse_with(state)?;
        let body = StatementsBlockParser.parse_with(state)?;

        Ok(RawExpression::from(WhileExpression {
            condition: Box::new(condition),
            body,
        })
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}
