use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{
    declaration::{ImportItem, Item},
    token::RawToken::*,
};

impl<'c> Parser<'c> {
    pub(crate) fn parse_import(&mut self) -> ParserResult<Item> {
        self.advance()?;

        let path = self.parse_name()?;

        consume!(with_docstring self, Semicolon, "import");

        Ok(ImportItem::new(path).into())
    }
}

#[cfg(test)]
mod import_tests {
    use crate::{macros::parser_test, Parser};
    use ry_interner::Interner;

    parser_test!(single_import, "import test;");
    parser_test!(imports, "import test; import test2.test;");
}
