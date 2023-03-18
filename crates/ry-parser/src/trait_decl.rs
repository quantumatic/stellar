use crate::{error::ParserError, macros::*, Parser, ParserResult};

use ry_ast::*;
use ry_ast::{location::Span, token::RawToken};

impl<'c> Parser<'c> {
    pub(crate) fn parse_trait_declaration(
        &mut self,
        public: Option<Span>,
    ) -> ParserResult<TopLevelStatement> {
        self.advance(false)?; // 'trait'

        check_token0!(
            self,
            "identifier for trait name",
            RawToken::Identifier(_),
            "trait declaration"
        )?;

        let name = self.get_name();

        self.advance(false)?; // 'name'

        let generic_annotations = self.parse_generic_annotations()?;

        check_token!(self, RawToken::OpenBrace, "trait declaration")?;

        self.advance(true)?; // '{'

        let methods = self.parse_trait_methods()?;

        check_token!(self, RawToken::CloseBrace, "trait declaration")?;

        self.advance(true)?; // '}'

        Ok(TopLevelStatement::TraitDecl(TraitDecl {
            public,
            generic_annotations,
            name,
            methods,
        }))
    }

    pub(crate) fn parse_trait_methods(&mut self) -> ParserResult<Vec<(String, TraitMethod)>> {
        let mut definitions = vec![];

        while !self.current.value.is(RawToken::CloseBrace) {
            self.consume_local_docstring()?;

            let trait_def = self.parse_trait_method()?;
            definitions.push((self.consume_local_docstring()?, trait_def));
        }

        Ok(definitions)
    }

    fn parse_trait_method(&mut self) -> ParserResult<TraitMethod> {
        let mut public = None;

        if self.current.value.is(RawToken::Pub) {
            public = Some(self.current.span);
            self.advance(false)?; // pub
        }

        check_token!(self, RawToken::Fun, "trait method")?;

        self.advance(false)?; // 'fun'

        check_token0!(
            self,
            "identifier for method name",
            RawToken::Identifier(_),
            "trait method"
        )?;

        let name = self.get_name();

        self.advance(false)?; // name

        let generic_annotations = self.parse_generic_annotations()?;

        check_token!(self, RawToken::OpenParent, "trait method")?;

        self.advance(false)?; // '('

        let arguments = parse_list!(
            self,
            "trait method arguments",
            RawToken::CloseParent,
            false,
            || self.parse_function_argument()
        );

        let mut return_type = None;

        if !self.current.value.is(RawToken::Semicolon)
            && !self.current.value.is(RawToken::OpenBrace)
        {
            return_type = Some(self.parse_type()?);
        }

        let mut body = None;

        match self.current.value {
            RawToken::Semicolon => self.advance(true)?,
            RawToken::OpenBrace => {
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
