use crate::{error::ParseResult, macros::parse_list, Parser, ParserState};
use ry_ast::{
    declaration::{Documented, EnumDeclarationItem, WithDocstring},
    name::Name,
    token::{
        Punctuator::{CloseBrace, OpenBrace},
        RawToken::Punctuator,
    },
    Visibility,
};

#[derive(Default)]
pub(crate) struct EnumDeclarationParser {
    pub(crate) visibility: Visibility,
}

impl Parser for EnumDeclarationParser {
    type Output = EnumDeclarationItem;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let name = state.consume_identifier("enum name in enum declaration")?;

        state.consume(Punctuator(OpenBrace), "enum declaration")?;

        let variants = parse_list!(
            state,
            "enum declaration",
            Punctuator(CloseBrace),
            || -> ParseResult<Documented<Name>> {
                let doc = state.consume_docstring()?;
                Ok(state
                    .consume_identifier("enum variant name")?
                    .with_docstring(doc))
            }
        );

        state.advance();

        Ok(EnumDeclarationItem {
            visibility: self.visibility,
            name,
            variants,
        })
    }
}

#[cfg(test)]
mod enum_tests {
    use crate::{macros::parser_test, Parser, ParserState};
    use ry_interner::Interner;

    use super::EnumDeclarationParser;

    parser_test!(EnumDeclarationParser, no_variants, "enum test {}");
    parser_test!(EnumDeclarationParser, single_variant, "enum test { a }");
    parser_test!(EnumDeclarationParser, variants, "enum test { a, b, c, }");
}
