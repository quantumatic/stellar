use ry_ast::{expression::{WhileExpression, RawExpression, Expression}, span::At};

use crate::{statement::StatementsBlockParser, Parser, ParserState, error::ParseResult};

use super::ExpressionParser;

pub(crate) struct WhileExpressionParser;

impl Parser for WhileExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();
        let start = state.current.span.start;

        let condition = ExpressionParser::default().parse_with(state)?;
        let body = StatementsBlockParser.parse_with(state)?;

        Ok(RawExpression::from(WhileExpression {
            condition: Box::new(condition),
            body,
        })
        .at(start..state.current.span.end))
    }
}