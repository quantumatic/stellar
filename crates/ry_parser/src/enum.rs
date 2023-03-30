use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{span::*, token::RawToken::*, *};

impl<'c> Parser<'c> {
    pub(crate) fn parse_enum_declaration(&mut self, visiblity: Option<Span>) -> ParserResult<Item> {
        self.advance()?;

        let name = consume_ident!(self, "enum name in enum declaration");

        consume!(with_docstring self, OpenBrace, "enum declaration");

        let variants = parse_list!(
            self,
            "enum declaration",
            CloseBrace,
            true, // top level
            || {
                let doc = self.consume_non_module_docstring()?;

                let variant = consume_ident!(self, "enum variant name");

                Ok((doc, variant))
            }
        );

        self.advance_with_docstring()?; // `}`

        Ok(Item::EnumDecl(EnumDecl {
            public,
            name,
            variants,
        }))
    }
}

#[cfg(test)]
mod enum_tests {
    use crate::{macros::parser_test, Parser};
    use string_interner::StringInterner;

    parser_test!(no_variants, "enum test {}");
    parser_test!(single_variant, "enum test { a }");
    parser_test!(variants, "enum test { a, b, c, }");
}
