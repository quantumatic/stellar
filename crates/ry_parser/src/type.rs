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
    span::At,
    token::{Keyword::*, Punctuator::*, RawToken::*},
    Mutability,
};

pub(crate) struct PathParser;

impl Parser for PathParser {
    type Output = Path;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut path = vec![];
        let first_identifier = state.consume_identifier("path")?;
        path.push(first_identifier.inner);

        let (start, mut end) = (first_identifier.span.start, first_identifier.span.end);

        while state.next.inner == Punctuator(Dot) {
            state.advance();
            path.push(state.consume_identifier("path")?.inner);
            end = state.current.span.end;
        }

        Ok(path.at(start..end))
    }
}

pub(crate) struct PrimaryTypeParser;

impl Parser for PrimaryTypeParser {
    type Output = Type;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = state.next.span.start;
        let path = PathParser.parse_with(state)?;
        let type_annotations = TypeAnnotationsParser.optionally_parse_with(state)?;

        Ok(RawType::from(PrimaryType {
            path,
            type_annotations,
        })
        .at(start..state.current.span.end))
    }
}

pub(crate) struct ReferenceTypeParser;

impl Parser for ReferenceTypeParser {
    type Output = Type;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let start = state.current.span.start;

        let mut mutability = Mutability::immutable();

        if let Keyword(Mut) = state.next.inner {
            mutability = Mutability::mutable(state.next.span);

            state.advance();
        }

        let inner = TypeParser.parse_with(state)?;

        Ok(RawType::from(ReferenceType {
            mutability,
            inner: Box::new(inner),
        })
        .at(start..state.current.span.end))
    }
}

pub(crate) struct TypeParser;

impl Parser for TypeParser {
    type Output = Type;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let r#type = match state.next.inner {
            Identifier(..) => PrimaryTypeParser.parse_with(state)?,
            Punctuator(And) => {
                state.advance();
                let start = state.current.span.start;

                let mut mutability = Mutability::immutable();

                if state.next.inner == Keyword(Mut) {
                    mutability = Mutability::mutable(state.next.span);

                    state.advance();
                }

                let inner = TypeParser.parse_with(state)?;

                RawType::from(ReferenceType {
                    mutability,
                    inner: Box::new(inner),
                })
                .at(start..state.current.span.end)
            }
            Punctuator(OpenBracket) => {
                state.advance();
                let start = state.current.span.start;

                let inner = TypeParser.parse_with(state)?;

                state.consume(Punctuator(CloseBracket), "array type")?;

                RawType::from(ArrayType {
                    inner: Box::new(inner),
                })
                .at(start..state.current.span.end)
            }
            _ => {
                return Err(ParseError::unexpected_token(
                    state.next.clone(),
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

    fn optionally_parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if state.next.inner != Punctuator(OpenBracket) {
            return Ok(vec![]);
        }

        state.advance();

        let result = parse_list!(state, "generics", Punctuator(CloseBracket), || {
            TypeParser.parse_with(state)
        });

        state.advance();

        Ok(result)
    }
}

pub(crate) struct GenericsParser;

impl OptionalParser for GenericsParser {
    type Output = Generics;

    fn optionally_parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if state.next.inner != Punctuator(OpenBracket) {
            return Ok(vec![]);
        }

        state.advance();

        let result = parse_list!(
            state,
            "generics",
            Punctuator(CloseBracket),
            || -> ParseResult<Generic> {
                let name = state.consume_identifier("generic name")?;

                let mut constraint = None;

                if state.next.inner == Punctuator(Colon) {
                    state.advance();
                    constraint = Some(TypeParser.parse_with(state)?);
                }

                Ok(Generic { name, constraint })
            }
        );

        state.advance();

        Ok(result)
    }
}

pub(crate) struct WhereClauseParser;

impl OptionalParser for WhereClauseParser {
    type Output = WhereClause;

    fn optionally_parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        if state.next.inner != Keyword(Where) {
            return Ok(vec![]);
        }

        state.advance();

        Ok(parse_list!(
            state,
            "where clause",
            Punctuator(OpenBrace | Semicolon),
            || {
                let r#type = TypeParser.parse_with(state)?;

                state.consume(Punctuator(Colon), "where clause")?;

                let constraint = TypeParser.parse_with(state)?;

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
