use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::token::RawToken::*;

impl<'c> Parser<'c> {
    /// TODO: fix the problem with comments and imports messed up
    pub(crate) fn parse_imports(&mut self) -> ParserResult<Vec<ry_ast::Import>> {
        let mut imports = vec![];

        while self.current.value.is(Import) {
            imports.push(self.parse_import()?);
            self.advance(false)?; // `;`
        }

        Ok(imports)
    }

    pub(crate) fn parse_import(&mut self) -> ParserResult<ry_ast::Import> {
        self.advance(false)?; // `import`

        check_token!(self, Identifier => "namespace (for example: `std::io`)")?;

        let path = self.parse_name()?;

        check_token!(self, Semicolon => "import")?;

        Ok(ry_ast::Import { path })
    }
}

#[cfg(test)]
mod import_tests {
    use crate::{macros::parser_test, Parser};
    use string_interner::StringInterner;

    parser_test!(single_import, "import test;");
    parser_test!(imports, "import test; import test2::test;");
}
