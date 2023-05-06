use crate::{error::ParseResult, Parser, ParserState};
use ry_ast::{
    expression::{Expression, PropertyAccessExpression, RawExpression},
    span::{At, Span},
};

pub(crate) struct PropertyAccessExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for PropertyAccessExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = self.left.span().start();

        state.next_token();

        Ok(RawExpression::from(PropertyAccessExpression {
            left: Box::new(self.left),
            property: state.consume_identifier("property")?,
        })
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}
