use crate::{error::ParseResult, r#type::TypeParser, Parser, ParserState};
use ry_ast::{
    expression::{AsExpression, Expression, RawExpression},
    span::At,
};

pub(crate) struct CastExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for CastExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let right = TypeParser.parse_with(state)?;

        let span = self.left.span.start..state.current.span.end;

        Ok(RawExpression::from(AsExpression {
            left: Box::new(self.left),
            right,
        })
        .at(span))
    }
}
