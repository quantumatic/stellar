use crate::{error::ParseError, macros::*, ParseResult, Parser};
use ry_ast::{
    r#type::{
        ArrayType, Generic, Generics, PrimaryType, RawType, ReferenceType, Type, WhereClause,
        WhereClauseUnit,
    },
    span::{At, Spanned},
    token::{Keyword::*, Punctuator::*, RawToken::*},
    Mutability,
};
use ry_interner::Symbol;

impl Parser<'_> {
    pub(crate) fn parse_path(&mut self) -> ParseResult<Spanned<Vec<Symbol>>> {
        let mut path = vec![];

        let first_identifier = self.consume_identifier("path")?;
        path.push(first_identifier.inner);

        let (start, mut end) = (first_identifier.span.start(), first_identifier.span.end());

        while let Punctuator(Dot) = self.next.inner {
            self.advance();

            path.push(self.consume_identifier("path")?.inner);

            end = self.current.span.end();
        }

        Ok(path.at(start..end))
    }

    pub(crate) fn parse_type(&mut self) -> ParseResult<Type> {
        let start = self.next.span.start();

        let r#type = match self.next.inner {
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
                .at(start..self.current.span.end())
            }
            Punctuator(And) => {
                self.advance();
                let start = self.current.span.start();

                let mut mutability = Mutability::immutable();

                if let Keyword(Mut) = self.next.inner {
                    mutability = Mutability::mutable(self.next.span);

                    self.advance();
                }

                let inner = self.parse_type()?;

                RawType::from(ReferenceType {
                    mutability,
                    inner: Box::new(inner),
                })
                .at(start..self.current.span.end())
            }
            Punctuator(OpenBracket) => {
                self.advance();
                let start = self.current.span.start();

                let inner = self.parse_type()?;

                self.consume(Punctuator(CloseBracket), "array type")?;

                RawType::from(ArrayType {
                    inner: Box::new(inner),
                })
                .at(start..self.current.span.end())
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
        Ok(if self.next.inner == Punctuator(OpenBracket) {
            self.advance();

            let result = Some(parse_list!(
                self,
                "generics",
                Punctuator(CloseBracket),
                false,
                || self.parse_type()
            ));

            self.advance();

            result
        } else {
            None
        })
    }

    pub(crate) fn optionally_parse_generics(&mut self) -> ParseResult<Generics> {
        if self.next.inner != Punctuator(OpenBracket) {
            return Ok(vec![]);
        }

        self.advance();

        let result = parse_list!(
            self,
            "generics",
            Punctuator(CloseBracket),
            false,
            || -> ParseResult<Generic> {
                dbg!(&self.next);
                let name = self.consume_identifier("generic name")?;

                let mut constraint = None;

                if self.next.inner == Punctuator(Colon) {
                    self.advance();
                    constraint = Some(self.parse_type()?);
                }

                Ok(Generic { name, constraint })
            }
        );

        self.advance();

        Ok(result)
    }

    pub(crate) fn optionally_parse_where_clause(&mut self) -> ParseResult<WhereClause> {
        Ok(if let Keyword(Where) = self.next.inner {
            self.advance();

            let result = parse_list!(
                self,
                "where clause",
                Punctuator(OpenBrace | Semicolon),
                false, // top level
                || {
                    let r#type = self.parse_type()?;

                    self.consume(Punctuator(Colon), "where clause")?;

                    let constraint = self.parse_type()?;

                    Ok::<WhereClauseUnit, ParseError>(WhereClauseUnit { r#type, constraint })
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
