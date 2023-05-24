use super::associated_functions::AssociatedFunctionsParser;
use crate::{
    r#type::{GenericsParser, WhereClauseParser},
    OptionalParser, ParseResult, Parser, ParserState,
};
use ry_ast::{
    declaration::{Item, TraitDeclarationItem},
    Token, Visibility,
};

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

        state.consume(Token!['{'], "trait declaration")?;

        let items = AssociatedFunctionsParser.parse_with(state)?;

        state.consume(Token!['}'], "trait declaration")?;

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
        "trait Into[T] { fun into(self: Self): T; }"
    );
}
