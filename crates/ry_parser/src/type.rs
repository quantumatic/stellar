use crate::{macros::parse_list, path::PathParser, OptionalParser, Parse, ParseState};
use ry_ast::{
    token::RawToken, GenericArgument, GenericParameter, Path, Token, TypeAst, TypeBounds, TypePath,
    TypePathSegment, WhereClause, WhereClauseItem,
};
use ry_diagnostics::{expected, parser::ParseDiagnostic, BuildDiagnostic};
use ry_workspace::span::{Span, SpanIndex};

pub(crate) struct TypeBoundsParser;

pub(crate) struct TypeParser;

struct TypeWithQualifiedPathParser;

struct TraitObjectTypeParser;

struct ParenthesizedTupleOrFunctionTypeParser;

struct TypePathParser;

struct TypePathSegmentParser;

pub(crate) struct GenericParametersParser;

pub(crate) struct GenericArgumentsParser;

pub(crate) struct WhereClauseParser;

impl Parse for TypeBoundsParser {
    type Output = Option<TypeBounds>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut bounds = vec![];
        bounds.push(TypePathParser.parse(state)?);

        while state.next_token.raw == Token![+] {
            state.advance();

            bounds.push(TypePathParser.parse(state)?);
        }

        Some(bounds)
    }
}

impl Parse for TypeParser {
    type Output = Option<TypeAst>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        match state.next_token.raw {
            Token!['('] => ParenthesizedTupleOrFunctionTypeParser.parse(state),
            RawToken::Identifier => TypePathParser.parse(state).map(TypeAst::Path),
            Token![dyn] => TraitObjectTypeParser.parse(state),
            Token!['['] => TypeWithQualifiedPathParser.parse(state),
            _ => {
                state.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: state.next_token,
                        expected: expected!("identifier", Token!['['], Token![#], Token!['(']),
                        node: "type".to_owned(),
                    }
                    .build(),
                );

                None
            }
        }
    }
}

impl Parse for TypeWithQualifiedPathParser {
    type Output = Option<TypeAst>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start();
        state.advance(); // `[`

        let left = Box::new(TypeParser.parse(state)?);
        state.consume(Token![as], "type with qualified path")?;

        let right = TypePathParser.parse(state)?;

        state.consume(Token![']'], "type with qualified path")?;
        state.consume(Token![.], "type with qualified path")?;

        let mut segments = vec![TypePathSegmentParser.parse(state)?];

        while state.next_token.raw == Token![.] {
            state.advance();

            segments.push(TypePathSegmentParser.parse(state)?);
        }

        Some(TypeAst::WithQualifiedPath {
            span: Span::new(start, state.current_token.span.end(), state.file_id),
            left,
            right,
            segments,
        })
    }
}

impl Parse for TraitObjectTypeParser {
    type Output = Option<TypeAst>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start();

        state.advance(); // `dyn`

        Some(TypeAst::TraitObject {
            bounds: TypeBoundsParser.parse(state)?,
            span: Span::new(start, state.current_token.span.end(), state.file_id),
        })
    }
}

impl Parse for ParenthesizedTupleOrFunctionTypeParser {
    type Output = Option<TypeAst>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start();
        state.advance(); // `(`

        let element_types = parse_list!(state, "parenthesized or tuple type", Token![')'], {
            TypeParser.parse(state)
        });

        state.advance(); // `)`

        if state.next_token.raw == Token![:] {
            state.advance();

            let return_type = Box::new(TypeParser.parse(state)?);

            return Some(TypeAst::Function {
                span: Span::new(
                    start,
                    state.current_token.span.end(),
                    state.current_token.span.file_id(),
                ),
                parameter_types: element_types,
                return_type,
            });
        }

        let span = Span::new(
            start,
            state.current_token.span.end(),
            state.current_token.span.file_id(),
        );

        let mut element_types = element_types.into_iter();

