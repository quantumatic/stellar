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

        check_token0!(
            self,
            "path (for example: `std::io`)",
            Identifier(_),
            "import"
        )?;

        let path = self.parse_name()?;

        check_token!(self, Semicolon, "import")?;

        Ok(ry_ast::Import { path })
    }
}

mod import_tests {}
