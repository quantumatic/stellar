use super::{function::FunctionParser, type_alias::TypeAliasParser};
use crate::{
    error::{expected, ParseError},
    r#type::{GenericsParser, WhereClauseParser},
    OptionalParser, ParseResult, Parser, ParserState,
};
use ry_ast::{
    declaration::{Documented, Item, TraitDeclarationItem, TraitItem, WithDocComment},
    token::{
        Keyword::{Fun, Pub, Type},
        Punctuator::{CloseBrace, OpenBrace},
        RawToken::{Keyword, Punctuator},
    },
    Visibility,
};

pub(crate) struct TraitItemsParser;

impl Parser for TraitItemsParser {
    type Output = Vec<Documented<TraitItem>>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut items = vec![];

        while state.next.inner != Punctuator(CloseBrace) {
            // TODO: Add type aliases here

            let doc = state.consume_docstring()?;

            let visibility = if state.next.inner == Keyword(Pub) {
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

#[derive(Default)]
pub(crate) struct TraitDeclarationParser {
    pub(crate) visibility: Visibility,
}

impl Parser for TraitDeclarationParser {
    type Output = Item;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let name = state.consume_identifier("trait name in trait declaration")?;

        let generics = GenericsParser.optionally_parse_with(state)?;

        let r#where = WhereClauseParser.optionally_parse_with(state)?;

        state.consume(Punctuator(OpenBrace), "trait declaration")?;

        let items = TraitItemsParser.parse_with(state)?;

        state.consume(Punctuator(CloseBrace), "trait declaration")?;

        Ok(TraitDeclarationItem {
            visibility: self.visibility,
            name,
            generics,
            r#where,
            items,
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::parser_test;

    parser_test!(TraitDeclarationParser, empty_struct, "trait test {}");
    parser_test!(TraitDeclarationParser, trait1, "trait test { fun f(); }");
    parser_test!(
        TraitDeclarationParser,
        trait2,
        "trait Into[T] { fun into(self: &Self): T; }"
    );
}
