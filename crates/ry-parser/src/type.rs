use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{location::WithSpan, token::RawToken::*, *};
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

            check_token!(self, Identifier => "namespace member/namespace")?;

            name.push(self.current_ident());

            end = self.current.span.end;

            self.advance(false)?; // id
        }

        Ok(name.with_span(start..end))
    }

    pub(crate) fn parse_type(&mut self) -> ParserResult<Type> {
        let start = self.current.span.start;

        let mut lhs = match &self.current.value {
            Identifier(_) => {
                let start = self.current.span.end;
                let name = self.parse_name()?;
                let generic_part = self.parse_type_generic_part()?;

                let mut end = self.current.span.end;

                if generic_part.is_some() {
                    end = self.previous.as_ref().unwrap().span.end;
                }

                Box::new(RawType::Primary(
                    name,
                    if let Some(v) = generic_part {
                        v
                    } else {
                        vec![]
                    },
                ))
                .with_span(start..end)
            }
            And => {
                let start = self.current.span.start;

                self.advance(false)?; // `&`

                let mut mutable = false;

                if self.current.value.is(Mut) {
                    mutable = true;

                    self.advance(false)?; // `mut`
                }

                let inner_type = self.parse_type()?;

                let end = self.current.span.end;

                Box::new(RawType::Reference(mutable, inner_type)).with_span(start..end)
            }
            OpenBracket => {
                let start = self.current.span.start;

                self.advance(false)?; // `[`

                let inner_type = self.parse_type()?;

                check_token!(self, CloseBracket => "array type")?;

                let end = self.current.span.end;

                self.advance(false)?; // `]`

                Box::new(RawType::Array(inner_type)).with_span(start..end)
            }
            Bang => {
                let start = self.current.span.start;

                self.advance(false)?; // '!'

                let inner_type = self.parse_type()?;

                let end = self.current.span.end;

                Box::new(RawType::NegativeTrait(inner_type)).with_span(start..end)
            }
            _ => {
                return Err(ParserError::UnexpectedToken(
                    self.current.clone(),
                    "type".into(),
                    None,
                ))
            }
        };

        while self.current.value.is(QuestionMark) {
            lhs = Box::new(RawType::Option(lhs)).with_span(start..self.current.span.end);
            self.advance(false)?;
        }

        Ok(lhs)
    }

    pub(crate) fn parse_type_generic_part(&mut self) -> ParserResult<Option<Vec<Type>>> {
        if self.current.value.is(OpenBracket) {
            self.advance(false)?; // `[`

            Ok(Some(parse_list!(
                self,
                "generics",
                CloseBracket,
                false,
                || self.parse_type()
            )))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn parse_generic_annotations(&mut self) -> ParserResult<GenericAnnotations> {
        if !self.current.value.is(OpenBracket) {
            return Ok(vec![]);
        }

        self.advance(false)?; // `[`

        Ok(parse_list!(
            self,
            "generic annotations",
            CloseBracket,
            false, // top level
            || {
                check_token!(self, Identifier => "generic name in generic annotation")?;

                let generic = self.current_ident_with_span();

                self.advance(false)?; // ident

                let mut constraint = None;

                if !self.current.value.is_one_of(&[CloseBracket, Comma]) {
                    constraint = Some(self.parse_type()?);
                }

                Ok((generic, constraint))
            }
        ))
    }

    pub fn parse_generic(&mut self) -> ParserResult<WithSpan<SymbolU32>> {
        let name = self.current_ident_with_span();

        self.advance(false)?; // id

        Ok(name)
    }
}

#[cfg(test)]
mod type_tests {
    use crate::{macros::parser_test, Parser};
    use string_interner::StringInterner;

    parser_test!(primary_type1, "pub fun test() i32 {}");
    parser_test!(
        primary_type2,
        "pub fun div[T](a T, b T) Result[T, DivisionError] {}"
    );
    parser_test!(array_type, "pub fun test(a [i32]) {}");
    parser_test!(reference_type, "pub fun test(a &mut i32) i32 {}");
    parser_test!(negative_trait_type, "pub fun test(a !Into[string]) {}");
}
