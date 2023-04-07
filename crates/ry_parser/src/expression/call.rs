use super::ExpressionParser;
use crate::{error::ParseResult, macros::parse_list, Parser, ParserState};
use ry_ast::{
    expression::{CallExpression, Expression, RawExpression},
    precedence::Precedence,
    span::At,
    token::{Punctuator::CloseParent, RawToken::Punctuator},
};

pub(crate) struct CallExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for CallExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let arguments = parse_list!(
            state,
            "call arguments list",
            Punctuator(CloseParent),
            || ExpressionParser {
                precedence: Precedence::Lowest
            }
            .parse_with(state)
        );

        state.advance();

        let span = self.left.span.start..state.current.span.end;

        Ok(RawExpression::from(CallExpression {
            left: Box::new(self.left),
            arguments,
        })
        .at(span))
    }
}
