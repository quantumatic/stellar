use crate::{error::ParserError, macros::*, Parser, ParserResult};

use ry_ast::*;
use ry_ast::{
    location::{Span, WithSpan},
    token::*,
};

impl<'c> Parser<'c> {
    pub(crate) fn parse_name(&mut self) -> ParserResult<WithSpan<String>> {
        let start = self.current.span.start;

        let mut name = self.current.value.ident().unwrap();
        name.push_str("::");

        let mut end = self.current.span.end;

        self.advance(false)?; // id

        while self.current.value.is(RawToken::DoubleColon) {
            self.advance(false)?; // '::'

            check_token0!(self, "identifier", RawToken::Identifier(_), "name")?;

            name.push_str(&self.current.value.ident().unwrap());
            name.push_str("::");

            end = self.current.span.end;

            self.advance(false)?; // id
        }

        name.pop();
        name.pop();

        Ok(name.with_span(start..end))
    }

    pub fn get_name(&mut self) -> WithSpan<String> {
        self.current
            .value
            .ident()
            .unwrap()
            .with_span(self.current.span)
    }

    pub(crate) fn parse_type(&mut self) -> ParserResult<Type> {
        let start = self.current.span.start;

        let mut lhs = match &self.current.value {
            RawToken::Identifier(_) => self.parse_primary_type(),
            RawToken::Asterisk => self.parse_pointer_type(),
            RawToken::OpenBracket => self.parse_array_type(),
            _ => Err(ParserError::UnexpectedToken(
                self.current.clone(),
                "type".into(),
                None,
            )),
        }?;

        while self.current.value.is(RawToken::QuestionMark) {
            lhs = Box::new(RawType::Option(lhs)).with_span(start..self.current.span.end);
            self.advance(false)?;
        }

        Ok(lhs)
    }

    fn parse_primary_type(&mut self) -> ParserResult<Type> {
        let start = self.current.span.end;
        let name = self.parse_name()?;
        let generic_part = self.parse_type_generic_part()?;

        let mut end = self.current.span.end;

        if generic_part.is_some() {
            end = self.previous.as_ref().unwrap().span.end;
        }

        Ok(WithSpan::new(
            Box::new(RawType::Primary(
                name,
                if let Some(v) = generic_part {
                    v
                } else {
                    vec![]
                },
            )),
            Span::new(start, end),
        ))
    }

    pub(crate) fn parse_type_generic_part(&mut self) -> ParserResult<Option<Vec<Type>>> {
        if self.current.value.is(RawToken::LessThan) {
            self.advance(false)?; // '<'

            Ok(Some(parse_list!(
                self,
                "generics",
                RawToken::GreaterThan,
                false,
                || self.parse_type()
            )))
        } else {
            Ok(None)
        }
    }

    fn parse_array_type(&mut self) -> ParserResult<Type> {
        let start = self.current.span.start;

        self.advance(false)?; // '['

        let inner_type = self.parse_type()?;

        check_token!(self, RawToken::CloseBracket, "array type")?;

        let end = self.current.span.end;

        self.advance(false)?; // ']'

        Ok(WithSpan::new(
            Box::new(RawType::Array(inner_type)),
            Span::new(start, end),
        ))
    }

    fn parse_pointer_type(&mut self) -> ParserResult<Type> {
        let start = self.current.span.start;

        self.advance(false)?; // '*'

        let inner_type = self.parse_type()?;

        let end = self.current.span.end;

        Ok(WithSpan::new(
            Box::new(RawType::Pointer(inner_type)),
            Span::new(start, end),
        ))
    }

    pub(crate) fn parse_generic_annotations(&mut self) -> ParserResult<GenericAnnotations> {
        let mut generics = vec![];

        if !self.current.value.is(RawToken::LessThan) {
            return Ok(generics);
        }

        self.advance(false)?; // '<'

        if self.current.value.is(RawToken::GreaterThan) {
            self.advance(false)?; // '>'
            return Ok(generics);
        }

        loop {
            check_token0!(
                self,
                "identifier",
                RawToken::Identifier(_),
                "generic annotation"
            )?;

            let generic = self.parse_generic()?;

            let mut constraint = None;

            if !self.current.value.is(RawToken::Comma)
                && !self.current.value.is(RawToken::GreaterThan)
            {
                constraint = Some(self.parse_type()?);
            }

            generics.push((generic, constraint));

            if !self.current.value.is(RawToken::Comma) {
                check_token!(self, RawToken::GreaterThan, "generic annotations")?;

                self.advance(false)?; // >

                return Ok(generics);
            }

            self.advance(false)?;
        }
    }

    pub fn parse_generic(&mut self) -> ParserResult<WithSpan<String>> {
        let start = self.current.span.start;

        let name = self.current.value.ident().unwrap();
        let end = self.current.span.end;

        self.advance(false)?; // id

        Ok(name.with_span(start..end))
    }
}
