use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{
    location::{Span, WithSpan},
    token::RawToken::*,
    *,
};
use string_interner::symbol::SymbolU32;

impl<'c> Parser<'c> {
    pub(crate) fn parse_name(&mut self) -> ParserResult<WithSpan<Vec<SymbolU32>>> {
        let start = self.current.span.start;

        let mut name = vec![];

        name.push(self.current_ident());

        let mut end = self.current.span.end;

        self.advance(false)?; // id

        while self.current.value.is(DoubleColon) {
            self.advance(false)?; // `::`

            check_token0!(self, "identifier", Identifier(_), "name")?;

            name.push(self.current_ident());

            end = self.current.span.end;

            self.advance(false)?; // id
        }

        Ok(name.with_span(start..end))
    }

    pub(crate) fn parse_type(&mut self) -> ParserResult<Type> {
        let start = self.current.span.start;

        let mut lhs = match &self.current.value {
            Identifier(_) => self.parse_primary_type(),
            And => self.parse_reference_type(),
            OpenBracket => self.parse_array_type(),
            _ => Err(ParserError::UnexpectedToken(
                self.current.clone(),
                "type".into(),
                None,
            )),
        }?;

        while self.current.value.is(QuestionMark) {
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
        if self.current.value.is(LessThan) {
            self.advance(false)?; // `<`

            Ok(Some(parse_list!(
                self,
                "generics",
                GreaterThan,
                false,
                || self.parse_type()
            )))
        } else {
            Ok(None)
        }
    }

    fn parse_array_type(&mut self) -> ParserResult<Type> {
        let start = self.current.span.start;

        self.advance(false)?; // `[`

        let inner_type = self.parse_type()?;

        check_token!(self, CloseBracket, "array type")?;

        let end = self.current.span.end;

        self.advance(false)?; // `]`

        Ok(WithSpan::new(
            Box::new(RawType::Array(inner_type)),
            Span::new(start, end),
        ))
    }

    fn parse_reference_type(&mut self) -> ParserResult<Type> {
        let start = self.current.span.start;

        self.advance(false)?; // `&`

        let mut mutable = false;

        if self.current.value.is(Mut) {
            mutable = true;

            self.advance(false)?; // `mut`
        }

        let inner_type = self.parse_type()?;

        let end = self.current.span.end;

        Ok(WithSpan::new(
            Box::new(RawType::Reference(mutable, inner_type)),
            Span::new(start, end),
        ))
    }

    pub(crate) fn parse_generic_annotations(&mut self) -> ParserResult<GenericAnnotations> {
        let mut generics = vec![];

        if !self.current.value.is(LessThan) {
            return Ok(generics);
        }

        self.advance(false)?; // '<'

        if self.current.value.is(GreaterThan) {
            self.advance(false)?; // '>'
            return Ok(generics);
        }

        loop {
            check_token0!(self, "identifier", Identifier(_), "generic annotation")?;

            let generic = self.parse_generic()?;

            let mut constraint = None;

            if !self.current.value.is(Comma) && !self.current.value.is(GreaterThan) {
                constraint = Some(self.parse_type()?);
            }

            generics.push((generic, constraint));

            if !self.current.value.is(Comma) {
                check_token!(self, GreaterThan, "generic annotations")?;

                self.advance(false)?; // >

                return Ok(generics);
            }

            self.advance(false)?;
        }
    }

    pub fn parse_generic(&mut self) -> ParserResult<WithSpan<SymbolU32>> {
        let name = self.current_ident_with_span();

        self.advance(false)?; // id

        Ok(name)
    }
}
