use super::ExpressionParser;
use crate::{error::ParseResult, Parser, ParserState};
use ry_ast::{
    expression::Expression,
    precedence::Precedence,
    Token
};

pub(crate) struct ParenthesizedExpressionParser;

impl Parser for ParenthesizedExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let expression = ExpressionParser {
            precedence: Precedence::Lowest,
        }
        .parse_with(state)?;

        state.consume(Token![')'], "parenthesized expression")?;

        Ok(expression)
    }
}
