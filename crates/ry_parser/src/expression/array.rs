use super::ExpressionParser;
use crate::{error::ParseResult, macros::parse_list, Parser, ParserState};
use ry_ast::{
    expression::{ArrayLiteralExpression, Expression, RawExpression},
    span::At,
    token::{Punctuator::CloseBracket, RawToken::Punctuator},
};

pub(crate) struct ArrayLiteralExpressionParser;

impl Parser for ArrayLiteralExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let start = state.next.span.start;

        let literal = parse_list!(state, "array literal", Punctuator(CloseBracket), || {
            ExpressionParser::default().parse_with(state)
        });

        let end = state.current.span.end;

        Ok(RawExpression::from(ArrayLiteralExpression { literal }).at(start..end))
    }
}
