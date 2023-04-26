use super::ExpressionParser;
use crate::{error::ParseResult, macros::parse_list, Parser, ParserState};
use ry_ast::{
    expression::{CallExpression, Expression, RawExpression},
    precedence::Precedence,
    span::At,
    Token,
};

pub(crate) struct CallExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for CallExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = self.left.span().start();

        state.next_token();

        let arguments = parse_list!(state, "call arguments list", Token![')'], || {
            ExpressionParser {
                precedence: Precedence::Lowest,
            }
            .parse_with(state)
        });

        state.next_token();

        Ok(RawExpression::from(CallExpression {
            left: Box::new(self.left),
            arguments,
        })
        .at(start..state.current.span().end()))
    }
}
