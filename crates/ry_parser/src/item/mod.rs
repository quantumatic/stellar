mod r#enum;
mod function;
mod r#impl;
mod imports;
mod struct_decl;
mod trait_decl;
mod type_alias;
mod associated_functions;

use self::{
    function::FunctionItemParser, imports::ImportParser, r#enum::EnumDeclarationParser,
    r#impl::ImplItemParser, struct_decl::StructDeclarationParser,
    trait_decl::TraitDeclarationParser, type_alias::TypeAliasItemParser,
};
use crate::{
    error::{expected, ParseError, ParseResult},
    Parser, ParserState,
};
use ry_ast::{
    declaration::{Docstring, Item, WithDocComment},
    token::{
        Keyword::{Enum, Fun, Impl, Import, Pub, Struct, Trait, Type},
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
            items.push(ItemParser.parse_with(state)?.with_doc_comment(docstring));

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
            state.next_token();
        }

        Ok(match state.next.inner {
            Keyword(Enum) => EnumDeclarationParser { visibility }.parse_with(state)?,
            Keyword(Import) => ImportParser { visibility }.parse_with(state)?,
            Keyword(Struct) => StructDeclarationParser { visibility }.parse_with(state)?,
            Keyword(Trait) => TraitDeclarationParser { visibility }.parse_with(state)?,
            Keyword(Fun) => FunctionItemParser { visibility }.parse_with(state)?,
            Keyword(Impl) => ImplItemParser { visibility }.parse_with(state)?,
            Keyword(Type) => TypeAliasItemParser { visibility }.parse_with(state)?,
            _ => {
                let error = Err(ParseError::unexpected_token(
                    state.next.clone(),
                    expected!(
                        Keyword(Import),
                        Keyword(Fun),
                        Keyword(Trait),
                        Keyword(Enum),
                        Keyword(Struct),
                        Keyword(Impl),
                        Keyword(Type)
                    ),
                    "item",
                ));
                state.next_token();
                return error;
            }
        })
    }
}
