use super::{function::FunctionParser, type_alias::TypeAliasParser};
use crate::{
    error::{expected, ParseError, ParseResult},
    Parser, ParserState,
};
use ry_ast::{
    declaration::{Documented, TraitItem, WithDocComment},
    token::{Keyword::*, RawToken::Keyword},
    Token, Visibility,
};

pub(crate) struct AssociatedFunctionsParser;

impl Parser for AssociatedFunctionsParser {
    type Output = Vec<Documented<TraitItem>>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut items = vec![];

        while state.next.inner != Token!['}'] {
            let doc = state.consume_docstring()?;

            let visibility = if state.next.inner == Token![pub] {
                state.next_token();
                Visibility::public(state.current.span)
            } else {
                Visibility::private()
            };

            items.push(match state.next.inner {
                Keyword(Fun) => Ok(TraitItem::from(
                    FunctionParser { visibility }.parse_with(state)?,
                )
                .with_doc_comment(doc)),
                Keyword(Type) => Ok(TraitItem::from(
                    TypeAliasParser { visibility }.parse_with(state)?,
                )
                .with_doc_comment(doc)),
                _ => Err(ParseError::unexpected_token(
                    state.next.clone(),
                    expected!(Keyword(Fun), Keyword(Type)),
                    "trait item",
                )),
            }?);
        }

        Ok(items)
    }
}
