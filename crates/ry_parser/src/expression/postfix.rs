use crate::{error::ParseResult, Parser, ParserState};
use ry_ast::{
    expression::{Expression, RawExpression, UnaryExpression},
    span::{At, Span},
};

pub(crate) struct PostfixExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for PostfixExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = self.left.span().start();

        state.next_token();

        Ok(RawExpression::from(UnaryExpression {
            inner: Box::new(self.left),
            op: state.current.clone(),
            postfix: true,
        })
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}
