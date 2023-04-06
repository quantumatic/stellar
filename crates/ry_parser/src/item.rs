use crate::{
    error::{expected, ParseError, ParseResult},
    function_decl::FunctionParser,
    imports::ImportParser,
    r#enum::EnumDeclarationParser,
    struct_decl::StructDeclarationParser,
    trait_decl::TraitDeclarationParser,
    Parser, ParserState,
};
use ry_ast::{
    declaration::{Docstring, Item, WithDocstring},
    token::{
        Keyword::*,
        RawToken::{EndOfFile, Keyword},
    },
    Items, Visibility,
};

pub(crate) struct ItemsParser {
    pub(crate) first_docstring: Docstring,
}

impl Parser for ItemsParser {
    type Output = Items;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut items = vec![];
        let mut docstring = self.first_docstring;

        while state.next.inner != EndOfFile {
            items.push(ItemParser.parse_with(state)?.with_docstring(docstring));

            docstring = state.consume_docstring()?;
        }

        Ok(items)
    }
}

pub(crate) struct ItemParser;

impl Parser for ItemParser {
    type Output = Item;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut visibility = Visibility::private();

        if state.next.inner == Keyword(Pub) {
            visibility = Visibility::public(state.next.span);
            state.advance();
        }

        Ok(match state.next.inner {
            Keyword(Enum) => EnumDeclarationParser { visibility }
                .parse_with(state)?
                .into(),
            Keyword(Import) => ImportParser { visibility }.parse_with(state)?.into(),
            Keyword(Struct) => StructDeclarationParser { visibility }
                .parse_with(state)?
                .into(),
            Keyword(Trait) => TraitDeclarationParser { visibility }
                .parse_with(state)?
                .into(),
            Keyword(Fun) => FunctionParser { visibility }.parse_with(state)?.into(),
            _ => {
                let error = Err(ParseError::unexpected_token(
                    state.next.clone(),
                    expected!(
                        Keyword(Import),
                        Keyword(Fun),
                        Keyword(Trait),
                        Keyword(Enum),
                        Keyword(Struct),
                        Keyword(Pub)
                    ),
                    "item",
                ));
                state.advance();
                return error;
            }
        })
    }
}
