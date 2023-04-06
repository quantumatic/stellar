use crate::{error::*, r#type::PathParser, Parser, ParserState};
use ry_ast::{
    declaration::{ImportItem, Item},
    token::{Punctuator::*, RawToken::Punctuator},
    Visibility,
};

pub(crate) struct ImportParser {
    visibility: Visibility,
}

impl Parser for ImportParser {
    type Output = Item;

    fn parse_with(self, parser: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        parser.advance();

        let path = PathParser.parse()?;
        parser.consume(Punctuator(Semicolon), "import")?;

        Ok(ImportItem { path }.into())
    }
}

// #[cfg(test)]
// mod import_tests {
//     use crate::{macros::parser_test, Parser};
//     use ry_interner::Interner;

//     parser_test!(single_import, "import test;");
//     parser_test!(imports, "import test; import test2.test;");
// }
