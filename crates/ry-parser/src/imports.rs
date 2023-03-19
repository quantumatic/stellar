use crate::{error::ParserError, macros::*, Parser, ParserResult};

use ry_ast::token::RawToken;
use ry_ast::*;

impl<'c> Parser<'c> {
    /// TODO: fix the problem with comments and imports messed up
    pub(crate) fn parse_imports(&mut self) -> ParserResult<Vec<Import>> {
        let mut imports = vec![];

        while self.current.value.is(RawToken::Import) {
            imports.push(self.parse_import()?);
            self.advance(false)?; // ';'
        }

        Ok(imports)
    }

    pub(crate) fn parse_import(&mut self) -> ParserResult<Import> {
        self.advance(false)?; // import

        check_token0!(
            self,
            "path (for example: `std::io`)",
            RawToken::Identifier(_),
            "import"
        )?;

        let path = self.parse_name()?;

        check_token!(self, RawToken::Semicolon, "import")?;

        Ok(Import { path })
    }
}

mod import_tests {}
