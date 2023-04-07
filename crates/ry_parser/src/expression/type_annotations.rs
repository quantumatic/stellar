use crate::{error::ParseResult, macros::parse_list, r#type::TypeParser, Parser, ParserState};
use ry_ast::{
    expression::{Expression, RawExpression, TypeAnnotationsExpression},
    span::At,
    token::{Punctuator::CloseBracket, RawToken::Punctuator},
};

pub(crate) struct TypeAnnotationsExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for TypeAnnotationsExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let type_annotations =
            parse_list!(state, "type annotations", Punctuator(CloseBracket), || {
                TypeParser.parse_with(state)
            });

        let span = self.left.span.start..state.current.span.end;

        state.advance();

        Ok(RawExpression::from(TypeAnnotationsExpression {
            left: Box::new(self.left),
            type_annotations,
        })
        .at(span))
    }
}
