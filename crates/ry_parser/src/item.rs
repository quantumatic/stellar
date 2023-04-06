use crate::error::{expected, ParseError};
use crate::imports::ImportParser;
use crate::{error::ParseResult, r#enum::EnumDeclarationParser, Parser, ParserState};
use ry_ast::{
    declaration::{Docstring, Item, WithDocstring},
    token::{Keyword::*, RawToken::Keyword},
    Items, Visibility,
};

pub(crate) struct ItemsParser {
    pub first_docstring: Docstring,
}

impl Parser for ItemsParser {
    type Output = Items;

    fn parse_with(self, parser: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut items = vec![];
        let mut docstring = self.first_docstring;

        items.push(
            ItemParser { docstring }
                .parse(parser)?
                .with_docstring(docstring),
        );

        docstring = parser.consume_docstring();
    }
}

pub(crate) struct ItemParser {
    pub docstring: Docstring,
}

impl Parser for ItemParser {
    type Output = Item;

    fn parse_with(parser: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut visibility = Visibility::private();

        if parser.next.inner == Keyword(Pub) {
            visibility = Visibility::public(parser.next.span);
            parser.advance();
        }

        Ok(match parser.next.inner {
            Keyword(Enum) => EnumDeclarationParser { visibility }
                .parse_with(parser)
                .into(),
            Keyword(Import) => ImportParser { visibility }.parse_with(parser).into(),
            _ => {
                let error = Err(ParseError::unexpected_token(
                    parser.next.clone(),
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
                parser.advance();
                return error;
            }
        })
    }
}
