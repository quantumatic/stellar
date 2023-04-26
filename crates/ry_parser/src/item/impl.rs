use crate::{
    error::ParseResult,
    r#type::{GenericsParser, TypeParser, WhereClauseParser},
    OptionalParser, Parser, ParserState,
};
use ry_ast::{
    declaration::{ImplItem, Item},
    Token, Visibility,
};

use super::associated_functions::AssociatedFunctionsParser;

#[derive(Default)]
pub(crate) struct ImplItemParser {
    pub(crate) visibility: Visibility,
}

impl Parser for ImplItemParser {
    type Output = Item;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let generics = GenericsParser.optionally_parse_with(state)?;

        let mut r#type = TypeParser.parse_with(state)?;
        let mut r#trait = None;

        if *state.next.unwrap() == Token![for] {
            state.next_token();

            r#trait = Some(r#type);
            r#type = TypeParser.parse_with(state)?;
        }

        let r#where = WhereClauseParser.optionally_parse_with(state)?;

        state.consume(Token!['{'], "type implementation")?;

        let implementations = AssociatedFunctionsParser.parse_with(state)?;

        state.consume(Token!['}'], "type implementation")?;

        Ok(ImplItem {
            visibility: self.visibility,
            generics,
            r#type,
            r#trait,
            r#where,
            implementations,
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::parser_test;

    parser_test!(ImplItemParser, impl1, "impl[T] NotOption for T {}");
    parser_test!(
        ImplItemParser,
        impl2,
        "impl[T] Into[Option[M]] for Tuple[T, M] where M: Into[T] {}"
    );
}