        match (element_types.next(), element_types.next()) {
            (Some(element), None) => {
                if state
                    .source_file
                    .source()
                    .index(Span::new(
                        element.span().end(),
                        state.current_token.span.end(),
                        state.current_token.span.file_id(),
                    ))
                    .contains(',')
                {
                    Some(TypeAst::Tuple {
                        span,
                        element_types: vec![element],
                    })
                } else {
                    Some(TypeAst::Parenthesized {
                        span,
                        inner: Box::from(element),
                    })
                }
            }
            (None, None) => Some(TypeAst::Tuple {
                span,
                element_types: vec![],
            }),
            (Some(previous), Some(next)) => {
                let mut new_element_types = vec![];
                new_element_types.push(previous);
                new_element_types.push(next);

                new_element_types.append(&mut element_types.collect::<Vec<_>>());

                Some(TypeAst::Tuple {
                    span,
                    element_types: new_element_types,
                })
            }
            _ => unreachable!(),
        }
    }
}

impl OptionalParser for GenericParametersParser {
    type Output = Option<Option<Vec<GenericParameter>>>;

    fn optionally_parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        if state.next_token.raw != Token!['['] {
            return Some(None);
        }

        state.advance();

        let result = parse_list!(state, "generic parameters", Token![']'], {
            Some(GenericParameter {
                name: state.consume_identifier("generic parameter name")?,
                bounds: if state.next_token.raw == Token![:] {
                    state.advance();

                    Some(TypeBoundsParser.parse(state)?)
                } else {
                    None
                },
                default_value: if state.next_token.raw == Token![=] {
                    state.advance();

                    Some(TypeParser.parse(state)?)
                } else {
                    None
                },
            })
        });

        state.advance();

        Some(Some(result))
    }
}

impl Parse for TypePathParser {
    type Output = Option<TypePath>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start();

        let mut segments = vec![];
        segments.push(TypePathSegmentParser.parse(state)?);

        while state.next_token.raw == Token![.] {
            state.advance();

            segments.push(TypePathSegmentParser.parse(state)?);
        }

        Some(TypePath {
            span: Span::new(start, state.current_token.span.end(), state.file_id),
            segments,
        })
    }
}

impl Parse for TypePathSegmentParser {
    type Output = Option<TypePathSegment>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let path = PathParser.parse(state)?;
        let generic_arguments = GenericArgumentsParser.optionally_parse(state)?;

        Some(TypePathSegment {
            span: Span::new(
                path.span.start(),
                state.current_token.span.end(),
                state.file_id,
            ),
            path,
            generic_arguments,
        })
    }
}

impl OptionalParser for GenericArgumentsParser {
    type Output = Option<Option<Vec<GenericArgument>>>;

    fn optionally_parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        if state.next_token.raw != Token!['['] {
            return Some(None);
        }

        Some(self.parse(state))
    }
}
impl Parse for GenericArgumentsParser {
    type Output = Option<Vec<GenericArgument>>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let result = parse_list!(state, "generic arguments", Token![']'], {
            let ty = TypeParser.parse(state)?;

            match (state.next_token.raw, &ty) {
                (Token![=], TypeAst::Path(TypePath { segments, .. })) => {
                    match segments.as_slice() {
                        [TypePathSegment {
                            path: Path { identifiers, .. },
                            generic_arguments: None,
                            ..
                        }] if identifiers.len() == 1 => {
                            state.advance();
                            let value = TypeParser.parse(state)?;
                            Some(GenericArgument::AssociatedType {
                                name: *identifiers
                                    .first()
                                    .expect("Cannot get first identifier of type path"),
                                value,
                            })
                        }
                        _ => None,
                    }
                }
                _ => Some(GenericArgument::Type(ty)),
            }
        });

        state.advance();

        Some(result)
    }
}

impl OptionalParser for WhereClauseParser {
    type Output = Option<Option<WhereClause>>;

    fn optionally_parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        if state.next_token.raw != Token![where] {
            return Some(None);
        }

        state.advance();

        Some(Some(parse_list!(
            state,
            "where clause",
            (Token!['{']) or (Token![;]),
             {
                let left = TypeParser.parse(state)?;

                match state.next_token.raw {
                    Token![:] => {
                        state.advance();

                        Some(WhereClauseItem::Satisfies { ty: left, bounds: TypeBoundsParser.parse(state)? })
                    },
                    Token![=] => {
                        state.advance();

                        Some(WhereClauseItem::Eq { left, right: TypeParser.parse(state)? })
                    },
                    _ => {
                        state.diagnostics.push(ParseDiagnostic::UnexpectedTokenError {
                            got: state.next_token,
                            expected: expected!(Token![=], Token![:]),
                            node: "where clause".to_owned(),
                        }.build());

                        None
                    }
                }
            }
        )))
    }
}
