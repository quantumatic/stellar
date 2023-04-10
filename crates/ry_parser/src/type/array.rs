use super::TypeParser;
use crate::{error::ParseResult, Parser, ParserState};
use ry_ast::{
    r#type::{ArrayType, RawType, Type},
    span::At,
    token::{Punctuator::CloseBracket, RawToken::Punctuator},
};

pub(crate) struct ArrayTypeParser;

impl Parser for ArrayTypeParser {
    type Output = Type;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();
        let start = state.current.span.start;

        let inner = TypeParser.parse_with(state)?;

        state.consume(Punctuator(CloseBracket), "array type")?;

        Ok(RawType::from(ArrayType {
            inner: Box::new(inner),
        })
        .at(start..state.current.span.end))
    }
}
