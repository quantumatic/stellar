use crate::{error::*, Parser};
use ry_ast::{
    declaration::{ImportItem, Item},
    token::{Punctuator::*, RawToken::*},
};

impl Parser<'_> {
    pub(crate) fn parse_import(&mut self) -> ParseResult<Item> {
        self.advance();

        let path = self.parse_path()?;

        self.consume_with_docstring(Punctuator(Semicolon), "import")?;

        Ok(ImportItem { path }.into())
    }
}

#[cfg(test)]
mod import_tests {
    use crate::{macros::parser_test, Parser};
    use ry_interner::Interner;

    parser_test!(single_import, "import test;");
    parser_test!(imports, "import test; import test2.test;");
}
