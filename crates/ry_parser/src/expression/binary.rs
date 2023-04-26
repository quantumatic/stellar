use super::ExpressionParser;
use crate::{error::ParseResult, Parser, ParserState};
use ry_ast::{
    expression::{BinaryExpression, Expression, RawExpression},
    span::At,
};

pub(crate) struct BinaryExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for BinaryExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = self.left.span().start();

        let op = state.next.clone();
        let precedence = state.next.unwrap().to_precedence();

        state.next_token();

        let right = ExpressionParser { precedence }.parse_with(state)?;

        Ok(RawExpression::from(BinaryExpression {
            left: Box::new(self.left),
            right: Box::new(right),
            op,
        })
        .at(start..state.current.span().end()))
    }
}
