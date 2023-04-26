use super::ExpressionParser;
use crate::{error::ParseResult, Parser, ParserState};
use ry_ast::{
    expression::{Expression, ParenthesizedExpression, RawExpression},
    precedence::Precedence,
    span::At,
    Token,
};

pub(crate) struct ParenthesizedExpressionParser;

impl Parser for ParenthesizedExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();
        let start = state.current.span().start();

        let inner = ExpressionParser {
            precedence: Precedence::Lowest,
        }
        .parse_with(state)?;

        state.consume(Token![')'], "parenthesized expression")?;

        Ok(RawExpression::from(ParenthesizedExpression {
            inner: Box::new(inner),
        })
        .at(start..state.current.span().end()))
    }
}
