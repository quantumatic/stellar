use super::function_decl::FunctionParser;
use crate::{
    error::{expected, ParseError},
    r#type::{GenericsParser, WhereClauseParser},
    OptionalParser, ParseResult, Parser, ParserState,
};
use ry_ast::{
    declaration::{Documented, Function, Item, TraitDeclarationItem, WithDocstring},
    token::{
        Keyword::{Fun, Pub},
        Punctuator::{And, CloseBrace, OpenBrace, OpenBracket},
        RawToken::{Keyword, Punctuator},
    },
    Visibility,
};

pub(crate) struct TraitItemsParser;

impl Parser for TraitItemsParser {
    type Output = Vec<Documented<Function>>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut items = vec![];

        while state.next.inner != Punctuator(CloseBrace) {
            // TODO: Add type aliases here

            let doc = state.consume_docstring()?;

            let visibility = if state.next.inner == Keyword(Pub) {
                state.advance();
                Visibility::public(state.current.span)
            } else {
                Visibility::private()
            };

            items.push(match state.next.inner {
                Keyword(Fun) => Ok(FunctionParser { visibility }
                    .parse_with(state)?
                    .with_docstring(doc)),
                _ => Err(ParseError::unexpected_token(
                    state.next.clone(),
                    expected!("identifier", Punctuator(And), Punctuator(OpenBracket)),
                    "type",
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
        state.advance();

        let name = state.consume_identifier("trait name in trait declaration")?;

        let generics = GenericsParser.optionally_parse_with(state)?;

        let r#where = WhereClauseParser.optionally_parse_with(state)?;

        state.consume(Punctuator(OpenBrace), "trait declaration")?;

        let methods = TraitItemsParser.parse_with(state)?;

        state.consume(Punctuator(CloseBrace), "trait declaration")?;

        Ok(TraitDeclarationItem {
            visibility: self.visibility,
            name,
            generics,
            r#where,
            methods,
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
