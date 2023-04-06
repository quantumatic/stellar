use crate::{error::*, macros::*, Parser, ParserState};
use ry_ast::{
    declaration::{Documented, EnumDeclarationItem},
    name::Name,
    token::{Punctuator::*, RawToken::*},
    Visibility,
};

pub(crate) struct EnumDeclarationParser {
    pub visibility: Visibility,
}

impl Parser for EnumDeclarationParser {
    type Output = EnumDeclarationItem;

    fn parse_with(self, parser: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        parser.advance();

        let name = parser.consume_identifier("enum name in enum declaration")?;

        parser.consume(Punctuator(OpenBrace), "enum declaration")?;

        let variants = parse_list!(
            self,
            "enum declaration",
            Punctuator(CloseBrace),
            true, // top level
            || -> ParseResult<Documented<Name>> {
                let doc = parser.consume_non_module_docstring()?;
                Ok(parser
                    .consume_identifier("enum variant name")?
                    .with_docstring(doc))
            }
        );

        parser.advance_with_docstring();

        Ok(EnumDeclarationItem {
            visibility: self.visibility,
            name,
            variants,
        })
    }
}

#[cfg(test)]
mod enum_tests {
    use crate::{macros::parser_test, ParserState};
    use ry_interner::Interner;

    use super::EnumDeclarationParser;

    parser_test!(EnumDeclarationParser, no_variants, "enum test {}");
    parser_test!(EnumDeclarationParser, single_variant, "enum test { a }");
    parser_test!(EnumDeclarationParser, variants, "enum test { a, b, c, }");
}
