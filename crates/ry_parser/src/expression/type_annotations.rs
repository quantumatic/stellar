use crate::{error::ParseResult, r#type::TypeAnnotationsParser, Parser, ParserState};
use ry_ast::{
    expression::{Expression, RawExpression, TypeAnnotationsExpression},
    span::At,
};

pub(crate) struct TypeAnnotationsExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for TypeAnnotationsExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let type_annotations = TypeAnnotationsParser.parse_with(state)?;

        let span = self.left.span.start..state.current.span.end;

        state.advance();

        Ok(RawExpression::from(TypeAnnotationsExpression {
            left: Box::new(self.left),
            type_annotations,
        })
        .at(span))
    }
}
