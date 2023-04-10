use crate::{
    error::ParseResult,
    r#type::{GenericsParser, TypeParser},
    OptionalParser, Parser, ParserState,
};
use ry_ast::{
    declaration::{Item, TypeAlias},
    token::{
        Punctuator::{Assign, Semicolon},
        RawToken::Punctuator,
    },
    Visibility,
};

#[derive(Default)]
pub(crate) struct TypeAliasParser {
    pub(crate) visibility: Visibility,
}

impl Parser for TypeAliasParser {
    type Output = TypeAlias;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let name = state.consume_identifier("type alias")?;
        let generics = GenericsParser.optionally_parse_with(state)?;

        let r#for = if state.next.inner == Punctuator(Assign) {
            state.next_token();

            Some(TypeParser.parse_with(state)?)
        } else {
            None
        };

        state.consume(Punctuator(Semicolon), "type alias")?;

        Ok(TypeAlias {
            visibility: self.visibility,
            name,
            generics,
            r#for,
        })
    }
}

#[derive(Default)]
pub(crate) struct TypeAliasItemParser {
    pub(crate) visibility: Visibility,
}

impl Parser for TypeAliasItemParser {
    type Output = Item;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        Ok(TypeAliasParser {
            visibility: self.visibility,
        }
        .parse_with(state)?
        .into())
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::parser_test;

    parser_test!(TypeAliasItemParser, empty_type_alias, "type A;");
    parser_test!(TypeAliasItemParser, type_alias1, "type B = Option[i32];");
    parser_test!(TypeAliasItemParser, type_alias2, "type B[T] = Option[T];");
}
