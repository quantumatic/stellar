use super::TypeParser;
use crate::{error::ParseResult, Parser, ParserState};
use ry_ast::{
    r#type::{RawType, ReferenceType, Type},
    span::At,
    token::{Keyword::Mut, RawToken::Keyword},
    Mutability,
};

pub(crate) struct ReferenceTypeParser;

impl Parser for ReferenceTypeParser {
    type Output = Type;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let start = state.current.span.start;

        let mut mutability = Mutability::immutable();

        if state.next.inner == Keyword(Mut) {
            mutability = Mutability::mutable(state.next.span);

            state.advance();
        }

        let inner = TypeParser.parse_with(state)?;

        Ok(RawType::from(ReferenceType {
            mutability,
            inner: Box::new(inner),
        })
        .at(start..state.current.span.end))
    }
}
