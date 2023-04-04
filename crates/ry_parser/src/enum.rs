use crate::{error::*, macros::*, Parser};
use ry_ast::{
    declaration::{EnumDeclarationItem, Item, WithDocstring},
    span::*,
    token::{Punctuator::*, RawToken::*},
    Visibility,
};

impl Parser<'_> {
    pub(crate) fn parse_enum_declaration(&mut self, visibility: Visibility) -> ParseResult<Item> {
        self.advance()?;

        let name = consume_ident!(self, "enum name in enum declaration");

        consume!(with_docstring self, Punctuator(OpenBrace), "enum declaration");

        let variants = parse_list!(
            self,
            "enum declaration",
            Punctuator(CloseBrace),
            true, // top level
            || {
                let doc = self.consume_non_module_docstring()?;

                let variant = consume_ident!(self, "enum variant name");

                Ok(variant.with_docstring(doc))
            }
        );

        self.advance_with_docstring()?; // `}`

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
