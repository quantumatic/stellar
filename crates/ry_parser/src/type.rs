use ry_ast::{
    token::{Keyword, Punctuator, RawToken},
    Bounds, GenericParameter, Type, TypeConstructor, WherePredicate,
};

use crate::{
    diagnostics::UnexpectedTokenDiagnostic, expected, macros::parse_list, path::PathParser,
    OptionallyParse, Parse, ParseState,
};

pub(crate) struct BoundsParser;

impl Parse for BoundsParser {
    type Output = Option<Bounds>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut bounds = vec![];
        bounds.push(TypeConstructorParser.parse(state)?);

        while state.next_token.raw == Punctuator::Plus {
            state.advance();

            bounds.push(TypeConstructorParser.parse(state)?);
        }

        Some(bounds)
    }
}

pub(crate) struct TypeParser;

impl Parse for TypeParser {
    type Output = Option<Type>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        match state.next_token.raw {
            RawToken::Punctuator(Punctuator::OpenParent) => {
                ParenthesizedTupleOrFunctionTypeParser.parse(state)
            }
            RawToken::Identifier => TypeConstructorParser.parse(state).map(Type::Constructor),
            RawToken::Keyword(Keyword::Dyn) => InterfaceObjectTypeParser.parse(state),
            _ => {
                state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                    state.next_token,
                    expected!("identifier", Punctuator::OpenParent, Keyword::Dyn),
                    "type",
                ));

                None
            }
        }
    }
}

struct InterfaceObjectTypeParser;

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

struct ParenthesizedTupleOrFunctionTypeParser;

impl Parse for ParenthesizedTupleOrFunctionTypeParser {
    type Output = Option<Type>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.location.start;
        state.advance(); // `(`

        let element_types = parse_list!(
            state,
            node_name: "parenthesized or tuple type",
            closing_token: Punctuator::CloseParent,
            parse_element_expr: TypeParser.parse(state)
        );

        state.advance(); // `)`

        if state.next_token.raw == Punctuator::Colon {
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

pub(crate) struct GenericParametersParser;

impl OptionallyParse for GenericParametersParser {
    type Output = Option<Option<Vec<GenericParameter>>>;

    fn optionally_parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        if state.next_token.raw != Punctuator::OpenBracket {
            return Some(None);
        }

        state.advance();

        let result = parse_list!(
            state,
            node_name: "generic parameters",
            closing_token: Punctuator::CloseBracket,
            parse_element_expr: {
                Some(GenericParameter {
                    name: state.consume_identifier("generic parameter name")?,
                    bounds: if state.next_token.raw == Punctuator::Colon {
                        state.advance();

                        Some(BoundsParser.parse(state)?)
                    } else {
                        None
                    },
                    default_value: if state.next_token.raw == Punctuator::Eq {
                        state.advance();

                        Some(TypeParser.parse(state)?)
                    } else {
                        None
                    },
                })
            }
        );

        state.advance();

        Some(Some(result))
    }
}

struct TypeConstructorParser;

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

pub(crate) struct TypeArgumentsParser;

impl OptionallyParse for TypeArgumentsParser {
    type Output = Option<Option<Vec<Type>>>;

    fn optionally_parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        if state.next_token.raw != Punctuator::OpenBracket {
            return Some(None);
        }

        Some(self.parse(state))
    }
}
impl Parse for TypeArgumentsParser {
    type Output = Option<Vec<Type>>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let result = parse_list!(
            state,
            node_name: "type arguments",
            closing_token: Punctuator::CloseBracket,
            parse_element_expr: TypeParser.parse(state)
        );

        state.advance();

        Some(result)
    }
}

pub(crate) struct WherePredicatesParser;

impl OptionallyParse for WherePredicatesParser {
    type Output = Option<Option<Vec<WherePredicate>>>;

    fn optionally_parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        if state.next_token.raw != Keyword::Where {
            return Some(None);
        }

        state.advance();

        Some(Some(parse_list!(
            state,
            node_name: "where clause",
            closing_token: one_of(Punctuator::CloseBrace, Punctuator::Semicolon),
            parse_element_expr: {
                let left = TypeParser.parse(state)?;

                state.consume(Punctuator::Colon, "where predicate")?;

                Some(WherePredicate { ty: left, bounds: BoundsParser.parse(state)? })
            }
        )))
    }
}
