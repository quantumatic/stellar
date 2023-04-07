use crate::{error::ParseResult, path::PathParser, Parser, ParserState};
use ry_ast::{
    declaration::{ImportItem, Item},
    token::{Punctuator::Semicolon, RawToken::Punctuator},
    Visibility,
};

#[derive(Default)]
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

#[cfg(test)]
mod tests {
    use super::ImportParser;
    use crate::{macros::parser_test, Parser, ParserState};
    use ry_interner::Interner;

    parser_test!(ImportParser, single_import, "import test;");
    parser_test!(ImportParser, imports, "import test; import test2.test;");
}
