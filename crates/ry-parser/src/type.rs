use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{
    location::{Span, WithSpan},
    token::RawToken::*,
    *,
};
use string_interner::DefaultSymbol;

impl<'c> Parser<'c> {
    pub(crate) fn parse_name(&mut self) -> ParserResult<WithSpan<Vec<DefaultSymbol>>> {
        let mut name = vec![];

        let first_ident = consume_ident!(self, "namespace member/namespace");
        name.push(first_ident.value);

        let Span { start, mut end } = first_ident.span;

        while self.next.value.is(DoubleColon) {
            self.advance()?; // `::`

            name.push(consume_ident!(self, "namespace member/namespace").value);

            end = self.current.span.end;
        }

        Ok(name.with_span(start..end))
    }

    pub(crate) fn parse_type(&mut self) -> ParserResult<Type> {
        let start = self.next.span.start;

        self.check_scanning_error_for_next_token()?;

        let mut lhs = match &self.next.value {
            Identifier(_) => {
                let name = self.parse_name()?;
                let generic_part = self.parse_type_generic_part()?;

                let end = self.current.span.end;

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
                self.advance()?;
                let start = self.current.span.start;

                let mut mutable = false;

                if self.next.value.is(Mut) {
                    mutable = true;

                    self.advance()?; // `mut`
                }

                let inner_type = self.parse_type()?;

                let end = self.current.span.end;

                Box::new(RawType::Reference(mutable, inner_type)).with_span(start..end)
            }
            OpenBracket => {
                self.advance()?;
                let start = self.current.span.start;

                let inner_type = self.parse_type()?;

                consume!(self, CloseBracket, "array type");

                let end = self.current.span.end;

                Box::new(RawType::Array(inner_type)).with_span(start..end)
            }
            Bang => {
                self.advance()?;
                let start = self.current.span.start;

                let inner_type = self.parse_type()?;

                let end = self.current.span.end;

                Box::new(RawType::NegativeTrait(inner_type)).with_span(start..end)
            }
            _ => {
                return Err(ParserError::UnexpectedToken(
                    self.next.clone(),
                    "`!` (negative trait), `[` (array type), `&` (reference type) or \n\tidentifier"
                        .to_owned(),
                    "type".to_owned(),
                ));
            }
        };

        while self.next.value.is(QuestionMark) {
            lhs = Box::new(RawType::Option(lhs)).with_span(start..self.next.span.end);
            self.advance()?;
        }

        Ok(lhs)
    }

    pub(crate) fn parse_type_generic_part(&mut self) -> ParserResult<Option<Vec<Type>>> {
        Ok(if self.next.value.is(OpenBracket) {
            self.advance()?; // `[`

            let result =
                Some(parse_list!(self, "generics", CloseBracket, false, || self.parse_type()));

            self.advance()?; // `]`

            result
        } else {
            None
        })
    }

    pub(crate) fn parse_generic_annotations(&mut self) -> ParserResult<GenericAnnotations> {
        if !self.next.value.is(OpenBracket) {
            return Ok(vec![]);
        }

        self.advance()?; // `[`

        let result = parse_list!(
            self,
            "generic annotations",
            CloseBracket,
            false, // top level
            || {
                let generic = consume_ident!(self, "generic name in generic annotation");

                let mut constraint = None;

                if self.next.value.is(Of) {
                    self.advance()?; // `of`

                    constraint = Some(self.parse_type()?);
                }

                Ok((generic, constraint))
            }
        );

        self.advance()?;

        Ok(result)
    }

    pub(crate) fn parse_where_clause(&mut self) -> ParserResult<WhereClause> {
        Ok(if self.next.value.is(Where) {
            self.advance()?; // `where`

            let result = parse_list!(
                self,
                "where clause",
                OpenBrace | Semicolon,
                false, // top level
                || {
                    let left = self.parse_type()?;

                    consume!(self, Of, "where clause");

                    let right = self.parse_type()?;

                    Ok((left, right))
                }
            );

            result
        } else {
            vec![]
        })
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
