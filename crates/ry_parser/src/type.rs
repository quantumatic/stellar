use crate::{error::ParserError, macros::*, Parser, ParserResult};
use ry_ast::{
    r#type::{
        ArrayType, Generic, Generics, OptionType, PrimaryType, RawType, ReferenceType, Type,
        WhereClause, WhereClauseUnit,
    },
    span::{WithSpan, WithSpannable},
    token::RawToken::*,
};

use string_interner::DefaultSymbol;

impl<'c> Parser<'c> {
    pub(crate) fn parse_name(&mut self) -> ParserResult<WithSpan<Vec<DefaultSymbol>>> {
        let mut name = vec![];

        let first_ident = consume_ident!(self, "namespace member/namespace");
        name.push(*first_ident.unwrap());

        let (start, mut end) = (first_ident.span().start(), first_ident.span().end());

        while self.next.unwrap().is(Dot) {
            self.advance()?; // `.`

            name.push(*consume_ident!(self, "namespace member/namespace").unwrap());

            end = self.current.span().end();
        }

        Ok(name.with_span(start..end))
    }

    pub(crate) fn parse_type(&mut self) -> ParserResult<Type> {
        let start = self.next.span().start();

        self.check_scanning_error_for_next_token()?;

        let mut left = match self.next.unwrap() {
            Identifier(_) => {
                let name = self.parse_name()?;
                let generic_part = self.parse_type_generic_part()?;

                Box::<RawType>::new(
                    PrimaryType::new(
                        name,
                        if let Some(v) = generic_part {
                            v
                        } else {
                            vec![]
                        },
                    )
                    .into(),
                )
                .with_span(start..self.current.span().end())
            }
            And => {
                self.advance()?;
                let start = self.current.span().start();

                let mut mutability = None;

                if self.next.unwrap().is(Mut) {
                    mutability = Some(self.next.span());

                    self.advance()?; // `mut`
                }

                let inner_type = self.parse_type()?;

                Box::<RawType>::new(ReferenceType::new(mutability, inner_type).into())
                    .with_span(start..self.current.span().end())
            }
            OpenBracket => {
                self.advance()?;
                let start = self.current.span().start();

                let inner_type = self.parse_type()?;

                consume!(self, CloseBracket, "array type");

                Box::<RawType>::new(ArrayType::new(inner_type).into())
                    .with_span(start..self.current.span().end())
            }
            _ => {
                return Err(ParserError::UnexpectedToken(
                    self.next.clone(),
                    "`[` (array type), `&` (reference type) or \n\tidentifier".to_owned(),
                    "type".to_owned(),
                ));
            }
        };

        while self.next.unwrap().is(QuestionMark) {
            left = Box::<RawType>::new(OptionType::new(left).into())
                .with_span(start..self.next.span().end());
            self.advance()?;
        }

        Ok(left)
    }

    pub(crate) fn parse_type_generic_part(&mut self) -> ParserResult<Option<Vec<Type>>> {
        Ok(if self.next.unwrap().is(OpenBracket) {
            self.advance()?; // `[`

            let result =
                Some(parse_list!(self, "generics", CloseBracket, false, || self.parse_type()));

            self.advance()?; // `]`

            result
        } else {
            None
        })
    }

    pub(crate) fn parse_generics(&mut self) -> ParserResult<Generics> {
        if !self.next.unwrap().is(OpenBracket) {
            return Ok(vec![]);
        }

        self.advance()?; // `[`

        let result = parse_list!(
            self,
            "generics",
            CloseBracket,
            false, // top level
            || {
                let generic = consume_ident!(self, "generic name");

                let mut constraint = None;

                if self.next.unwrap().is(Colon) {
                    self.advance()?; // `:`

                    constraint = Some(self.parse_type()?);
                }

                Ok(Generic::new(generic, constraint))
            }
        );

        self.advance()?;

        Ok(result)
    }

    pub(crate) fn parse_where_clause(&mut self) -> ParserResult<WhereClause> {
        Ok(if self.next.unwrap().is(Where) {
            self.advance()?; // `where`

            let result = parse_list!(
                self,
                "where clause",
                OpenBrace | Semicolon,
                false, // top level
                || {
                    let left = self.parse_type()?;

                    consume!(self, Colon, "where clause");

                    let right = self.parse_type()?;

                    Ok(WhereClauseUnit::new(left, right))
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

    parser_test!(primary_type1, "pub fun test(): i32 {}");
    parser_test!(
        primary_type2,
        "pub fun div[T](a: T, b: T): Result[T, DivisionError] {}"
    );
    parser_test!(array_type, "pub fun test(a: [i32]) {}");
    parser_test!(reference_type, "pub fun test(a: &mut i32): i32 {}");
    parser_test!(negative_trait_type, "pub fun test(a: Into[string]) {}");
}
