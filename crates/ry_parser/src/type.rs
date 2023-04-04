use crate::{error::ParseError, macros::*, ParseResult, Parser};
use ry_ast::{
    r#type::{
        ArrayType, Generic, Generics, PrimaryType, RawType, ReferenceType, Type, WhereClause,
        WhereClauseUnit,
    },
    span::{Spanned, WithSpan},
    token::{Keyword::*, Punctuator::*, RawToken::*},
    Mutability,
};
use ry_interner::Symbol;

impl Parser<'_> {
    pub(crate) fn parse_path(&mut self) -> ParseResult<Spanned<Vec<Symbol>>> {
        let mut path = vec![];

        let first_ident = consume_ident!(self, "path");
        path.push(*first_ident.unwrap());

        let (start, mut end) = (first_ident.span().start(), first_ident.span().end());

        while let Punctuator(Dot) = self.next.unwrap() {
            self.advance()?; // `.`

            path.push(*consume_ident!(self, "path").unwrap());

            end = self.current.span().end();
        }

        Ok(path.with_span(start..end))
    }

    pub(crate) fn parse_type(&mut self) -> ParseResult<Type> {
        let start = self.next.span().start();

        let r#type = match self.next.unwrap() {
            Identifier(_) => {
                let path = self.parse_path()?;
                let generic_part = self.parse_type_generic_part()?;

                RawType::from(PrimaryType {
                    path,
                    type_annotations: if let Some(v) = generic_part {
                        v
                    } else {
                        vec![]
                    },
                })
                .with_span(start..self.current.span().end())
            }
            Punctuator(And) => {
                self.advance()?;
                let start = self.current.span().start();

                let mut mutability = Mutability::immutable();

                if let Keyword(Mut) = self.next.unwrap() {
                    mutability = Mutability::mutable(self.next.span());

                    self.advance()?; // `mut`
                }

                let inner = self.parse_type()?;

                RawType::from(ReferenceType {
                    mutability,
                    inner: Box::new(inner),
                })
                .with_span(start..self.current.span().end())
            }
            Punctuator(OpenBracket) => {
                self.advance()?;
                let start = self.current.span().start();

                let inner = self.parse_type()?;

                consume!(self, Punctuator(CloseBracket), "array type");

                RawType::from(ArrayType {
                    inner: Box::new(inner),
                })
                .with_span(start..self.current.span().end())
            }
            _ => {
                return Err(ParseError::unexpected_token(
                    self.next.clone(),
                    "`[` (array type), `&` (reference type) or \n\tidentifier",
                    "type",
                ));
            }
        };

        Ok(r#type)
    }

    pub(crate) fn parse_type_generic_part(&mut self) -> ParseResult<Option<Vec<Type>>> {
        Ok(if let Punctuator(OpenBracket) = self.next.unwrap() {
            self.advance()?; // `[`

            let result = Some(parse_list!(
                self,
                "generics",
                Punctuator(CloseBracket),
                false,
                || self.parse_type()
            ));

            self.advance()?; // `]`

            result
        } else {
            None
        })
    }

    pub(crate) fn optionally_parse_generics(&mut self) -> ParseResult<Generics> {
        match self.next.unwrap() {
            Punctuator(OpenBracket) => {}
            _ => return Ok(vec![]),
        }

        self.advance()?; // `[`

        let result = parse_list!(self, "generics", Punctuator(CloseBracket), false, || {
            let name = consume_ident!(self, "generic name");

            let mut constraint = None;

            if let Punctuator(Colon) = self.next.unwrap() {
                self.advance()?;
                constraint = Some(self.parse_type()?);
            }

            Ok(Generic { name, constraint })
        });

        self.advance()?;

        Ok(result)
    }

    pub(crate) fn optionally_parse_where_clause(&mut self) -> ParseResult<WhereClause> {
        Ok(if let Keyword(Where) = self.next.unwrap() {
            self.advance()?; // `where`

            let result = parse_list!(
                self,
                "where clause",
                Punctuator(OpenBrace | Semicolon),
                false, // top level
                || {
                    let r#type = self.parse_type()?;

                    consume!(self, Punctuator(Colon), "where clause");

                    let constraint = self.parse_type()?;

                    Ok(WhereClauseUnit { r#type, constraint })
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
    use ry_interner::Interner;

    parser_test!(primary_type1, "pub fun test(): i32 {}");
    parser_test!(
        primary_type2,
        "pub fun div[T](a: T, b: T): Result[T, DivisionError] {}"
    );
    parser_test!(array_type, "pub fun test(a: [i32]) {}");
    parser_test!(reference_type, "pub fun test(a: &mut i32): i32 {}");
    parser_test!(negative_trait_type, "pub fun test(a: Into[string]) {}");
}
