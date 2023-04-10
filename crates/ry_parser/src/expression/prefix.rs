use super::ExpressionParser;
use crate::{error::ParseResult, Parser, ParserState};
use ry_ast::{
    expression::{Expression, RawExpression, UnaryExpression},
    precedence::Precedence,
    span::At,
};

pub(crate) struct PrefixExpressionParser;

impl Parser for PrefixExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let op = state.next.clone();
        state.next_token();

        let inner = ExpressionParser {
            precedence: Precedence::Unary,
        }
        .parse_with(state)?;
        let span = op.span.start..inner.span.end;

        Ok(RawExpression::from(UnaryExpression {
            inner: Box::new(inner),
            op,
            postfix: false,
        })
        .at(span))
    }
}
