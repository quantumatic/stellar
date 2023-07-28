use ry_ast::{
    token::RawToken, Bounds, GenericParameter, Token, Type, TypeConstructor, WherePredicate,
};

use crate::{
    diagnostics::UnexpectedTokenDiagnostic, expected, macros::parse_list, path::PathParser,
    OptionallyParse, Parse, ParseState,
};

pub(crate) struct BoundsParser;

pub(crate) struct TypeParser;

struct InterfaceObjectTypeParser;

struct ParenthesizedTupleOrFunctionTypeParser;

struct TypeConstructorParser;

pub(crate) struct GenericParametersParser;

pub(crate) struct TypeArgumentsParser;

pub(crate) struct WherePredicatesParser;

impl Parse for BoundsParser {
    type Output = Option<Bounds>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut bounds = vec![];
        bounds.push(TypeConstructorParser.parse(state)?);

        while state.next_token.raw == Token![+] {
            state.advance();

            bounds.push(TypeConstructorParser.parse(state)?);
        }

        Some(bounds)
    }
}

impl Parse for TypeParser {
    type Output = Option<Type>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        match state.next_token.raw {
            Token!['('] => ParenthesizedTupleOrFunctionTypeParser.parse(state),
            RawToken::Identifier => TypeConstructorParser.parse(state).map(Type::Constructor),
            Token![dyn] => InterfaceObjectTypeParser.parse(state),
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

impl Parse for InterfaceObjectTypeParser {
    type Output = Option<Type>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.location.start;

        state.advance(); // `dyn`

        Some(Type::InterfaceObject {
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

impl OptionallyParse for GenericParametersParser {
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

impl Parse for TypeConstructorParser {
    type Output = Option<TypeConstructor>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let path = PathParser.parse(state)?;
        let type_arguments = TypeArgumentsParser.optionally_parse(state)?;

        Some(TypeConstructor {
            location: state.location_from(path.location.start),
            path,
            type_arguments,
        })
    }
}

impl OptionallyParse for TypeArgumentsParser {
    type Output = Option<Option<Vec<Type>>>;

    fn optionally_parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        if state.next_token.raw != Token!['['] {
            return Some(None);
        }

        Some(self.parse(state))
    }
}
impl Parse for TypeArgumentsParser {
    type Output = Option<Vec<Type>>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let result = parse_list!(state, "type arguments", Token![']'], {
            TypeParser.parse(state)
        });

        state.advance();

        Some(result)
    }
}

impl OptionallyParse for WherePredicatesParser {
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

                state.consume(Token![:], "where predicate")?;

                Some(WherePredicate { ty: left, bounds: BoundsParser.parse(state)? })
            }
        )))
    }
}
