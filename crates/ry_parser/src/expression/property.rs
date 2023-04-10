use crate::{error::ParseResult, Parser, ParserState};
use ry_ast::{
    expression::{Expression, PropertyAccessExpression, RawExpression},
    span::At,
};

pub(crate) struct PropertyAccessExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for PropertyAccessExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let property = state.consume_identifier("property")?;

        let span = self.left.span.start..property.span.end;

        Ok(RawExpression::from(PropertyAccessExpression {
            left: Box::new(self.left),
            property,
        })
        .at(span))
    }
}
