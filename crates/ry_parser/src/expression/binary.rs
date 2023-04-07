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
        let op = state.next.clone();
        let precedence = state.next.inner.to_precedence();

        state.advance();

        let right = ExpressionParser { precedence }.parse_with(state)?;

        let span = self.left.span.start..state.current.span.end;

        Ok(RawExpression::from(BinaryExpression {
            left: Box::new(self.left),
            right: Box::new(right),
            op,
        })
        .at(span))
    }
}
