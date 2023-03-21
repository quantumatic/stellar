use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{location::Span, token::RawToken::*, *};

impl<'c> Parser<'c> {
    pub(crate) fn parse_trait_declaration(&mut self, public: Option<Span>) -> ParserResult<Item> {
        self.advance(false)?; // `trait`

        check_token!(self, Identifier => "trait name in trait declaration")?;

        let name = self.current_ident_with_span();

        self.advance(false)?; // name

        let generic_annotations = self.parse_generic_annotations()?;

        check_token!(self, OpenBrace => "trait declaration")?;

        self.advance(true)?; // `{`

        let methods = self.parse_trait_methods()?;

        check_token!(self, CloseBrace => "trait declaration")?;

        self.advance(true)?; // `}`

        Ok(Item::TraitDecl(TraitDecl {
            public,
            generic_annotations,
            name,
            methods,
        }))
    }

    pub(crate) fn parse_trait_methods(&mut self) -> ParserResult<Vec<(Docstring, TraitMethod)>> {
        let mut definitions = vec![];

        while !self.current.value.is(CloseBrace) {
            self.consume_local_docstring()?;

            let trait_def = self.parse_trait_method()?;
            definitions.push((self.consume_local_docstring()?, trait_def));
        }

        Ok(definitions)
    }

    fn parse_trait_method(&mut self) -> ParserResult<TraitMethod> {
        let mut public = None;

        if self.current.value.is(Pub) {
            public = Some(self.current.span);
            self.advance(false)?; // `pub`
        }

        check_token!(self, Fun => "trait method")?;

        self.advance(false)?; // `fun`

        check_token!(
            self,
            Identifier => "trait method name"
        )?;

        let name = self.current_ident_with_span();

        self.advance(false)?; // name

        let generic_annotations = self.parse_generic_annotations()?;

        check_token!(self, OpenParent => "trait method")?;

        self.advance(false)?; // `(`

        let arguments = parse_list!(self, "trait method arguments", CloseParent, false, || self
            .parse_function_argument());

        let mut return_type = None;

        if !self.current.value.is(Semicolon) && !self.current.value.is(OpenBrace) {
            return_type = Some(self.parse_type()?);
        }

        let mut body = None;

        match self.current.value {
            Semicolon => self.advance(true)?,
            OpenBrace => {
                body = Some(self.parse_statements_block(true)?);
            }
            _ => {
                return Err(ParserError::UnexpectedToken(
                    self.current.clone(),
                    "`;` (for method definition) or `{` (for method declaration)".to_owned(),
                    Some("trait method".to_owned()),
                ));
            }
        }

        Ok(TraitMethod {
            public,
            name,
            generic_annotations,
            params: arguments,
            return_type,
            body,
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
