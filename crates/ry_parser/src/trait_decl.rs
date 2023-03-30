use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{
    span::{Span, WithSpannable},
    token::RawToken::*,
    *,
};

impl<'c> Parser<'c> {
    pub(crate) fn parse_trait_declaration(
        &mut self,
        visibility: Option<Span>,
    ) -> ParserResult<Item> {
        self.advance()?;

        let name = consume_ident!(self, "trait name in trait declaration");

        let generic_annotations = self.parse_generic_annotations()?;

        let r#where = self.parse_where_clause()?;

        consume!(with_docstring self, OpenBrace, "trait declaration");

        let methods = self.parse_trait_methods()?;

        consume!(self, CloseBrace, "trait declaration");

        Ok(Item::TraitDecl(TraitDecl {
            public,
            generic_annotations,
            name,
            methods,
            r#where,
        }))
    }

    pub(crate) fn parse_trait_methods(&mut self) -> ParserResult<Vec<(Docstring, TraitMethod)>> {
        let mut definitions = vec![];

        while !self.next.unwrap().is(CloseBrace) {
            definitions.push((
                self.consume_non_module_docstring()?,
                self.parse_trait_method()?,
            ));
        }

        Ok(definitions)
    }

    fn parse_trait_method(&mut self) -> ParserResult<TraitMethod> {
        let mut public = None;

        if self.next.unwrap().is(Pub) {
            self.advance()?;
            public = Some(self.current.span());
        }

        consume!(self, Fun, "trait method");

        let name = consume_ident!(self, "trait method name");

        let generic_annotations = self.parse_generic_annotations()?;

        consume!(self, OpenParent, "trait method");

        let arguments = parse_list!(self, "trait method arguments", CloseParent, false, || self
            .parse_function_argument());

        self.advance()?;

        let mut return_type = None;

        if !self.next.unwrap().is_one_of(&[Semicolon, OpenBrace, Where]) {
            return_type = Some(self.parse_type()?);
        }

        let mut body = None;

        let r#where = self.parse_where_clause()?;

        if self.next.unwrap().is(OpenBrace) {
            body = Some(self.parse_statements_block(true)?);
        } else {
            self.advance_with_docstring()?; // `;`
        }

        Ok(TraitMethod {
            public,
            name,
            generic_annotations,
            params: arguments,
            return_type,
            body,
            r#where,
        })
    }
}

#[cfg(test)]
mod trait_tests {
    use crate::{macros::parser_test, Parser};
    use string_interner::StringInterner;

    parser_test!(empty_trait, "trait test {}");
    parser_test!(r#trait, "trait test { fun f(); }");
    parser_test!(
        r#trait_with_generics,
        "trait Into[T] { fun into(self &Self) T; }"
    );
}
