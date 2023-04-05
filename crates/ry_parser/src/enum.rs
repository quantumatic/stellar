use crate::{error::*, macros::*, Parser};
use ry_ast::{
    declaration::{Documented, EnumDeclarationItem, Item, WithDocstring},
    name::Name,
    token::{Punctuator::*, RawToken::*},
    Visibility,
};

impl Parser<'_> {
    pub(crate) fn parse_enum_declaration(&mut self, visibility: Visibility) -> ParseResult<Item> {
        self.advance();

        let name = self.consume_identifier("enum name in enum declaration")?;

        self.consume_with_docstring(Punctuator(OpenBrace), "enum declaration")?;

        let variants = parse_list!(
            self,
            "enum declaration",
            Punctuator(CloseBrace),
            true, // top level
            || -> ParseResult<Documented<Name>> {
                let doc = self.consume_non_module_docstring()?;
                Ok(self
                    .consume_identifier("enum variant name")?
                    .with_docstring(doc))
            }
        );

        self.advance_with_docstring();

        Ok(EnumDeclarationItem {
            visibility,
            name,
            variants,
        }
        .into())
    }
}

#[cfg(test)]
mod enum_tests {
    use crate::{macros::parser_test, Parser};
    use ry_interner::Interner;

    parser_test!(no_variants, "enum test {}");
    parser_test!(single_variant, "enum test { a }");
    parser_test!(variants, "enum test { a, b, c, }");
}
