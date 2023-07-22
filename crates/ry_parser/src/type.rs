use ry_ast::{
    token::RawToken, Bounds, GenericParameter, Path, Token, Type, TypeArgument, TypePath,
    TypePathSegment, WherePredicate,
};

use crate::{
    diagnostics::UnexpectedTokenDiagnostic, expected, macros::parse_list, path::PathParser,
    OptionalParser, Parse, ParseState,
};

pub(crate) struct BoundsParser;

pub(crate) struct TypeParser;

struct TypeWithQualifiedPathParser;

struct TraitObjectTypeParser;

struct ParenthesizedTupleOrFunctionTypeParser;

struct TypePathParser;

struct TypePathSegmentParser;

pub(crate) struct GenericParametersParser;

pub(crate) struct TypeArgumentsParser;

pub(crate) struct WherePredicatesParser;

impl Parse for BoundsParser {
    type Output = Option<Bounds>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut bounds = vec![];
        bounds.push(TypePathSegmentParser.parse(state)?);

        while state.next_token.raw == Token![+] {
            state.advance();

            bounds.push(TypePathSegmentParser.parse(state)?);
        }

        Some(bounds)
    }
}

impl Parse for TypeParser {
    type Output = Option<Type>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        match state.next_token.raw {
            Token!['('] => ParenthesizedTupleOrFunctionTypeParser.parse(state),
            RawToken::Identifier => TypePathParser.parse(state).map(Type::Path),
            Token![dyn] => TraitObjectTypeParser.parse(state),
            Token!['['] => TypeWithQualifiedPathParser.parse(state),
            _ => {
                state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                    state.next_token,
                    expected!("identifier", Token!['['], Token![#], Token!['(']),
                    "type",
                ));

                None
            }
        }
    }
}

impl Parse for TypeWithQualifiedPathParser {
    type Output = Option<Type>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.location.start;
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

        Some(Type::WithQualifiedPath {
            location: state.location_from(start),
            left,
            right,
            segments,
        })
    }
}

impl Parse for TraitObjectTypeParser {
    type Output = Option<Type>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.location.start;

        state.advance(); // `dyn`

        Some(Type::TraitObject {
            bounds: BoundsParser.parse(state)?,
            location: state.location_from(start),
        })
    }
}

impl Parse for ParenthesizedTupleOrFunctionTypeParser {
    type Output = Option<Type>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.location.start;
        state.advance(); // `(`

        let element_types = parse_list!(state, "parenthesized or tuple type", Token![')'], {
            TypeParser.parse(state)
        });

        state.advance(); // `)`

        if state.next_token.raw == Token![:] {
            state.advance();

            let return_type = Box::new(TypeParser.parse(state)?);

            return Some(Type::Function {
                location: state.location_from(start),
                parameter_types: element_types,
                return_type,
            });
        }

        let location = state.location_from(start);

        let mut element_types = element_types.into_iter();

        match (element_types.next(), element_types.next()) {
            (Some(element), None) => {
                if state
                    .resolve_location(state.location_from(element.location().end))
                    .contains(',')
                {
                    Some(Type::Tuple {
                        location,
                        element_types: vec![element],
                    })
                } else {
                    Some(Type::Parenthesized {
                        location,
                        inner: Box::from(element),
                    })
                }
            }
            (None, None) => Some(Type::Tuple {
                location,
                element_types: vec![],
            }),
            (Some(previous), Some(next)) => {
                let mut new_element_types = vec![];
                new_element_types.push(previous);
                new_element_types.push(next);

                new_element_types.append(&mut element_types.collect::<Vec<_>>());

                Some(Type::Tuple {
                    location,
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

                    Some(BoundsParser.parse(state)?)
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
        let start = state.next_token.location.start;

        let mut segments = vec![];
        segments.push(TypePathSegmentParser.parse(state)?);

        while state.next_token.raw == Token![.] {
            state.advance();

            segments.push(TypePathSegmentParser.parse(state)?);
        }

        Some(TypePath {
            location: state.location_from(start),
            segments,
        })
    }
}

impl Parse for TypePathSegmentParser {
    type Output = Option<TypePathSegment>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let path = PathParser.parse(state)?;
        let type_arguments = TypeArgumentsParser.optionally_parse(state)?;

        Some(TypePathSegment {
            location: state.location_from(path.location.start),
            path,
            type_arguments,
        })
    }
}

impl OptionalParser for TypeArgumentsParser {
    type Output = Option<Option<Vec<TypeArgument>>>;

    fn optionally_parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        if state.next_token.raw != Token!['['] {
            return Some(None);
        }

        Some(self.parse(state))
    }
}
impl Parse for TypeArgumentsParser {
    type Output = Option<Vec<TypeArgument>>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let result = parse_list!(state, "generic arguments", Token![']'], {
            let ty = TypeParser.parse(state)?;

            match (state.next_token.raw, &ty) {
                (Token![=], Type::Path(TypePath { segments, .. })) => match segments.as_slice() {
                    [TypePathSegment {
                        path: Path { identifiers, .. },
                        type_arguments: None,
                        ..
                    }] if identifiers.len() == 1 => {
                        state.advance();
                        let value = TypeParser.parse(state)?;
                        Some(TypeArgument::AssociatedType {
                            name: *identifiers
                                .first()
                                .expect("Cannot get first identifier of type path"),
                            value,
                        })
                    }
                    _ => None,
                },
                _ => Some(TypeArgument::Type(ty)),
            }
        });

        state.advance();

        Some(result)
    }
}

impl OptionalParser for WherePredicatesParser {
    type Output = Option<Option<Vec<WherePredicate>>>;

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

                        Some(WherePredicate::Satisfies { ty: left, bounds: BoundsParser.parse(state)? })
                    },
                    Token![=] => {
                        state.advance();

                        Some(WherePredicate::Eq { left, right: TypeParser.parse(state)? })
                    },
                    _ => {
                        state.add_diagnostic(
                            UnexpectedTokenDiagnostic::new(
                                state.next_token,
                                expected!(Token![=], Token![:]),
                                "where clause",
                        ));

                        None
                    }
                }
            }
        )))
    }
}
