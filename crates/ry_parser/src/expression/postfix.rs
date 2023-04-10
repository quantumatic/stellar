use crate::{error::ParseResult, Parser, ParserState};
use ry_ast::{
    expression::{Expression, RawExpression, UnaryExpression},
    span::At,
};

pub(crate) struct PostfixExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for PostfixExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let op = state.current.clone();

        let span = self.left.span.start..op.span.end;

        Ok(RawExpression::from(UnaryExpression {
            inner: Box::new(self.left),
            op,
            postfix: true,
        })
        .at(span))
    }
}
