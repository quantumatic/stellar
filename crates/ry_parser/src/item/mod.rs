mod associated_functions;
mod r#enum;
mod function;
mod r#impl;
mod imports;
mod struct_decl;
mod trait_decl;
mod type_alias;

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
    token::RawToken,
    Items, Token, Visibility,
};

pub(crate) struct ItemsParser {
    pub(crate) first_docstring: Docstring,
}

impl Parser for ItemsParser {
    type Output = Items;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut items = vec![];
        let mut docstring = self.first_docstring;

        while *state.next.unwrap() != RawToken::EndOfFile {
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

        if *state.next.unwrap() == Token![pub] {
            visibility = Visibility::public(state.next.span());
            state.next_token();
        }

        Ok(match state.next.unwrap() {
            Token![enum] => EnumDeclarationParser { visibility }.parse_with(state)?,
            Token![import] => ImportParser { visibility }.parse_with(state)?,
            Token![struct] => StructDeclarationParser { visibility }.parse_with(state)?,
            Token![trait] => TraitDeclarationParser { visibility }.parse_with(state)?,
            Token![fun] => FunctionItemParser { visibility }.parse_with(state)?,
            Token![impl] => ImplItemParser { visibility }.parse_with(state)?,
            Token![type] => TypeAliasItemParser { visibility }.parse_with(state)?,
            _ => {
                let error = Err(ParseError::unexpected_token(
                    state.next.clone(),
                    expected!(
                        Token![import],
                        Token![fun],
                        Token![trait],
                        Token![enum],
                        Token![struct],
                        Token![impl],
                        Token![type]
                    ),
                    "item",
                ));
                state.next_token();
                return error;
            }
        })
    }
}
