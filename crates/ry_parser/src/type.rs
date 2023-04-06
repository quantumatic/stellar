use crate::{
    error::{expected, ParseError, ParseResult},
    macros::parse_list,
    OptionalParser, Parser, ParserState,
};
use ry_ast::{
    name::Path,
    r#type::{
        ArrayType, Generic, Generics, PrimaryType, RawType, ReferenceType, Type, TypeAnnotations,
        WhereClause, WhereClauseUnit,
    },
    token::{Keyword::*, Punctuator::*, RawToken::*},
    Mutability,
};

pub(crate) struct PathParser;

impl Parser for PathParser {
    type Output = Path;

    fn parse_with(parser: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut path = vec![];
        let first_identifier = parser.consume_identifier("path")?;
        path.push(first_identifier.inner);

        let (start, mut end) = (first_identifier.span.start(), first_identifier.span.end());

        while parser.next.inner == Punctuator(Dot) {
            parser.advance();
            path.push(parser.consume_identifier("path")?.inner);
            end = parser.current.span.end();
        }

        Ok(path.at(start..end))
    }
}

pub(crate) struct PrimaryTypeParser;

impl Parser for PrimaryTypeParser {
    type Output = Type;

    fn parse_with(parser: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = parser.next.span.start();
        let path = PathParser.parse(parser)?;
        let type_annotations = TypeAnnotationsParser.parse()?;

        RawType::from(PrimaryType {
            path,
            type_annotations,
        })
        .at(start..parser.current.span.end())
    }
}

pub(crate) struct ReferenceTypeParser;

impl Parser for ReferenceTypeParser {
    type Output = Type;

    fn parse_with(parser: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        parser.advance();

        let start = parser.current.span.start();

        let mut mutability = Mutability::immutable();

        if let Keyword(Mut) = parser.next.inner {
            mutability = Mutability::mutable(parser.next.span);

            parser.advance();
        }

        let inner = TypeParser.parse(parser)?;

        RawType::from(ReferenceType {
            mutability,
            inner: Box::new(inner),
        })
        .at(start..parser.current.span.end())
    }
}

pub(crate) struct TypeParser;

impl Parser for TypeParser {
    type Output = Type;

    fn parse_with(self, parser: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = parser.next.span.start();

        let r#type = match self.next.inner {
            Identifier(..) => PrimaryTypeParser.parse(parser)?,
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
                    expected!("identifier", Punctuator(And), Punctuator(OpenBracket)),
                    "type",
                ));
            }
        };

        Ok(r#type)
    }
}

pub(crate) struct TypeAnnotationsParser;

impl OptionalParser for TypeAnnotationsParser {
    type Output = TypeAnnotations;

    fn optionally_parse_with(parser: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if parser.next.inner != Punctuator(OpenBracket) {
            return Ok(vec![]);
        }

        parser.advance();

        let result = Some(parse_list!(
            parser,
            "generics",
            Punctuator(CloseBracket),
            false,
            || parser.parse_type()
        ));

        parser.advance();

        Ok(result)
    }
}

pub(crate) struct GenericsParser;

impl OptionalParser for GenericsParser {
    type Output = Generics;

    fn optionally_parse_with(self, parser: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if parser.next.inner != Punctuator(OpenBracket) {
            return Ok(vec![]);
        }

        parser.advance();

        let result = parse_list!(
            parser,
            "generics",
            Punctuator(CloseBracket),
            false,
            || -> ParseResult<Generic> {
                let name = parser.consume_identifier("generic name")?;

                let mut constraint = None;

                if parser.next.inner == Punctuator(Colon) {
                    parser.advance();
                    constraint = Some(parser.parse_type()?);
                }

                Ok(Generic { name, constraint })
            }
        );

        parser.advance();

        Ok(result)
    }
}

pub(crate) struct WhereClauseParser;

impl OptionalParser for WhereClauseParser {
    type Output = WhereClause;

    fn optionally_parse_with(parser: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if parser.next.inner != Keyword(Where) {
            return Ok(vec![]);
        }

        parser.advance();

        Ok(parse_list!(
            parser,
            "where clause",
            Punctuator(OpenBrace | Semicolon),
            false, // top level
            || {
                let r#type = parser.parse_type()?;

                parser.consume(Punctuator(Colon), "where clause")?;

                let constraint = parser.parse_type()?;

                Ok::<WhereClauseUnit, ParseError>(WhereClauseUnit { r#type, constraint })
            }
        ))
    }
}

// #[cfg(test)]
// mod type_tests {
//     use crate::{macros::parser_test, Parser};
//     use ry_interner::Interner;

//     parser_test!(primary_type1, "pub fun test(): i32 {}");
//     parser_test!(
//         primary_type2,
//         "pub fun div[T](a: T, b: T): Result[T, DivisionError] {}"
//     );
//     parser_test!(array_type, "pub fun test(a: [i32]) {}");
//     parser_test!(reference_type, "pub fun test(a: &mut i32): i32 {}");
//     parser_test!(negative_trait_type, "pub fun test(a: Into[string]) {}");
// }
