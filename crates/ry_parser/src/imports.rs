use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::token::RawToken::*;

impl<'c> Parser<'c> {
    pub(crate) fn parse_import(&mut self) -> ParserResult<ry_ast::Import> {
        self.advance()?;

        let path = self.parse_name()?;

        consume!(with_docstring self, Semicolon, "import");

        Ok(ry_ast::Import { path })
    }
}

#[cfg(test)]
mod import_tests {
    use crate::{macros::parser_test, Parser};
    use string_interner::StringInterner;

    parser_test!(single_import, "import test;");
    parser_test!(imports, "import test; import test2.test;");
}
