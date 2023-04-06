use crate::{error::*, r#type::PathParser, Parser, ParserState};
use ry_ast::{
    declaration::{ImportItem, Item},
    token::{Punctuator::*, RawToken::Punctuator},
    Visibility,
};

pub(crate) struct ImportParser {
    pub(crate) visibility: Visibility,
}

impl Parser for ImportParser {
    type Output = Item;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let path = PathParser.parse_with(state)?;
        state.consume(Punctuator(Semicolon), "import")?;

        Ok(ImportItem {
            visibility: self.visibility,
            path,
        }
        .into())
    }
}

// #[cfg(test)]
// mod import_tests {
//     use crate::{macros::parser_test, Parser};
//     use ry_interner::Interner;

//     parser_test!(single_import, "import test;");
//     parser_test!(imports, "import test; import test2.test;");
// }
